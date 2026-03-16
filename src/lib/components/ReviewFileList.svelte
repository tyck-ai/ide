<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { sessionReview, activeReview, type WorktreeFileDiff } from '$lib/stores/sessionReview';
	import { activeSessionId, activeSession } from '$lib/stores/agentTerminal';
	import { projectRoot } from '$lib/stores/editor';
	import { toast } from '$lib/stores/toast';

	let merging = $state(false);
	let pushing = $state(false);
	let hasRemote = $state(false);

	// Check if remote exists
	$effect(() => {
		if ($projectRoot) {
			invoke<boolean>('git_has_remote', { path: $projectRoot }).then(v => hasRemote = v).catch(() => hasRemote = false);
		}
	});

	async function mergeToMain() {
		const session = $activeSession;
		if (!session || !$projectRoot || merging) return;
		merging = true;
		try {
			const message = `Agent changes from ${session.branchName}\n\nSession: ${session.label}`;
			await invoke('git_merge_branch', {
				path: $projectRoot,
				sourceBranch: session.branchName,
				message,
			});
			toast.success(`Merged ${session.branchName} → main`);
		} catch (e) {
			toast.error(`Merge failed: ${e}`);
		}
		merging = false;
	}

	async function pushAndPr() {
		const session = $activeSession;
		if (!session || !$projectRoot || pushing) return;
		pushing = true;
		try {
			await invoke('git_push_branch', {
				path: session.worktreePath || $projectRoot,
				branch: session.branchName,
			});
			toast.success(`Pushed ${session.branchName}`);

			// Try to create PR via gh CLI
			try {
				const prUrl = await invoke<string>('gh_create_pr', {
					path: session.worktreePath || $projectRoot,
					title: session.instructions || session.label,
					body: `Agent session: ${session.label}\nBranch: ${session.branchName}`,
					base: 'main',
					head: session.branchName,
				});
				toast.success(`PR created: ${prUrl}`);
			} catch {
				toast.info('Branch pushed. Create a PR manually on GitHub.');
			}
		} catch (e) {
			toast.error(`Push failed: ${e}`);
		}
		pushing = false;
	}

	const modified = $derived(($activeReview?.diffs ?? []).filter(d => d.status === 'M'));
	const added = $derived(($activeReview?.diffs ?? []).filter(d => d.status === 'A'));
	const deleted = $derived(($activeReview?.diffs ?? []).filter(d => d.status === 'D'));
	const totalPending = $derived(($activeReview?.diffs ?? []).length);

	function getDecision(path: string): string | undefined {
		return $activeReview?.fileDecisions.get(path);
	}

	function getConflictReason(path: string): string | undefined {
		return $activeReview?.conflicts.get(path);
	}

	function selectFile(path: string) {
		if ($activeSessionId) {
			sessionReview.selectReviewFile($activeSessionId, path);
			sessionReview.enterReviewMode($activeSessionId);
		}
	}

	function acceptFile(path: string) {
		if ($activeSessionId) {
			sessionReview.acceptFile($activeSessionId, path);
		}
	}

	function rejectFile(path: string) {
		if ($activeSessionId) {
			sessionReview.rejectFile($activeSessionId, path);
		}
	}

	function statusIcon(status: string): string {
		switch (status) {
			case 'M': return '✎';
			case 'A': return '+';
			case 'D': return '−';
			default: return '•';
		}
	}
</script>

<div class="review-list">
	{#if totalPending === 0}
		<div class="empty">
			<span class="empty-icon">✓</span>
			<span class="empty-text">No changes to review</span>
		</div>
	{:else}
		{#if modified.length > 0}
			<div class="group">
				<div class="group-header">Modified ({modified.length})</div>
				{#each modified as diff (diff.path)}
					{@const decision = getDecision(diff.path)}
					{@const conflict = getConflictReason(diff.path)}
					<div class="file-row" class:accepted={decision === 'accepted'} class:conflict={decision === 'conflict'}>
						<button class="file-name" onclick={() => selectFile(diff.path)}>
							<span class="file-icon modified">✎</span>
							{diff.path}
						</button>
						{#if conflict}
							<span class="conflict-badge" title={conflict}>⚠</span>
						{:else if decision !== 'accepted'}
							<div class="file-actions">
								<button class="action accept" onclick={() => acceptFile(diff.path)} title="Accept">✓</button>
								<button class="action reject" onclick={() => rejectFile(diff.path)} title="Reject">✕</button>
							</div>
						{:else}
							<span class="done-badge">✓</span>
						{/if}
					</div>
				{/each}
			</div>
		{/if}

		{#if added.length > 0}
			<div class="group">
				<div class="group-header">Added ({added.length})</div>
				{#each added as diff (diff.path)}
					{@const decision = getDecision(diff.path)}
					<div class="file-row" class:accepted={decision === 'accepted'}>
						<button class="file-name" onclick={() => selectFile(diff.path)}>
							<span class="file-icon added">+</span>
							{diff.path}
						</button>
						{#if decision !== 'accepted'}
							<div class="file-actions">
								<button class="action accept" onclick={() => acceptFile(diff.path)}>✓</button>
								<button class="action reject" onclick={() => rejectFile(diff.path)}>✕</button>
							</div>
						{:else}
							<span class="done-badge">✓</span>
						{/if}
					</div>
				{/each}
			</div>
		{/if}

		{#if deleted.length > 0}
			<div class="group">
				<div class="group-header">Deleted ({deleted.length})</div>
				{#each deleted as diff (diff.path)}
					{@const decision = getDecision(diff.path)}
					<div class="file-row" class:accepted={decision === 'accepted'}>
						<button class="file-name" onclick={() => selectFile(diff.path)}>
							<span class="file-icon deleted">−</span>
							{diff.path}
						</button>
						{#if decision !== 'accepted'}
							<div class="file-actions">
								<button class="action accept" onclick={() => acceptFile(diff.path)}>✓</button>
								<button class="action reject" onclick={() => rejectFile(diff.path)}>✕</button>
							</div>
						{:else}
							<span class="done-badge">✓</span>
						{/if}
					</div>
				{/each}
			</div>
		{/if}

		<div class="review-actions">
			<button class="review-btn merge" onclick={mergeToMain} disabled={merging || totalPending === 0}>
				{merging ? 'Merging...' : 'Merge to Main'}
			</button>
			{#if hasRemote}
				<button class="review-btn push" onclick={pushAndPr} disabled={pushing || totalPending === 0}>
					{pushing ? 'Pushing...' : 'Push & PR'}
				</button>
			{/if}
		</div>
	{/if}
</div>

<style>
	.review-list {
		height: 100%;
		overflow-y: auto;
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 200px;
		gap: 8px;
		color: var(--color-text-subtle);
	}
	.empty-icon { font-size: 24px; color: var(--color-success); }
	.empty-text { font-size: 12px; }
	.group { margin-bottom: 4px; }
	.group-header {
		padding: 8px 12px;
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-subtle);
		background: var(--color-surface);
	}
	.file-row {
		display: flex;
		align-items: center;
		padding: 0 8px 0 0;
	}
	.file-row.accepted { opacity: 0.5; }
	.file-row.conflict {
		background: rgba(248, 81, 73, 0.05);
	}
	.file-name {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.file-name:hover { color: var(--color-accent); }
	.file-icon { font-size: 11px; font-weight: 700; flex-shrink: 0; }
	.file-icon.modified { color: var(--color-warning, #d29922); }
	.file-icon.added { color: var(--color-success, #3fb950); }
	.file-icon.deleted { color: var(--color-error, #f85149); }
	.file-actions {
		display: flex;
		gap: 2px;
		flex-shrink: 0;
	}
	.action {
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 3px;
		padding: 2px 6px;
		font-size: 10px;
		cursor: pointer;
		color: var(--color-text-subtle);
	}
	.action.accept:hover { color: var(--color-success); border-color: var(--color-success); }
	.action.reject:hover { color: var(--color-error); border-color: var(--color-error); }
	.done-badge { color: var(--color-success); font-size: 12px; }
	.conflict-badge { color: var(--color-warning); font-size: 14px; }

	.review-actions {
		display: flex;
		gap: 8px;
		padding: 12px;
		border-top: 1px solid var(--color-border-muted);
		margin-top: 8px;
	}
	.review-btn {
		flex: 1;
		padding: 8px 12px;
		border-radius: 6px;
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		border: none;
	}
	.review-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
	.review-btn.merge {
		background: var(--color-success, #238636);
		color: white;
	}
	.review-btn.push {
		background: var(--color-accent);
		color: white;
	}
</style>
