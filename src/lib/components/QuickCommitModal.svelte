<script lang="ts">
	import { onMount } from 'svelte';
	import { git } from '$lib/stores/git';
	import { showQuickCommit } from '$lib/stores/layout';

	let commitMessage = $state('');
	let committing = $state(false);
	let inputEl: HTMLTextAreaElement;

	onMount(() => {
		inputEl?.focus();
	});

	function close() {
		showQuickCommit.set(false);
		commitMessage = '';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			close();
		}
		if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
			e.preventDefault();
			doCommit();
		}
	}

	async function doCommit() {
		if (!commitMessage.trim() || $git.staged.length === 0 || committing) return;
		committing = true;
		const sha = await git.commit(commitMessage.trim());
		committing = false;
		if (sha) {
			close();
		}
	}

	async function doCommitAndPush() {
		if (!commitMessage.trim() || $git.staged.length === 0 || committing) return;
		committing = true;
		await git.commitAndPush(commitMessage.trim());
		committing = false;
		close();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			close();
		}
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

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={handleBackdropClick}>
	<div class="modal">
		<div class="modal-header">
			<span class="modal-title">Quick Commit</span>
			<button class="close-btn" onclick={close}>×</button>
		</div>

		<div class="modal-content">
			{#if $git.staged.length > 0}
				<div class="staged-section">
					<div class="section-label">Staged ({$git.staged.length})</div>
					<div class="staged-list">
						{#each $git.staged as file (file.path)}
							<div class="staged-file">
								<span class="status-icon" style="color: {getStatusColor(file.status)}">{getStatusIcon(file.status)}</span>
								<span class="file-path">{file.path}</span>
							</div>
						{/each}
					</div>
				</div>

				<div class="message-section">
					<textarea
						bind:this={inputEl}
						bind:value={commitMessage}
						placeholder="Commit message..."
						class="commit-input"
						rows="3"
					></textarea>
					<div class="hint">Press Cmd+Enter to commit</div>
				</div>
			{:else}
				<div class="empty-state">
					<span class="empty-text">No staged changes</span>
					<span class="empty-hint">Stage some changes before committing</span>
				</div>
			{/if}
		</div>

		<div class="modal-footer">
			<button class="cancel-btn" onclick={close}>Cancel</button>
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

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 100px;
		z-index: 1000;
	}

	.modal {
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 12px;
		width: 480px;
		display: flex;
		flex-direction: column;
		box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border-muted);
	}

	.modal-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}

	.close-btn {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 20px;
		cursor: pointer;
		padding: 0;
		line-height: 1;
	}

	.close-btn:hover {
		color: var(--color-text);
	}

	.modal-content {
		padding: 16px 20px;
	}

	.staged-section {
		margin-bottom: 16px;
	}

	.section-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 8px;
	}

	.staged-list {
		max-height: 150px;
		overflow-y: auto;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		padding: 8px 0;
	}

	.staged-file {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 12px;
		font-size: 12px;
	}

	.status-icon {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-weight: 600;
		font-size: 11px;
		width: 14px;
	}

	.file-path {
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.message-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.commit-input {
		width: 100%;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		padding: 12px;
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

	.hint {
		font-size: 11px;
		color: var(--color-text-subtle);
		text-align: right;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 24px;
		text-align: center;
	}

	.empty-text {
		font-size: 14px;
		color: var(--color-text);
	}

	.empty-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.modal-footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		padding: 16px 20px;
		border-top: 1px solid var(--color-border-muted);
	}

	.cancel-btn {
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text-subtle);
		font-size: 12px;
		font-weight: 600;
		padding: 8px 16px;
		cursor: pointer;
	}

	.cancel-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.commit-btn {
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
</style>
