<script lang="ts">
	import { activeFileEdits, acceptEdit, rejectEdit, acceptAllForFile, rejectAllForFile, type DevModeEdit } from '$lib/stores/devModeEdits';
	import { activeFilePath } from '$lib/stores/editor';
	import { isDevMode } from '$lib/stores/settings';

	interface Props {
		editorEl?: HTMLDivElement;
	}

	let { editorEl }: Props = $props();

	function getShortPath(path: string): string {
		const parts = path.split('/');
		return parts.length > 2 ? `.../${parts.slice(-2).join('/')}` : path;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (!$isDevMode || $activeFileEdits.length === 0) return;

		const first = $activeFileEdits[0];
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			acceptEdit(first.id);
		} else if ((e.metaKey || e.ctrlKey) && e.key === 'Backspace' && !e.shiftKey) {
			e.preventDefault();
			rejectEdit(first.id);
		} else if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'Enter') {
			e.preventDefault();
			if ($activeFilePath) acceptAllForFile($activeFilePath);
		} else if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'Backspace') {
			e.preventDefault();
			if ($activeFilePath) rejectAllForFile($activeFilePath);
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $isDevMode && $activeFileEdits.length > 0}
	<div class="inline-edit-bar">
		<div class="edit-info">
			<span class="edit-icon">✏️</span>
			<span class="edit-count">
				{$activeFileEdits.length} pending edit{$activeFileEdits.length > 1 ? 's' : ''}
			</span>
		</div>
		<div class="edit-actions">
			{#if $activeFileEdits.length > 1}
				<button class="edit-btn accept-all" onclick={() => { if ($activeFilePath) acceptAllForFile($activeFilePath); }}>
					✓ Accept All
				</button>
				<button class="edit-btn reject-all" onclick={() => { if ($activeFilePath) rejectAllForFile($activeFilePath); }}>
					✕ Reject All
				</button>
			{/if}
		</div>
	</div>

	<div class="inline-edits">
		{#each $activeFileEdits as edit (edit.id)}
			<div class="inline-edit">
				<div class="edit-diff">
					{#if edit.oldContent}
						<div class="diff-removed">
							{#each edit.oldContent.split('\n') as line}
								<div class="diff-line removed">- {line}</div>
							{/each}
						</div>
					{/if}
					<div class="diff-added">
						{#each edit.newContent.split('\n') as line}
							<div class="diff-line added">+ {line}</div>
						{/each}
					</div>
				</div>
				<div class="edit-buttons">
					<button class="edit-btn accept" onclick={() => acceptEdit(edit.id)} title="Accept (Cmd+Enter)">
						✓ Accept
					</button>
					<button class="edit-btn reject" onclick={() => rejectEdit(edit.id)} title="Reject (Cmd+Backspace)">
						✕ Reject
					</button>
				</div>
			</div>
		{/each}
	</div>
{/if}

<style>
	.inline-edit-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 12px;
		background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface));
		border-bottom: 1px solid var(--color-accent);
		font-size: 12px;
		gap: 12px;
	}
	.edit-info {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--color-accent);
		font-weight: 600;
	}
	.edit-actions {
		display: flex;
		gap: 6px;
	}

	.inline-edits {
		max-height: 300px;
		overflow-y: auto;
		border-bottom: 1px solid var(--color-border-muted);
	}
	.inline-edit {
		border-bottom: 1px solid var(--color-border-muted);
	}
	.edit-diff {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 12px;
		line-height: 1.5;
		padding: 8px 12px;
		overflow-x: auto;
	}
	.diff-line {
		white-space: pre;
	}
	.diff-line.removed {
		background: rgba(248, 81, 73, 0.1);
		color: #f85149;
	}
	.diff-line.added {
		background: rgba(63, 185, 80, 0.1);
		color: #3fb950;
	}
	.edit-buttons {
		display: flex;
		gap: 6px;
		padding: 4px 12px 8px;
	}
	.edit-btn {
		padding: 4px 12px;
		border-radius: 4px;
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		border: 1px solid var(--color-border-muted);
		background: var(--color-surface);
		color: var(--color-text);
	}
	.edit-btn.accept, .edit-btn.accept-all {
		background: rgba(63, 185, 80, 0.15);
		color: #3fb950;
		border-color: rgba(63, 185, 80, 0.3);
	}
	.edit-btn.accept:hover, .edit-btn.accept-all:hover {
		background: rgba(63, 185, 80, 0.25);
	}
	.edit-btn.reject, .edit-btn.reject-all {
		background: rgba(248, 81, 73, 0.15);
		color: #f85149;
		border-color: rgba(248, 81, 73, 0.3);
	}
	.edit-btn.reject:hover, .edit-btn.reject-all:hover {
		background: rgba(248, 81, 73, 0.25);
	}
</style>
