<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { activeSessionId } from '$lib/stores/agentTerminal';
	import {
		sessionReview,
		activeReview,
		type WorktreeFileDiff,
		type FileDecision,
	} from '$lib/stores/sessionReview';

	const statusColors: Record<string, string> = {
		A: 'var(--color-success)',
		M: 'var(--color-accent)',
		D: 'var(--color-error)',
	};

	function decisionFor(path: string, decisions: Map<string, FileDecision>): FileDecision {
		return decisions.get(path) ?? 'pending';
	}

	function onSelectFile(path: string) {
		const sid = $activeSessionId;
		if (sid) sessionReview.selectReviewFile(sid, path);
	}

	function onAcceptFile(e: Event, path: string) {
		e.stopPropagation();
		const sid = $activeSessionId;
		if (sid) sessionReview.acceptFile(sid, path);
	}

	function onRejectFile(e: Event, path: string) {
		e.stopPropagation();
		const sid = $activeSessionId;
		if (sid) sessionReview.rejectFile(sid, path);
	}

	async function onAcceptAll() {
		const sid = $activeSessionId;
		const review = $activeReview;
		if (sid && review && review.diffs.length > 0) await sessionReview.acceptAll(sid);
	}

	function onRejectAll() {
		const sid = $activeSessionId;
		const review = $activeReview;
		if (sid && review && review.diffs.length > 0) sessionReview.rejectAll(sid);
	}

	function onExit() {
		const sid = $activeSessionId;
		if (sid) sessionReview.exitReviewMode(sid);
	}

	function onForceAccept(e: Event, path: string) {
		e.stopPropagation();
		const sid = $activeSessionId;
		if (sid) sessionReview.forceAcceptFile(sid, path);
	}

	function conflictReason(path: string, conflicts: Map<string, string>): string {
		return conflicts.get(path) ?? '';
	}

	function hasConflicts(review: { fileDecisions: Map<string, FileDecision> }): boolean {
		return Array.from(review.fileDecisions.values()).some(d => d === 'conflict');
	}

	async function askAgentToResolve() {
		const sid = $activeSessionId;
		if (!sid) return;
		
		const review = $activeReview;
		if (!review) return;

		const conflictedFiles = Array.from(review.fileDecisions.entries())
			.filter(([_, decision]) => decision === 'conflict')
			.map(([path, _]) => path);

		if (conflictedFiles.length === 0) return;

		const message = `Please resolve the merge conflicts in the following files. The files contain conflict markers (<<<<<<< ======= >>>>>>>) that need to be resolved:\n${conflictedFiles.map(f => `- ${f}`).join('\n')}\n`;
		
		await invoke('write_terminal', { id: sid, data: message + '\n' });
	}
</script>

{#if $activeReview}
	{@const review = $activeReview}
<div class="review-panel">
	<div class="review-header">
		<div class="review-title">
			<span>Review Changes</span>
			{#if review.diffs.length > 0}
				<span class="pending-count">{review.diffs.length} pending</span>
			{/if}
			<button class="exit-btn" onclick={onExit} title="Exit review">&times;</button>
		</div>
	</div>

	<div class="action-bar">
		<button class="action-btn accept" onclick={onAcceptAll} disabled={review.diffs.length === 0}>Accept All</button>
		<button class="action-btn reject" onclick={onRejectAll} disabled={review.diffs.length === 0}>Reject All</button>
	</div>

	{#if hasConflicts(review)}
		<button class="ask-agent-btn" onclick={askAgentToResolve}>
			<span class="ask-agent-icon">🤖</span>
			Ask Agent to Resolve Conflicts
		</button>
	{/if}

	<div class="file-list">
		{#each review.diffs as diff (diff.path)}
			{@const decision = decisionFor(diff.path, review.fileDecisions)}
			<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
			<div
				class="file-card"
				class:selected={diff.path === review.selectedFile}
				class:conflict={decision === 'conflict'}
				onclick={() => onSelectFile(diff.path)}
				role="button"
				tabindex="0"
			>
				<div class="file-top">
					<span class="file-status" style="color: {statusColors[diff.status] ?? 'var(--color-text-subtle)'}">
						{diff.status}
					</span>
					<span class="file-path" title={diff.path}>
						{diff.path}
					</span>
				</div>
				{#if decision === 'conflict'}
					{@const reason = conflictReason(diff.path, review.conflicts)}
					<div class="conflict-reason" title={reason}>
						{#if reason.includes('markers written')}
							⚠ Conflict markers in worktree - agent can resolve
						{:else}
							Conflict: {reason.length > 50 ? reason.slice(0, 50) + '...' : reason}
						{/if}
					</div>
				{/if}
				<div class="file-bottom">
					<div class="file-stats">
						{#if diff.additions > 0}
							<span class="stat-add">+{diff.additions}</span>
						{/if}
						{#if diff.deletions > 0}
							<span class="stat-del">-{diff.deletions}</span>
						{/if}
					</div>
				<div class="file-actions">
					{#if decision === 'conflict'}
						<button class="file-btn force" onclick={(e) => onForceAccept(e, diff.path)} title="Overwrite with agent version">Force</button>
						<button class="file-btn reject" onclick={(e) => onRejectFile(e, diff.path)} title="Keep your version">&#10005;</button>
					{:else}
						<button class="file-btn accept" onclick={(e) => onAcceptFile(e, diff.path)} title="Accept">&#10003;</button>
						<button class="file-btn reject" onclick={(e) => onRejectFile(e, diff.path)} title="Reject">&#10005;</button>
					{/if}
				</div>
				</div>
			</div>
		{/each}

		{#if review.diffs.length === 0}
			<div class="empty">No changes detected</div>
		{/if}
	</div>
</div>
{:else}
	<div class="review-panel">
		<div class="empty">No active review session</div>
	</div>
{/if}

<style>
	.review-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--color-base);
		overflow: hidden;
	}
	.review-header {
		padding: 10px 12px 8px;
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
	}
	.review-title {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: 13px;
		font-weight: 700;
		color: var(--color-text);
		margin-bottom: 6px;
	}
	.exit-btn {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 16px;
		cursor: pointer;
		padding: 0 4px;
		line-height: 1;
	}
	.exit-btn:hover {
		color: var(--color-text);
	}
	.pending-count {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-warning);
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
		padding: 2px 8px;
		border-radius: 10px;
		margin-left: auto;
		margin-right: 8px;
	}
	.action-bar {
		display: flex;
		gap: 4px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
	}
	.action-btn {
		flex: 1;
		padding: 4px 8px;
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		background: none;
		color: var(--color-text-muted);
		font-size: 10px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s;
	}
	.action-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
	.action-btn:hover:not(:disabled) {
		border-color: var(--color-border);
		color: var(--color-text);
	}
	.action-btn.accept:hover:not(:disabled) {
		border-color: var(--color-success);
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 5%, transparent);
	}
	.action-btn.reject:hover:not(:disabled) {
		border-color: var(--color-error);
		color: var(--color-error);
		background: color-mix(in srgb, var(--color-error) 5%, transparent);
	}
	.ask-agent-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		margin: 8px 12px;
		padding: 8px 12px;
		background: linear-gradient(135deg, color-mix(in srgb, var(--color-accent) 15%, transparent), color-mix(in srgb, var(--color-accent) 15%, transparent));
		border: 1px solid color-mix(in srgb, var(--color-accent) 50%, transparent);
		border-radius: 6px;
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s;
	}
	.ask-agent-btn:hover {
		background: linear-gradient(135deg, color-mix(in srgb, var(--color-accent) 25%, transparent), color-mix(in srgb, var(--color-accent) 25%, transparent));
		border-color: var(--color-accent);
		color: var(--color-text);
	}
	.ask-agent-icon {
		font-size: 14px;
	}
	.file-list {
		flex: 1;
		overflow-y: auto;
		padding: 4px 0;
	}
	.file-card {
		padding: 8px 12px;
		cursor: pointer;
		border-left: 2px solid transparent;
		transition: all 0.1s;
	}
	.file-card:hover {
		background: var(--color-surface);
	}
	.file-card.selected {
		background: var(--color-surface);
		border-left-color: var(--color-accent);
	}
	.file-card.conflict {
		border-left-color: var(--color-warning);
		background: color-mix(in srgb, var(--color-warning) 3%, transparent);
	}
	.conflict-reason {
		font-size: 10px;
		color: var(--color-warning);
		padding: 2px 0 2px 18px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.file-top {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 3px;
	}
	.file-status {
		font-size: 10px;
		font-weight: 800;
		min-width: 12px;
	}
	.file-path {
		font-size: 11px;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex: 1;
	}
	.file-bottom {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-left: 18px;
	}
	.file-stats {
		display: flex;
		gap: 6px;
	}
	.stat-add {
		font-size: 10px;
		font-weight: 700;
		color: var(--color-success);
	}
	.stat-del {
		font-size: 10px;
		font-weight: 700;
		color: var(--color-error);
	}
	.file-actions {
		display: flex;
		gap: 4px;
	}
	.file-btn {
		width: 22px;
		height: 22px;
		border: 1px solid var(--color-border-muted);
		border-radius: 3px;
		background: none;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 11px;
		transition: all 0.15s;
	}
	.file-btn.accept {
		color: var(--color-success);
	}
	.file-btn.accept:hover {
		background: color-mix(in srgb, var(--color-success) 15%, transparent);
		border-color: var(--color-success);
	}
	.file-btn.reject {
		color: var(--color-error);
	}
	.file-btn.reject:hover {
		background: color-mix(in srgb, var(--color-error) 15%, transparent);
		border-color: var(--color-error);
	}
	.file-btn.force {
		color: var(--color-warning);
		width: auto;
		padding: 0 6px;
		font-size: 9px;
		font-weight: 700;
	}
	.file-btn.force:hover {
		background: color-mix(in srgb, var(--color-warning) 15%, transparent);
		border-color: var(--color-warning);
	}
	.empty {
		padding: 24px 12px;
		text-align: center;
		color: var(--color-text-subtle);
		font-size: 12px;
	}
</style>
