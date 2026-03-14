<script lang="ts">
	import { pendingEdits, updateEditStatus, type PendingEdit } from '$lib/stores/agent';
	import { activeFilePath } from '$lib/stores/editor';

	function fileName(path: string): string {
		return path.split('/').pop() || path;
	}

	function groupByFile(edits: PendingEdit[]): Map<string, PendingEdit[]> {
		const map = new Map<string, PendingEdit[]>();
		for (const edit of edits) {
			const existing = map.get(edit.filePath) || [];
			existing.push(edit);
			map.set(edit.filePath, existing);
		}
		return map;
	}

	function acceptFile(filePath: string) {
		$pendingEdits
			.filter(e => e.filePath === filePath && e.status === 'pending')
			.forEach(e => updateEditStatus(e.toolId, 'accepted'));
	}

	function rejectFile(filePath: string) {
		$pendingEdits
			.filter(e => e.filePath === filePath && e.status === 'pending')
			.forEach(e => updateEditStatus(e.toolId, 'rejected'));
	}

	function jumpToFile(filePath: string) {
		activeFilePath.set(filePath);
	}

	let grouped = $derived(groupByFile($pendingEdits.filter(e => e.status === 'pending')));
</script>

{#if grouped.size > 0}
	<div class="change-plan">
		<div class="plan-header">
			<span class="plan-icon">&#9998;</span>
			<span class="plan-title">Change Plan</span>
			<span class="plan-count">{grouped.size} file{grouped.size > 1 ? 's' : ''}</span>
		</div>

		<div class="file-list">
			{#each [...grouped.entries()] as [filePath, edits] (filePath)}
				<div class="file-card">
					<button class="file-name" onclick={() => jumpToFile(filePath)}>
						{fileName(filePath)}
					</button>
					<div class="edit-summary">
						{edits.length} edit{edits.length > 1 ? 's' : ''}
					</div>
					<div class="file-actions">
						<button class="accept-btn" onclick={() => acceptFile(filePath)}>
							&#10003; Accept
						</button>
						<button class="reject-btn" onclick={() => rejectFile(filePath)}>
							&#10007; Reject
						</button>
					</div>
				</div>
			{/each}
		</div>
	</div>
{/if}

<style>
	.change-plan {
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		overflow: hidden;
		margin: 8px 0;
	}
	.plan-header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		border-bottom: 1px solid var(--color-border-muted);
	}
	.plan-icon {
		color: var(--color-warning);
		font-size: 14px;
	}
	.plan-title {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
	}
	.plan-count {
		font-size: 11px;
		color: var(--color-text-subtle);
		margin-left: auto;
	}
	.file-list {
		display: flex;
		flex-direction: column;
	}
	.file-card {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-muted);
	}
	.file-card:last-child {
		border-bottom: none;
	}
	.file-name {
		background: none;
		border: none;
		color: var(--color-accent);
		font-size: 12px;
		cursor: pointer;
		text-decoration: none;
	}
	.file-name:hover {
		text-decoration: underline;
	}
	.edit-summary {
		font-size: 11px;
		color: var(--color-text-subtle);
		flex: 1;
	}
	.file-actions {
		display: flex;
		gap: 4px;
	}
	.accept-btn, .reject-btn {
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		padding: 2px 8px;
		font-size: 11px;
		cursor: pointer;
	}
	.accept-btn {
		color: var(--color-success);
		border-color: color-mix(in srgb, var(--color-success) 25%, transparent);
	}
	.accept-btn:hover {
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
	}
	.reject-btn {
		color: var(--color-error);
		border-color: color-mix(in srgb, var(--color-error) 25%, transparent);
	}
	.reject-btn:hover {
		background: color-mix(in srgb, var(--color-error) 12%, transparent);
	}
</style>
