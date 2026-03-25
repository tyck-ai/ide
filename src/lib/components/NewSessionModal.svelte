<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { get } from 'svelte/store';
	import { agentProviders, activeProviderId } from '$lib/stores/agentProvider';
	import { agentSessions, spawnAgentSession } from '$lib/stores/agentTerminal';
	import { projectRoot } from '$lib/stores/editor';
	import { startGitPoller } from '$lib/stores/git';
	import { pickSessionName } from '$lib/utils/sessionNames';
	import SubRepoPicker, { type SubRepo } from './SubRepoPicker.svelte';

	interface Props {
		onClose: () => void;
	}

	let { onClose }: Props = $props();

	// ── Views ──────────────────────────────────────────────────────────────────
	// 'checking' → loading git context
	// 'pick'     → parent folder with sub-repos: let user choose or init here
	// 'init'     → no git at all: offer to init
	// 'form'     → normal new-session form
	type View = 'checking' | 'pick' | 'init' | 'form';
	let view = $state<View>('checking');
	let subRepos = $state<SubRepo[]>([]);
	let selectedRepo = $state<string | null>(null);

	// ── Session form state ─────────────────────────────────────────────────────
	const usedNames = new Set(get(agentSessions).map(s => s.label));
	let sessionName = $state(pickSessionName(usedNames));
	let selectedProvider = $state($activeProviderId || $agentProviders[0]?.id || '');
	let instructions = $state('');
	let branchName = $state('');
	let starting = $state(false);
	let error = $state<string | null>(null);

	// ── Git init state ─────────────────────────────────────────────────────────
	let initializingGit = $state(false);

	// ── On mount: check git context ────────────────────────────────────────────
	const cwd = get(projectRoot);

	async function checkGitContext() {
		if (!cwd) { view = 'form'; return; }
		try {
			const result = await invoke<{ isRepo: boolean; subRepos: SubRepo[] }>('find_git_context', { cwd });
			if (result.isRepo) {
				view = 'form';
			} else if (result.subRepos.length > 0) {
				subRepos = result.subRepos;
				view = 'pick';
			} else {
				view = 'init';
			}
		} catch {
			// Can't determine context — allow proceeding anyway
			view = 'form';
		}
	}

	checkGitContext();

	// ── Zoom into sub-repo ─────────────────────────────────────────────────────
	async function zoomInto(repoPath: string) {
		projectRoot.set(repoPath);
		startGitPoller(repoPath);
		invoke('notify_workspace_opened', { path: repoPath }).catch(() => {});
		const folderName = repoPath.split('/').pop() ?? repoPath;
		getCurrentWindow().setTitle(`${folderName} — tyck`).catch(() => {});
		view = 'form';
	}

	// ── Initialize git in current folder ──────────────────────────────────────
	async function initGit(targetPath: string) {
		initializingGit = true;
		try {
			await invoke('git_init_repo', { path: targetPath });
			if (targetPath !== cwd) {
				await zoomInto(targetPath);
			} else {
				startGitPoller(targetPath);
				view = 'form';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}
		initializingGit = false;
	}

	// ── Start session ──────────────────────────────────────────────────────────
	async function startSession() {
		if (starting) return;
		starting = true;
		error = null;

		try {
			const sessionId = await spawnAgentSession(
				undefined,
				selectedProvider,
				undefined,
				sessionName.trim() || undefined,
				branchName.trim() || undefined
			);

			if (instructions.trim()) {
				const msg = instructions.trim() + '\r';
				const sendWithRetry = async (attempts: number, delay: number) => {
					for (let i = 0; i < attempts; i++) {
						await new Promise(r => setTimeout(r, delay));
						try {
							await invoke('write_terminal', { id: sessionId, data: msg });
							return;
						} catch {
							// Agent not ready yet, retry
						}
					}
				};
				sendWithRetry(5, 2000).catch(() => {});
			}

			onClose();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			starting = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
		if (e.key === 'Enter' && (e.metaKey || e.ctrlKey) && view === 'form') startSession();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={onClose}>
	<div class="modal" onclick={(e) => e.stopPropagation()}>

		{#if view === 'checking'}
			<div class="checking">
				<div class="spinner"></div>
			</div>

		{:else if view === 'pick'}
			<div class="modal-title">Choose a project</div>
			<p class="modal-sub">
				This folder contains multiple projects. Which one should the agent work in?
			</p>
			<SubRepoPicker repos={subRepos} selected={selectedRepo} onSelect={(p) => selectedRepo = p} />
			{#if error}
				<div class="error">{error}</div>
			{/if}
			<div class="modal-actions split">
				<button class="btn secondary" onclick={() => cwd && initGit(cwd)} disabled={initializingGit}>
					{initializingGit ? 'Initializing...' : 'Initialize git here instead'}
				</button>
				<div class="actions-right">
					<button class="btn cancel" onclick={onClose}>Cancel</button>
					<button
						class="btn primary"
						onclick={() => selectedRepo && zoomInto(selectedRepo)}
						disabled={!selectedRepo}
					>
						Open Project
					</button>
				</div>
			</div>

		{:else if view === 'init'}
			<div class="modal-title">No git repository found</div>
			<p class="modal-sub">
				Agent Mode uses git to isolate changes in worktrees. Initialize a repository in this folder to continue.
			</p>
			<div class="init-path">
				<span class="init-path-label">Folder</span>
				<span class="init-path-value">{cwd?.split('/').pop()}</span>
			</div>
			{#if error}
				<div class="error">{error}</div>
			{/if}
			<div class="modal-actions">
				<button class="btn cancel" onclick={onClose}>Cancel</button>
				<button class="btn primary" onclick={() => cwd && initGit(cwd)} disabled={initializingGit}>
					{initializingGit ? 'Initializing...' : 'Initialize Git'}
				</button>
			</div>

		{:else}
			<div class="modal-title">New Agent Session</div>

			<label class="field">
				<span class="field-label">Name</span>
				<input
					type="text"
					class="field-input"
					bind:value={sessionName}
					placeholder="Session name"
					maxlength="32"
				/>
			</label>

			<label class="field">
				<span class="field-label">Agent</span>
				<select class="field-input" bind:value={selectedProvider}>
					{#each $agentProviders as p (p.id)}
						<option value={p.id}>{p.displayName}</option>
					{/each}
					{#if $agentProviders.length === 0}
						<option value="" disabled>No agents installed</option>
					{/if}
				</select>
			</label>

			<label class="field">
				<span class="field-label">Instructions <span class="optional">(optional)</span></span>
				<textarea
					class="field-input field-textarea"
					bind:value={instructions}
					placeholder="Describe what the agent should work on..."
					rows="3"
				></textarea>
			</label>

			<label class="field">
				<span class="field-label">Branch name <span class="optional">(optional)</span></span>
				<input
					type="text"
					class="field-input"
					bind:value={branchName}
					placeholder="tyck/claude-code/auto-generated"
				/>
			</label>

			{#if error}
				<div class="error">{error}</div>
			{/if}

			<div class="modal-actions">
				<button class="btn cancel" onclick={onClose}>Cancel</button>
				<button
					class="btn primary"
					onclick={startSession}
					disabled={starting || $agentProviders.length === 0}
				>
					{starting ? 'Starting...' : 'Start Session'}
				</button>
			</div>
		{/if}

	</div>
</div>

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}
	.modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		padding: 24px;
		width: 420px;
		max-width: 90vw;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.modal-title {
		font-size: 16px;
		font-weight: 600;
	}
	.modal-sub {
		font-size: 13px;
		color: var(--color-text-subtle);
		line-height: 1.5;
		margin: 0;
	}

	/* ── Checking ── */
	.checking {
		display: flex;
		justify-content: center;
		padding: 16px 0;
	}
	.spinner {
		width: 20px;
		height: 20px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }

	/* ── Repo picker ── */
	:global(.repo-list) {
		max-height: 220px;
		overflow-y: auto;
	}

	/* ── Init path display ── */
	.init-path {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		background: var(--color-base);
		border-radius: 7px;
		border: 1px solid var(--color-border-muted);
	}
	.init-path-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}
	.init-path-value {
		font-size: 13px;
		color: var(--color-text);
	}

	/* ── Session form ── */
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.field-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-secondary);
	}
	.optional {
		font-weight: 400;
		color: var(--color-text-subtle);
	}
	.field-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: inherit;
	}
	.field-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}
	.field-textarea {
		resize: vertical;
		min-height: 60px;
	}
	select.field-input { cursor: pointer; }

	/* ── Shared ── */
	.error {
		padding: 8px 12px;
		background: rgba(248, 81, 73, 0.1);
		border: 1px solid rgba(248, 81, 73, 0.3);
		border-radius: 6px;
		color: #f85149;
		font-size: 12px;
	}
	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 4px;
	}
	.modal-actions.split {
		justify-content: space-between;
		align-items: center;
	}
	.actions-right {
		display: flex;
		gap: 8px;
	}
	.btn {
		padding: 8px 18px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		white-space: nowrap;
	}
	.btn.cancel {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		color: var(--color-text);
	}
	.btn.secondary {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		padding-left: 0;
		font-size: 12px;
	}
	.btn.secondary:hover { color: var(--color-text); }
	.btn.primary {
		background: var(--color-accent);
		border: none;
		color: white;
	}
	.btn.primary:disabled,
	.btn.secondary:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
