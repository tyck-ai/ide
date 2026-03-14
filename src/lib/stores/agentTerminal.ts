import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeProvider, agentProviders } from './agentProvider';
import { projectRoot } from './editor';
import { agentStatusConnected, agentStatus, switchActiveStatus, recordStatus } from './agentStatus';
import { checkpoint } from './checkpoint';
import { sessionReview } from './sessionReview';
import { settings } from './settings';
import { activeSessionId } from './activeSession';
import { listen } from '@tauri-apps/api/event';

export { activeSessionId } from './activeSession';

export interface AgentSession {
	id: string;
	label: string;
	providerId: string;
	statusFile: string;
	resumeSessionId?: string;
}

export const agentSessions = writable<AgentSession[]>([]);

export const activeSession = derived(
	[agentSessions, activeSessionId],
	([$sessions, $id]) => $sessions.find(s => s.id === $id) ?? null
);

let sessionCounter = 0;

async function startProviderSessionDiscovery(worktreeSessionId: string): Promise<void> {
	try {
		await invoke('start_provider_session_discovery', { worktreeSessionId });
	} catch {
		// Non-critical - discovery will just not happen
	}
}

async function stopProviderSessionDiscovery(worktreeSessionId: string): Promise<void> {
	try {
		await invoke('stop_provider_session_discovery', { worktreeSessionId });
	} catch {
		// Non-critical
	}
}

export async function spawnAgentSession(resumeSessionId?: string, providerId?: string, resumeSessionPath?: string): Promise<string> {
	const provider = providerId
		? get(agentProviders).find(p => p.id === providerId) ?? get(activeProvider)
		: get(activeProvider);
	const cwd = get(projectRoot);

	// When resuming, try to find the original worktree that was used for this session
	let id: string;
	let existingWorktreePath: string | null = null;
	
	if (resumeSessionId && cwd) {
		try {
			const result = await invoke<[string, string] | null>('find_worktree_for_resume', {
				providerSessionId: resumeSessionId,
				mainCwd: cwd,
			});
			if (result) {
				id = result[0];
				existingWorktreePath = result[1];
			} else {
				id = crypto.randomUUID();
			}
		} catch {
			id = crypto.randomUUID();
		}
	} else {
		id = crypto.randomUUID();
	}

	// Initialize tyck project structure (provider-specific setup handled in Rust)
	if (cwd) {
		try {
			await invoke<string>('init_tyck', { cwd, provider: provider.id });
		} catch (e) {
			console.warn('init_tyck failed:', e);
		}
	}

	// Status file — used for Claude Code's statusLine feature
	const sessionTag = resumeSessionId || id;
	let statusFile = '';
	if (provider.id === 'claude-code') {
		try {
			statusFile = await invoke<string>('get_session_status_path', {
				cwd: cwd || '',
				provider: provider.id,
				sessionId: sessionTag,
			});
		} catch {
			statusFile = '';
		}
	}

	// Build args — get provider-specific resume args from Rust
	const args = [...provider.args];
	const reviewEnabled = get(settings).reviewEnabled;
	
	// Pass --resume when resuming an existing session
	// Always use the Claude session ID (resumeSessionId), not the worktree ID
	if (resumeSessionId) {
		const resumeArgs = await invoke<string[]>('get_resume_args', {
			provider: provider.id,
			sessionId: resumeSessionId, // Always use Claude's session ID for resume
		});
		args.push(...resumeArgs);
	}
	// For new sessions, don't pass --resume - Claude will create a new session

	// Provider-specific env
	const env = { ...provider.env };
	if (statusFile) {
		env['TYCK_STATUS_FILE'] = statusFile;
	}

	// Create worktree for agent isolation (only when review mode is enabled and cwd is a git repo)
	let agentCwd = cwd;

	if (cwd && reviewEnabled) {
		try {
			// If we already found an existing worktree, use it directly
			if (existingWorktreePath) {
				agentCwd = existingWorktreePath;
				await sessionReview.registerSession(id, cwd, existingWorktreePath);
				await sessionReview.refreshDiffs(id);
			} else {
				// Create new worktree
				const wtInfo = await invoke<{ worktreePath: string }>('create_worktree', {
					cwd,
					sessionId: id,
					providerId: provider.id,
				});
				agentCwd = wtInfo.worktreePath;
				await sessionReview.registerSession(id, cwd, wtInfo.worktreePath);
				await sessionReview.refreshDiffs(id);
			}

			// When resuming a session, symlink the original session file to the new worktree's
			// project directory so Claude Code can find it
			if (resumeSessionId && resumeSessionPath) {
				try {
					await invoke('prepare_resume_session', {
						provider: provider.id,
						originalSessionPath: resumeSessionPath,
						newCwd: agentCwd,
						sessionId: resumeSessionId,
					});
					
					// Record the association between this worktree and the provider session
					// so we can find it later when resuming again
					await invoke('set_worktree_provider_session', {
						worktreeSessionId: id,
						providerSessionId: resumeSessionId,
					});
				} catch {
					// Session file symlink failed, continue anyway
				}
			}
		} catch {
			// Fallback: use old checkpoint system
			try {
				await checkpoint.createCheckpoint(cwd, id);
			} catch (e2) {
				console.warn('create_checkpoint fallback failed:', e2);
			}
		}
	}

	await invoke('spawn_agent_terminal', {
		id,
		binary: provider.binary,
		args,
		env,
		cwd: agentCwd,
	});

	// For new sessions, start background discovery of the provider's session ID
	// This runs in the backend and survives frontend crashes
	if (cwd && reviewEnabled && !existingWorktreePath) {
		startProviderSessionDiscovery(id);
	}

	// On terminal exit: refresh worktree diffs
	if (cwd && reviewEnabled) {
		listen(`pty-exit-${id}`, async () => {
			try {
				await sessionReview.refreshDiffs(id);
			} catch {
				// non-critical
			}
		});
	}

	// Start status watcher (only meaningful for Claude Code currently)
	if (statusFile) {
		try {
			await invoke('watch_agent_status', { id, statusFile });
		} catch (e) {
			console.warn('watch_agent_status failed:', e);
		}
	}

	sessionCounter++;
	const label = resumeSessionId
		? `Resumed ${resumeSessionId.slice(0, 8)}`
		: `Session ${sessionCounter}`;

	const session: AgentSession = {
		id,
		label,
		providerId: provider.id,
		statusFile,
		resumeSessionId,
	};

	agentSessions.update(s => [...s, session]);
	activeSessionId.set(id);

	return id;
}

export function switchAgentSession(id: string) {
	activeSessionId.set(id);
	switchActiveStatus(id);
}

export async function closeAgentSession(id: string): Promise<void> {
	try {
		await invoke('kill_terminal', { id });
	} catch { /* already dead */ }

	try {
		await invoke('stop_agent_status_watch', { id });
	} catch { /* ignore */ }
	
	// Stop any background discovery for this session
	await stopProviderSessionDiscovery(id);

	// Check if there are pending (unsynced) changes - query backend directly
	// This is authoritative since backend tracks which files have been synced to main
	let hasPendingChanges = false;
	try {
		hasPendingChanges = await invoke<boolean>('worktree_has_pending_changes', { sessionId: id });
	} catch {
		// No worktree or error - assume no pending changes
	}

	if (hasPendingChanges) {
		// Just remove from local state, keep worktree for later resume
		sessionReview.removeSessionFromState(id);
	} else {
		// All changes have been synced to main, safe to cleanup
		try {
			await sessionReview.finalizeReview(id, true);
		} catch { /* may not have a worktree */ }
	}

	const sessions = get(agentSessions);
	const idx = sessions.findIndex(s => s.id === id);
	const newSessions = sessions.filter(s => s.id !== id);
	agentSessions.set(newSessions);

	if (get(activeSessionId) === id) {
		if (newSessions.length > 0) {
			const newIdx = Math.min(idx, newSessions.length - 1);
			const next = newSessions[newIdx];
			activeSessionId.set(next.id);
			switchActiveStatus(next.id);
		} else {
			activeSessionId.set(null);
			agentStatusConnected.set(false);
			agentStatus.set({
				modelId: '', modelName: '',
				contextUsedPercent: 0, contextWindowSize: 0,
				totalInputTokens: 0, totalOutputTokens: 0,
				totalCostUsd: 0, linesAdded: 0, linesRemoved: 0,
				sessionId: '',
			});
		}
	}
}

export async function resumeAgentSession(sessionId: string, providerId: string, sessionPath?: string): Promise<string> {
	return spawnAgentSession(sessionId, providerId, sessionPath);
}
