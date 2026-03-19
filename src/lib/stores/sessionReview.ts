import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { activeSessionId } from './activeSession';
import { openFiles, updateFileContent } from './editor';
import { isAgentMode } from './settings';

export interface WorktreeFileDiff {
	path: string;
	status: string; // A, M, D
	additions: number;
	deletions: number;
}

export type FileDecision = 'pending' | 'accepted' | 'rejected' | 'conflict';

export interface AcceptResult {
	success: boolean;
	conflict: boolean;
	conflictReason: string | null;
	/** True if conflict markers were written to the worktree file */
	conflictWritten: boolean;
}

export interface MergeResult {
	content: string;
	hasConflicts: boolean;
	baseContent: string;
	yoursContent: string;
	theirsContent: string;
}

export interface SessionReviewState {
	sessionId: string;
	mainCwd: string;
	worktreePath: string;
	diffs: WorktreeFileDiff[];
	fileDecisions: Map<string, FileDecision>;
	/** Maps file path → conflict reason for files that failed to apply. */
	conflicts: Map<string, string>;
	/** Files that have been processed (accepted/rejected) - excluded from future polling */
	processedFiles: Set<string>;

	reviewMode: boolean;
	selectedFile: string | null;
	error: string | null;
}

function createSessionReviewStore() {
	/** Map<sessionId, SessionReviewState> */
	const sessions = writable<Map<string, SessionReviewState>>(new Map());
	const { subscribe, update } = sessions;

	const fsUnlistens = new Map<string, () => void>();
	const refreshInFlight = new Set<string>();

	/** Paths (relative to worktree) saved by the developer — excluded from review. */
	const devEditedPaths = new Map<string, Set<string>>();

	function cleanupSession(sessionId: string) {
		const unlisten = fsUnlistens.get(sessionId);
		if (unlisten) {
			unlisten();
			fsUnlistens.delete(sessionId);
		}
		devEditedPaths.delete(sessionId);
	}

	function getSession(sessionId: string): SessionReviewState | undefined {
		return get(sessions).get(sessionId);
	}

	/** Register a new session with its worktree. Called after worktree creation. */
	async function registerSession(sessionId: string, mainCwd: string, worktreePath: string) {
		const agentMode = get(isAgentMode);
		
		update(map => {
			const next = new Map(map);
			next.set(sessionId, {
				sessionId,
				mainCwd,
				worktreePath,
				diffs: [],
				fileDecisions: new Map(),
				conflicts: new Map(),
				processedFiles: new Set(),
				reviewMode: false,
				selectedFile: null,
				error: null,
			});
			return next;
		});

		// Listen for fs-change events — fires immediately when agent writes a file.
		// Filter by window label: Tauri v2 listen() is global across all windows.
		try {
			const myLabel = getCurrentWindow().label;
			const unlisten = await listen<{ windowLabel: string }>('fs-change', (event) => {
				if (event.payload.windowLabel !== myLabel) return;
				refreshDiffs(sessionId);
			});
			fsUnlistens.set(sessionId, unlisten);
		} catch { /* non-critical */ }

		await refreshDiffs(sessionId);
		
		// Auto-enter review mode when agent mode is on
		if (agentMode) {
			await enterReviewMode(sessionId);
		}
	}

	function diffsEqual(a: WorktreeFileDiff[], b: WorktreeFileDiff[]): boolean {
		if (a.length !== b.length) return false;
		for (let i = 0; i < a.length; i++) {
			if (a[i].path !== b[i].path ||
				a[i].status !== b[i].status ||
				a[i].additions !== b[i].additions ||
				a[i].deletions !== b[i].deletions) {
				return false;
			}
		}
		return true;
	}

	async function refreshDiffs(sessionId: string) {
		if (refreshInFlight.has(sessionId)) {
			console.log('[sessionReview] refreshDiffs skipped - already in flight for', sessionId);
			return;
		}
		if (!getSession(sessionId)) {
			console.log('[sessionReview] refreshDiffs skipped - no session for', sessionId);
			return;
		}

		refreshInFlight.add(sessionId);
		try {
			const allDiffs = await invoke<WorktreeFileDiff[]>('scan_worktree_changes', {
				sessionId,
			});

			update(map => {
				const next = new Map(map);
				const s = next.get(sessionId);
				if (!s) return map;

				// Filter out processed files and files the dev saved manually
				const devPaths = devEditedPaths.get(sessionId) ?? new Set<string>();
				const diffs = allDiffs.filter(d =>
					!s.processedFiles.has(d.path) && !devPaths.has(d.path)
				);

				// Check if any conflicted files have changed (agent may have resolved them)
				const conflicts = new Map(s.conflicts);
				const decisions = new Map(s.fileDecisions);
				
				for (const [path, decision] of decisions) {
					if (decision === 'conflict') {
						const oldDiff = s.diffs.find(d => d.path === path);
						const newDiff = diffs.find(d => d.path === path);
						
						// If the diff stats changed, agent likely resolved the conflict
						// Reset to pending so user can try Accept again
						if (newDiff && oldDiff && 
							(newDiff.additions !== oldDiff.additions || newDiff.deletions !== oldDiff.deletions)) {
							decisions.set(path, 'pending');
							conflicts.delete(path);
						}
					}
				}

				// Skip update if diffs haven't changed and no conflict resolutions detected
				if (diffsEqual(diffs, s.diffs) && conflicts.size === s.conflicts.size) return map;

				// Add new files as pending
				for (const diff of diffs) {
					if (!decisions.has(diff.path)) {
						decisions.set(diff.path, 'pending');
					}
				}
				// Remove files no longer in diff
				for (const key of decisions.keys()) {
					if (!diffs.find(d => d.path === key)) {
						decisions.delete(key);
						conflicts.delete(key);
					}
				}

				next.set(sessionId, { ...s, diffs, fileDecisions: decisions, conflicts, error: null });
				return next;
			});
		} catch (e) {
			const msg = String(e);
			console.warn('[sessionReview] scan failed:', msg);
			update(map => {
				const next = new Map(map);
				const s = next.get(sessionId);
				if (s) next.set(sessionId, { ...s, error: msg });
				return next;
			});
		} finally {
			refreshInFlight.delete(sessionId);
		}
	}

	const acceptInFlight = new Set<string>();

	async function acceptFile(sessionId: string, path: string) {
		// Prevent concurrent accept calls for the same file
		const key = `${sessionId}:${path}`;
		if (acceptInFlight.has(key)) return;
		acceptInFlight.add(key);

		const session = getSession(sessionId);
		if (!session) { acceptInFlight.delete(key); return; }

		const diff = session.diffs.find(d => d.path === path);
		if (!diff) { acceptInFlight.delete(key); return; }

		try {
			const result = await invoke<AcceptResult>('accept_worktree_file', {
				sessionId,
				filePath: path,
				status: diff.status,
				force: false,
			});

			if (result.conflict) {
				update(map => {
					const next = new Map(map);
					const s = next.get(sessionId);
					if (!s) return map;
					const d = new Map(s.fileDecisions);
					d.set(path, 'conflict');
					const c = new Map(s.conflicts);
					c.set(path, result.conflictReason ?? 'File was modified outside the agent');
					next.set(sessionId, { ...s, fileDecisions: d, conflicts: c });
					return next;
				});
				
				// If conflict markers were written to worktree, refresh diffs
				// so the agent and UI can see the updated file content
				if (result.conflictWritten) {
					// Force refresh to pick up the new worktree content with markers
					refreshInFlight.delete(sessionId);
					await refreshDiffs(sessionId);
				}
			} else if (result.success) {
				reloadOpenBuffer(session.mainCwd, path);
				removeFileFromReview(sessionId, path);
			}
		} catch (e) {
			console.warn('[sessionReview] accept file failed:', path, e);
			update(map => {
				const next = new Map(map);
				const s = next.get(sessionId);
				if (!s) return map;
				const d = new Map(s.fileDecisions);
				d.set(path, 'conflict');
				const c = new Map(s.conflicts);
				c.set(path, String(e));
				next.set(sessionId, { ...s, fileDecisions: d, conflicts: c });
				return next;
			});
		} finally {
			acceptInFlight.delete(key);
		}
	}

	function rejectFile(sessionId: string, path: string) {
		removeFileFromReview(sessionId, path);
	}

	function removeFileFromReview(sessionId: string, path: string) {
		update(map => {
			const next = new Map(map);
			const s = next.get(sessionId);
			if (!s) return map;
			const newDiffs = s.diffs.filter(d => d.path !== path);
			const d = new Map(s.fileDecisions);
			d.delete(path);
			const c = new Map(s.conflicts);
			c.delete(path);
			const p = new Set(s.processedFiles);
			p.add(path);
			const newSelectedFile = s.selectedFile === path
				? (newDiffs.length > 0 ? newDiffs[0].path : null)
				: s.selectedFile;
			next.set(sessionId, { ...s, diffs: newDiffs, fileDecisions: d, conflicts: c, processedFiles: p, selectedFile: newSelectedFile });
			return next;
		});
	}

	async function acceptAll(sessionId: string) {
		const session = getSession(sessionId);
		if (!session) return;

		for (const diff of [...session.diffs]) {
			await acceptFile(sessionId, diff.path);
		}
	}

	function rejectAll(sessionId: string) {
		update(map => {
			const next = new Map(map);
			const s = next.get(sessionId);
			if (!s) return map;
			const p = new Set(s.processedFiles);
			for (const diff of s.diffs) {
				p.add(diff.path);
			}
			next.set(sessionId, {
				...s,
				diffs: [],
				fileDecisions: new Map(),
				conflicts: new Map(),
				processedFiles: p,
				selectedFile: null,
			});
			return next;
		});
	}

	async function enterReviewMode(sessionId: string) {
		await refreshDiffs(sessionId);
		update(map => {
			const next = new Map(map);
			const s = next.get(sessionId);
			if (!s) return map;
			next.set(sessionId, {
				...s,
				reviewMode: true,
				selectedFile: s.diffs.length > 0 ? s.diffs[0].path : null,
			});
			return next;
		});
	}

	function exitReviewMode(sessionId: string) {
		update(map => {
			const next = new Map(map);
			const s = next.get(sessionId);
			if (!s) return map;
			next.set(sessionId, { ...s, reviewMode: false, selectedFile: null });
			return next;
		});
	}

	function selectReviewFile(sessionId: string, path: string) {
		update(map => {
			const next = new Map(map);
			const s = next.get(sessionId);
			if (!s) return map;
			next.set(sessionId, { ...s, selectedFile: path });
			return next;
		});
	}

	/** Force-accept a conflicted file, overwriting main workspace. */
	async function forceAcceptFile(sessionId: string, path: string) {
		const session = getSession(sessionId);
		if (!session) return;

		const diff = session.diffs.find(d => d.path === path);
		if (!diff) return;

		try {
			await invoke<AcceptResult>('accept_worktree_file', {
				sessionId,
				filePath: path,
				status: diff.status,
				force: true,
			});
			reloadOpenBuffer(session.mainCwd, path);
			removeFileFromReview(sessionId, path);
		} catch (e) {
			console.warn('[sessionReview] force accept failed:', path, e);
		}
	}

	/** Resolve a conflict with user-edited merged content. */
	async function resolveConflict(sessionId: string, path: string, resolvedContent: string) {
		const session = getSession(sessionId);
		if (!session) return;

		try {
			await invoke('resolve_conflict', {
				sessionId,
				filePath: path,
				resolvedContent,
			});
			reloadOpenBuffer(session.mainCwd, path);
			removeFileFromReview(sessionId, path);
		} catch (e) {
			console.warn('[sessionReview] resolve conflict failed:', path, e);
		}
	}

	/** If a file is open in the editor, reload its content from disk. */
	function reloadOpenBuffer(mainCwd: string, filePath: string) {
		const fullPath = `${mainCwd}/${filePath}`;
		const files = get(openFiles);
		const isOpen = files.some(f => f.path === fullPath);
		if (isOpen) {
			invoke<string>('read_file', { path: fullPath })
				.then(content => updateFileContent(fullPath, content))
				.catch(() => {});
		}
	}

	/** Remove a session from local state only (keeps worktree for later resume). */
	function removeSessionFromState(sessionId: string) {
		cleanupSession(sessionId);
		update(map => {
			const next = new Map(map);
			next.delete(sessionId);
			return next;
		});
	}

	/** Remove a session and delete its worktree. */
	async function removeSession(sessionId: string) {
		cleanupSession(sessionId);
		try {
			await invoke('cleanup_worktree', { sessionId });
		} catch { /* may already be cleaned up */ }
		update(map => {
			const next = new Map(map);
			next.delete(sessionId);
			return next;
		});
	}

	/** Finalize a session review: save the review record and cleanup worktree. */
	async function finalizeReview(sessionId: string, cleanup: boolean = true) {
		const session = getSession(sessionId);
		if (!session) return;

		// Build decisions map from processed files
		const decisions: Record<string, string> = {};
		for (const path of session.processedFiles) {
			// We don't track individual decisions after processing, so mark as 'reviewed'
			// The backend will use 'rejected' as default for files not explicitly accepted
			decisions[path] = 'reviewed';
		}

		try {
			await invoke('finalize_session_review', {
				sessionId,
				decisions,
				cleanup,
			});
			console.log(`[sessionReview] Finalized review for session ${sessionId}`);
		} catch (e) {
			console.warn('[sessionReview] Failed to finalize review:', e);
		}

		// Clean up local state
		cleanupSession(sessionId);
		update(map => {
			const next = new Map(map);
			next.delete(sessionId);
			return next;
		});
	}

	/** Check if a session has pending (unreviewed) changes.
	 * 
	 * NOTE: This is for UI purposes only. For authoritative pending changes check,
	 * use the backend's worktree_has_pending_changes command which compares
	 * worktree files to main workspace.
	 */
	function hasPendingChanges(sessionId: string): boolean {
		const session = getSession(sessionId);
		if (!session) {
			return false;
		}
		
		// For UI: check if there are diffs that haven't been processed in this session
		const unprocessedDiffs = session.diffs.filter(d => !session.processedFiles.has(d.path));
		return unprocessedDiffs.length > 0;
	}


	/**
	 * Mark a file as saved by the developer so it is excluded from the review tab.
	 * The file stays excluded for the lifetime of the session.
	 */
	function markDevSaved(sessionId: string, absolutePath: string) {
		const session = getSession(sessionId);
		if (!session) return;

		// Convert absolute path to worktree-relative path
		const prefix = session.worktreePath.endsWith('/') ? session.worktreePath : session.worktreePath + '/';
		if (!absolutePath.startsWith(prefix)) return;
		const relativePath = absolutePath.slice(prefix.length);

		if (!devEditedPaths.has(sessionId)) devEditedPaths.set(sessionId, new Set());
		devEditedPaths.get(sessionId)!.add(relativePath);
	}

	return {
		subscribe,
		registerSession,
		refreshDiffs,
		acceptFile,
		rejectFile,
		acceptAll,
		rejectAll,
		enterReviewMode,
		exitReviewMode,
		selectReviewFile,
		forceAcceptFile,
		resolveConflict,
		removeSession,
		removeSessionFromState,
		removeFileFromReview,
		finalizeReview,
		hasPendingChanges,
		markDevSaved,
	};
}

export const sessionReview = createSessionReviewStore();

/** Derived: the review state for the currently active agent session. */
export const activeReview = derived(
	[sessionReview, activeSessionId],
	([$sessions, $activeId]) => {
		if (!$activeId) return null;
		return $sessions.get($activeId) ?? null;
	}
);

/** Derived: whether the active session has an active review with changes. */
export const hasActiveReview = derived(activeReview, $r => $r !== null);
