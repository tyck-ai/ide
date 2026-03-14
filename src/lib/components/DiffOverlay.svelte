<script lang="ts">
	import { pendingEdits, updateEditStatus, type PendingEdit } from '$lib/stores/agent';
	import { activeFilePath } from '$lib/stores/editor';

	function fileName(path: string): string {
		return path.split('/').pop() || path;
	}

	function preview(content: string, maxLen: number = 60): string {
		const trimmed = content.trim().replace(/\n/g, ' ');
		return trimmed.length > maxLen ? trimmed.slice(0, maxLen) + '...' : trimmed;
	}

	function jumpToFile(path: string) {
		activeFilePath.set(path);
	}

	let fileEdits = $derived(
		$pendingEdits.filter(e => e.status === 'pending')
	);
</script>

{#if fileEdits.length > 0}
	<div class="diff-overlay">
		<div class="diff-header">
			<span class="diff-icon">&#916;</span>
			<span class="diff-title">Pending Changes</span>
		</div>

		<div class="diff-list">
			{#each fileEdits as edit (edit.toolId)}
				<div class="diff-item">
					<button class="diff-file" onclick={() => jumpToFile(edit.filePath)}>
						{fileName(edit.filePath)}
					</button>

					{#if edit.oldContent}
						<div class="diff-line remove">
							<span class="diff-marker">-</span>
							<span class="diff-text">{preview(edit.oldContent)}</span>
						</div>
					{/if}
					<div class="diff-line add">
						<span class="diff-marker">+</span>
						<span class="diff-text">{preview(edit.newContent)}</span>
					</div>

					<div class="diff-actions">
						<button class="accept" onclick={() => updateEditStatus(edit.toolId, 'accepted')}>
							&#10003;
						</button>
						<button class="reject" onclick={() => updateEditStatus(edit.toolId, 'rejected')}>
							&#10007;
						</button>
					</div>
				</div>
			{/each}
		</div>
	</div>
{/if}

<style>
	.diff-overlay {
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		overflow: hidden;
		margin: 8px 0;
	}
	.diff-header {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-muted);
	}
	.diff-icon {
		color: var(--color-warning);
		font-size: 13px;
	}
	.diff-title {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}
	.diff-list {
		display: flex;
		flex-direction: column;
	}
	.diff-item {
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-muted);
	}
	.diff-item:last-child {
		border-bottom: none;
	}
	.diff-file {
		background: none;
		border: none;
		color: var(--color-accent);
		font-size: 12px;
		cursor: pointer;
		margin-bottom: 4px;
		display: block;
	}
	.diff-file:hover {
		text-decoration: underline;
	}
	.diff-line {
		display: flex;
		align-items: flex-start;
		gap: 4px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 11px;
		padding: 2px 4px;
		border-radius: 3px;
		margin: 2px 0;
	}
	.diff-line.remove {
		background: color-mix(in srgb, var(--color-error) 6%, transparent);
		color: var(--color-error);
	}
	.diff-line.add {
		background: color-mix(in srgb, var(--color-success) 6%, transparent);
		color: var(--color-success);
	}
	.diff-marker {
		font-weight: bold;
		flex-shrink: 0;
		width: 12px;
	}
	.diff-text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.diff-actions {
		display: flex;
		gap: 4px;
		margin-top: 6px;
	}
	.accept, .reject {
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 3px;
		padding: 2px 8px;
		font-size: 11px;
		cursor: pointer;
	}
	.accept {
		color: var(--color-success);
	}
	.accept:hover {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
	}
	.reject {
		color: var(--color-error);
	}
	.reject:hover {
		background: color-mix(in srgb, var(--color-error) 12%, transparent);
	}
</style>
