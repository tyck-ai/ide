<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { createGitStore, type FileStatus } from '$lib/stores/git';
	import { sessionReview } from '$lib/stores/sessionReview';
	import type { AgentSession } from '$lib/stores/agentTerminal';

	let { session }: { session: AgentSession } = $props();

	// Per-tab git store pointed at the worktree — used for staging ops and diff viewing
	const localGit = createGitStore();

	let selectedFile = $state<string | null>(null);
	let commitMessage = $state('');
	let committing = $state(false);
	let discardConfirmPath = $state<string | null>(null);
	let diffEditor = $state<any>(null);
	let editorContainer: HTMLDivElement;

	onMount(async () => {
		localGit.startWatching(session.worktreePath);

		const monaco = await import('monaco-editor');
		if (editorContainer) {
			diffEditor = monaco.editor.createDiffEditor(editorContainer, {
				theme: 'tyck-theme',
				readOnly: true,
				automaticLayout: true,
				minimap: { enabled: false },
				scrollBeyondLastLine: false,
				renderSideBySide: true,
			});
		}
	});

	onDestroy(() => {
		localGit.stopWatching();
		if (diffEditor) diffEditor.dispose();
	});

	// Load diff whenever both the selected file and editor are ready
	$effect(() => {
		if (selectedFile && diffEditor) {
			loadDiff(selectedFile);
		}
	});

	// Files the agent actually changed (from the same source as the Review tab)
	const sessionDiffs = $derived($sessionReview.get(session.id)?.diffs ?? []);
	const sessionDiffPaths = $derived(new Set(sessionDiffs.map(d => d.path)));

	// Cross-reference git status with sessionDiffs to categorize each file:
	// staged, unstaged, untracked, or committed (in worktree history but not in working tree)
	const stagedFiles = $derived($localGit.staged.filter(f => sessionDiffPaths.has(f.path)));
	const unstagedFiles = $derived($localGit.unstaged.filter(f => sessionDiffPaths.has(f.path)));
	const untrackedFiles = $derived($localGit.untracked.filter(p => sessionDiffPaths.has(p)));

	// Files the agent committed in the worktree (in sessionDiffs but not in current git status)
	const gitStatusPaths = $derived(new Set([
		...$localGit.staged.map(f => f.path),
		...$localGit.unstaged.map(f => f.path),
		...$localGit.untracked,
	]));
	const committedFiles = $derived(
		sessionDiffs.filter(d => !gitStatusPaths.has(d.path))
	);

	function selectFile(path: string) {
		selectedFile = path;
	}

	async function loadDiff(path: string) {
		try {
			const original = await localGit.getFileAtHead(path) ?? '';
			const modified = await invoke<string>('read_file', {
				path: `${session.worktreePath}/${path}`,
			}).catch(() => '');

			if (diffEditor) {
				const monaco = await import('monaco-editor');
				const oldModel = diffEditor.getModel();
				diffEditor.setModel({
					original: monaco.editor.createModel(original, undefined, monaco.Uri.parse(`agent-orig://${path}`)),
					modified: monaco.editor.createModel(modified, undefined, monaco.Uri.parse(`agent-mod://${path}`)),
				});
				oldModel?.original?.dispose();
				oldModel?.modified?.dispose();
			}
		} catch (e) {
			console.error('Failed to load diff:', e);
		}
	}

	async function stageFile(path: string) {
		await localGit.stage([path]);
	}

	async function unstageFile(file: FileStatus) {
		await localGit.unstage([file.path]);
	}

	function confirmDiscard(path: string) {
		discardConfirmPath = path;
	}

	async function doConfirmDiscard() {
		if (!discardConfirmPath) return;
		const path = discardConfirmPath;
		discardConfirmPath = null;
		await localGit.discardFile(path);
		if (selectedFile === path) selectedFile = null;
	}

	async function stageAll() {
		await localGit.stageAll();
	}

	async function doCommit() {
		if (!commitMessage.trim() || committing) return;
		committing = true;
		const sha = await localGit.commit(commitMessage.trim());
		committing = false;
		if (sha) commitMessage = '';
	}

	async function doCommitAndPush() {
		if (!commitMessage.trim() || committing) return;
		committing = true;
		const sha = await localGit.commit(commitMessage.trim());
		if (sha) {
			// Always set upstream — agent branches are new and have no remote tracking
			await localGit.push(true);
			commitMessage = '';
		}
		committing = false;
	}

	function getStatusIcon(status: string): string {
		switch (status) {
			case 'M': return 'M';
			case 'A': return 'A';
			case 'D': return 'D';
			case 'R': return 'R';
			default: return '?';
		}
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'M': return 'var(--color-warning)';
			case 'A': return 'var(--color-success)';
			case 'D': return 'var(--color-error)';
			case 'R': return 'var(--color-accent)';
			default: return 'var(--color-text-subtle)';
		}
	}
</script>

<div class="agent-changes-tab">
	<div class="file-list">
		<div class="workspace-badge">
			<span class="badge-dot">●</span>
			<span class="badge-label">live workspace</span>
		</div>

		{#if stagedFiles.length > 0}
			<div class="section">
				<div class="section-header">
					<span>Staged Changes ({stagedFiles.length})</span>
				</div>
				{#each stagedFiles as file (file.path)}
					<div class="file-item" class:selected={selectedFile === file.path}>
						<button class="file-btn" onclick={() => selectFile(file.path)}>
							<span class="status-icon" style="color: {getStatusColor(file.status)}">{getStatusIcon(file.status)}</span>
							<span class="file-path">{file.path}</span>
						</button>
						<button class="action-btn unstage" onclick={() => unstageFile(file)} title="Unstage">−</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if unstagedFiles.length > 0}
			<div class="section">
				<div class="section-header">
					<span>Changes ({unstagedFiles.length})</span>
					<button class="stage-all-btn" onclick={stageAll}>Stage All</button>
				</div>
				{#each unstagedFiles as file (file.path)}
					<div class="file-item" class:selected={selectedFile === file.path}>
						<button class="file-btn" onclick={() => selectFile(file.path)}>
							<span class="status-icon" style="color: {getStatusColor(file.status)}">{getStatusIcon(file.status)}</span>
							<span class="file-path">{file.path}</span>
						</button>
						<button class="action-btn stage" onclick={() => stageFile(file.path)} title="Stage">+</button>
						<button class="action-btn discard" onclick={() => confirmDiscard(file.path)} title="Discard">×</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if untrackedFiles.length > 0}
			<div class="section">
				<div class="section-header">
					<span>Untracked ({untrackedFiles.length})</span>
				</div>
				{#each untrackedFiles as path (path)}
					<div class="file-item" class:selected={selectedFile === path}>
						<button class="file-btn" onclick={() => selectFile(path)}>
							<span class="status-icon" style="color: var(--color-success)">A</span>
							<span class="file-path">{path}</span>
						</button>
						<button class="action-btn stage" onclick={() => stageFile(path)} title="Stage">+</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if committedFiles.length > 0}
			<div class="section">
				<div class="section-header">
					<span>Committed ({committedFiles.length})</span>
				</div>
				{#each committedFiles as diff (diff.path)}
					<div class="file-item" class:selected={selectedFile === diff.path}>
						<button class="file-btn" onclick={() => selectFile(diff.path)}>
							<span class="status-icon" style="color: {getStatusColor(diff.status)}">{getStatusIcon(diff.status)}</span>
							<span class="file-path">{diff.path}</span>
						</button>
						<span class="committed-badge">✓</span>
					</div>
				{/each}
			</div>
		{/if}

		{#if sessionDiffs.length === 0}
			<div class="empty-state">
				<span class="empty-icon">✓</span>
				<span class="empty-text">No agent changes</span>
			</div>
		{/if}

		<div class="commit-section">
			<textarea
				bind:value={commitMessage}
				placeholder="Commit message..."
				class="commit-input"
				rows="3"
			></textarea>
			<div class="commit-actions">
				<button
					class="commit-btn"
					onclick={doCommit}
					disabled={!commitMessage.trim() || $localGit.staged.length === 0 || committing}
				>
					{committing ? 'Committing...' : 'Commit'}
				</button>
				<button
					class="commit-btn push"
					onclick={doCommitAndPush}
					disabled={!commitMessage.trim() || $localGit.staged.length === 0 || committing}
				>
					Commit & Push
				</button>
			</div>
		</div>
	</div>

	<div class="diff-view">
		{#if selectedFile}
			<div class="diff-header">
				<span class="diff-path">{selectedFile}</span>
			</div>
		{:else}
			<div class="diff-placeholder">
				<span>Select a file to view changes</span>
			</div>
		{/if}
		<div class="diff-editor" bind:this={editorContainer}></div>
	</div>
</div>

{#if discardConfirmPath}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="discard-overlay" onclick={() => discardConfirmPath = null}>
		<div class="discard-modal" onclick={(e) => e.stopPropagation()}>
			<div class="discard-modal__title">Discard Changes</div>
			<div class="discard-modal__text">
				Discard all changes to <strong>{discardConfirmPath}</strong>? This cannot be undone.
			</div>
			<div class="discard-modal__actions">
				<button class="discard-modal__btn cancel" onclick={() => discardConfirmPath = null}>Cancel</button>
				<button class="discard-modal__btn danger" onclick={doConfirmDiscard}>Discard</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.agent-changes-tab {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.file-list {
		width: 320px;
		flex-shrink: 0;
		border-right: 1px solid var(--color-border-muted);
		display: flex;
		flex-direction: column;
		overflow-y: auto;
	}

	.workspace-badge {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--color-border-muted);
		background: color-mix(in srgb, var(--color-success) 6%, transparent);
		flex-shrink: 0;
	}

	.badge-dot {
		font-size: 8px;
		color: var(--color-success);
		animation: pulse 2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.4; }
	}

	.badge-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-success);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.section {
		border-bottom: 1px solid var(--color-border-muted);
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 16px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		background: var(--color-surface);
	}

	.stage-all-btn {
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		color: var(--color-accent);
		font-size: 10px;
		font-weight: 600;
		padding: 2px 8px;
		cursor: pointer;
	}

	.stage-all-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.file-item {
		display: flex;
		align-items: center;
		padding: 0 8px 0 0;
	}

	.file-item:hover {
		background: var(--color-overlay);
	}

	.file-item.selected {
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
	}

	.file-btn {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 12px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
		min-width: 0;
	}

	.status-icon {
		font-size: 11px;
		font-weight: 700;
		font-family: 'SF Mono', 'Fira Code', monospace;
		width: 14px;
		flex-shrink: 0;
	}

	.file-path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.action-btn {
		background: none;
		border: none;
		width: 22px;
		height: 22px;
		border-radius: 4px;
		font-size: 14px;
		cursor: pointer;
		opacity: 0;
		transition: opacity 0.1s;
	}

	.file-item:hover .action-btn {
		opacity: 1;
	}

	.action-btn.stage { color: var(--color-success); }
	.action-btn.unstage { color: var(--color-warning); }
	.action-btn.discard { color: var(--color-error); }

	.action-btn:hover {
		background: var(--color-border);
	}

	.committed-badge {
		font-size: 11px;
		color: var(--color-success);
		padding: 0 8px;
		opacity: 0.6;
	}

	.empty-state {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		color: var(--color-text-subtle);
		padding: 40px;
	}

	.empty-icon {
		font-size: 32px;
		color: var(--color-success);
	}

	.empty-text { font-size: 14px; }

	.commit-section {
		margin-top: auto;
		padding: 12px;
		border-top: 1px solid var(--color-border-muted);
		background: var(--color-surface);
		flex-shrink: 0;
	}

	.commit-input {
		width: 100%;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		padding: 10px 12px;
		color: var(--color-text);
		font-size: 13px;
		font-family: inherit;
		resize: none;
		outline: none;
		box-sizing: border-box;
	}

	.commit-input:focus { border-color: var(--color-accent); }
	.commit-input::placeholder { color: var(--color-text-subtle); }

	.commit-actions {
		display: flex;
		gap: 8px;
		margin-top: 8px;
	}

	.commit-btn {
		flex: 1;
		background: var(--color-overlay);
		border: none;
		border-radius: 6px;
		color: var(--color-text);
		font-size: 12px;
		font-weight: 600;
		padding: 8px 16px;
		cursor: pointer;
	}

	.commit-btn:hover:not(:disabled) { background: var(--color-border); }
	.commit-btn:disabled { opacity: 0.5; cursor: not-allowed; }

	.commit-btn.push {
		background: var(--color-accent);
		color: var(--color-base);
	}

	.commit-btn.push:hover:not(:disabled) { filter: brightness(1.1); }

	.diff-view {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.diff-header {
		display: flex;
		align-items: center;
		padding: 10px 16px;
		border-bottom: 1px solid var(--color-border-muted);
		background: var(--color-surface);
	}

	.diff-path {
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text);
	}

	.diff-placeholder {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-subtle);
		font-size: 14px;
	}

	.diff-editor {
		flex: 1;
		overflow: hidden;
	}

	.discard-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}

	.discard-modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 20px;
		max-width: 400px;
		width: 90%;
	}

	.discard-modal__title {
		font-size: 15px;
		font-weight: 600;
		margin-bottom: 12px;
	}

	.discard-modal__text {
		font-size: 13px;
		color: var(--color-text-subtle);
		line-height: 1.5;
		margin-bottom: 16px;
	}

	.discard-modal__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	.discard-modal__btn {
		padding: 6px 14px;
		border-radius: 4px;
		font-size: 13px;
		cursor: pointer;
		border: 1px solid var(--color-border);
	}

	.discard-modal__btn.cancel { background: var(--color-surface); color: var(--color-text); }
	.discard-modal__btn.danger { background: var(--color-error); color: white; border-color: var(--color-error); }
</style>
