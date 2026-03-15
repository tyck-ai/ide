<script lang="ts">
	import { onMount } from 'svelte';
	import { git, type FileStatus } from '$lib/stores/git';
	import { invoke } from '@tauri-apps/api/core';
	import { projectRoot } from '$lib/stores/editor';

	let selectedFile = $state<{ path: string; staged: boolean } | null>(null);
	let originalContent = $state('');
	let modifiedContent = $state('');
	let commitMessage = $state('');
	let committing = $state(false);
	let discardConfirmPath = $state<string | null>(null);
	let diffEditor: any = null;
	let editorContainer: HTMLDivElement;

	onMount(() => {
		const monaco = (window as any).monaco;
		if (monaco && editorContainer) {
			diffEditor = monaco.editor.createDiffEditor(editorContainer, {
				theme: 'vs-dark',
				readOnly: true,
				automaticLayout: true,
				minimap: { enabled: false },
				scrollBeyondLastLine: false,
				renderSideBySide: true,
			});
		}

		return () => {
			if (diffEditor) {
				diffEditor.dispose();
			}
		};
	});

	async function selectFile(path: string, staged: boolean) {
		selectedFile = { path, staged };
		await loadDiff(path, staged);
	}

	async function loadDiff(path: string, staged: boolean) {
		const root = $projectRoot;
		if (!root) return;

		try {
			const original = await git.getFileAtHead(path) ?? '';
			const modified = await invoke<string>('read_file', { path: `${root}/${path}` }).catch(() => '');
			
			originalContent = original;
			modifiedContent = modified;

			if (diffEditor) {
				const monaco = (window as any).monaco;
				diffEditor.setModel({
					original: monaco.editor.createModel(original, undefined, monaco.Uri.parse(`original://${path}`)),
					modified: monaco.editor.createModel(modified, undefined, monaco.Uri.parse(`modified://${path}`)),
				});
			}
		} catch (e) {
			console.error('Failed to load diff:', e);
		}
	}

	async function stageFile(file: FileStatus | string) {
		const path = typeof file === 'string' ? file : file.path;
		await git.stage([path]);
	}

	async function unstageFile(file: FileStatus) {
		await git.unstage([file.path]);
	}

	async function stageUntracked(path: string) {
		await git.stage([path]);
	}

	function discardFile(file: FileStatus | string) {
		const path = typeof file === 'string' ? file : file.path;
		discardConfirmPath = path;
	}

	async function confirmDiscard() {
		if (!discardConfirmPath) return;
		const path = discardConfirmPath;
		discardConfirmPath = null;
		await git.discardFile(path);
		if (selectedFile?.path === path) {
			selectedFile = null;
		}
	}

	async function stageAll() {
		await git.stageAll();
	}

	async function doCommit() {
		if (!commitMessage.trim() || committing) return;
		committing = true;
		const sha = await git.commit(commitMessage.trim());
		committing = false;
		if (sha) {
			commitMessage = '';
		}
	}

	async function doCommitAndPush() {
		if (!commitMessage.trim() || committing) return;
		committing = true;
		await git.commitAndPush(commitMessage.trim());
		committing = false;
		commitMessage = '';
	}

	function getStatusIcon(status: string): string {
		switch (status) {
			case 'M': return 'M';
			case 'A': return 'A';
			case 'D': return 'D';
			case 'R': return 'R';
			case 'C': return 'C';
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

<div class="changes-tab">
	<div class="file-list">
		{#if $git.staged.length > 0}
			<div class="section">
				<div class="section-header">
					<span>Staged Changes ({$git.staged.length})</span>
				</div>
				{#each $git.staged as file (file.path)}
					<div 
						class="file-item"
						class:selected={selectedFile?.path === file.path && selectedFile?.staged}
					>
						<button 
							class="file-btn"
							onclick={() => selectFile(file.path, true)}
						>
							<span class="status-icon" style="color: {getStatusColor(file.status)}">{getStatusIcon(file.status)}</span>
							<span class="file-path">{file.path}</span>
						</button>
						<button class="action-btn unstage" onclick={() => unstageFile(file)} title="Unstage">−</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if $git.unstaged.length > 0}
			<div class="section">
				<div class="section-header">
					<span>Changes ({$git.unstaged.length})</span>
					<button class="stage-all-btn" onclick={stageAll}>Stage All</button>
				</div>
				{#each $git.unstaged as file (file.path)}
					<div 
						class="file-item"
						class:selected={selectedFile?.path === file.path && !selectedFile?.staged}
					>
						<button 
							class="file-btn"
							onclick={() => selectFile(file.path, false)}
						>
							<span class="status-icon" style="color: {getStatusColor(file.status)}">{getStatusIcon(file.status)}</span>
							<span class="file-path">{file.path}</span>
						</button>
						<button class="action-btn stage" onclick={() => stageFile(file)} title="Stage">+</button>
						<button class="action-btn discard" onclick={() => discardFile(file)} title="Discard">×</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if $git.untracked.length > 0}
			<div class="section">
				<div class="section-header">
					<span>Untracked ({$git.untracked.length})</span>
				</div>
				{#each $git.untracked as path (path)}
					<div 
						class="file-item"
						class:selected={selectedFile?.path === path}
					>
						<button 
							class="file-btn"
							onclick={() => selectFile(path, false)}
						>
							<span class="status-icon" style="color: var(--color-text-subtle)">?</span>
							<span class="file-path">{path}</span>
						</button>
						<button class="action-btn stage" onclick={() => stageUntracked(path)} title="Stage">+</button>
						<button class="action-btn discard" onclick={() => discardFile(path)} title="Delete">×</button>
					</div>
				{/each}
			</div>
		{/if}

		{#if $git.conflicts.length > 0}
			<div class="section conflicts">
				<div class="section-header">
					<span>Conflicts ({$git.conflicts.length})</span>
				</div>
				<div class="conflict-help">
					Resolve conflicts in each file, then stage to mark resolved.
				</div>
				{#each $git.conflicts as path (path)}
					<div class="file-item conflict">
						<button class="file-btn" onclick={() => selectFile(path, false)}>
							<span class="status-icon" style="color: var(--color-error)">!</span>
							<span class="file-path">{path}</span>
						</button>
						<div class="file-actions">
							<button class="action-btn" title="Stage (mark as resolved)" onclick={() => git.stage([path])}>✓</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}

		{#if $git.staged.length === 0 && $git.unstaged.length === 0 && $git.untracked.length === 0}
			<div class="empty-state">
				<span class="empty-icon">✓</span>
				<span class="empty-text">No changes</span>
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
					disabled={!commitMessage.trim() || $git.staged.length === 0 || committing}
				>
					{committing ? 'Committing...' : 'Commit'}
				</button>
				<button 
					class="commit-btn push"
					onclick={doCommitAndPush}
					disabled={!commitMessage.trim() || $git.staged.length === 0 || committing}
				>
					Commit & Push
				</button>
			</div>
		</div>
	</div>

	<div class="diff-view">
		{#if selectedFile}
			<div class="diff-header">
				<span class="diff-path">{selectedFile.path}</span>
				<span class="diff-type">{selectedFile.staged ? 'Staged' : 'Working'}</span>
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
				<button class="discard-modal__btn danger" onclick={confirmDiscard}>Discard</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.discard-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}
	.discard-modal {
		background: var(--color-surface, #1e1e1e);
		border: 1px solid var(--color-border, #3c3c3c);
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
		color: var(--color-text-secondary, #a0a0a0);
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
		border: 1px solid var(--color-border, #3c3c3c);
	}
	.discard-modal__btn.cancel {
		background: var(--color-surface, #2d2d2d);
		color: var(--color-text, #e0e0e0);
	}
	.discard-modal__btn.danger {
		background: var(--color-error, #da3633);
		color: white;
		border-color: var(--color-error, #da3633);
	}
	.changes-tab {
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

	.section {
		border-bottom: 1px solid var(--color-border-muted);
	}

	.section.conflicts {
		background: color-mix(in srgb, var(--color-error) 5%, transparent);
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

	.conflict-help {
		padding: 6px 16px;
		font-size: 11px;
		color: var(--color-warning, #fbbf24);
		background: rgba(251, 191, 36, 0.08);
		border-bottom: 1px solid var(--color-border-muted);
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

	.file-item.conflict {
		background: color-mix(in srgb, var(--color-error) 5%, transparent);
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

	.action-btn.stage {
		color: var(--color-success);
	}

	.action-btn.unstage {
		color: var(--color-warning);
	}

	.action-btn.discard {
		color: var(--color-error);
	}

	.action-btn:hover {
		background: var(--color-border);
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

	.empty-text {
		font-size: 14px;
	}

	.commit-section {
		margin-top: auto;
		padding: 12px;
		border-top: 1px solid var(--color-border-muted);
		background: var(--color-surface);
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
	}

	.commit-input:focus {
		border-color: var(--color-accent);
	}

	.commit-input::placeholder {
		color: var(--color-text-subtle);
	}

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

	.commit-btn:hover:not(:disabled) {
		background: var(--color-border);
	}

	.commit-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.commit-btn.push {
		background: var(--color-accent);
		color: var(--color-base);
	}

	.commit-btn.push:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.diff-view {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.diff-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 16px;
		border-bottom: 1px solid var(--color-border-muted);
		background: var(--color-surface);
	}

	.diff-path {
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text);
	}

	.diff-type {
		font-size: 11px;
		color: var(--color-text-subtle);
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
</style>
