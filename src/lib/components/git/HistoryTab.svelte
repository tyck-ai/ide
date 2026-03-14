<script lang="ts">
	import { onMount } from 'svelte';
	import { git, type GitCommit, type CommitDetail } from '$lib/stores/git';

	let selectedCommit = $state<CommitDetail | null>(null);
	let loading = $state(false);
	let loadingMore = $state(false);

	onMount(() => {
		git.refreshCommits();
	});

	async function selectCommit(commit: GitCommit) {
		loading = true;
		selectedCommit = await git.getCommitDetail(commit.sha);
		loading = false;
	}

	async function loadMore() {
		loadingMore = true;
		await git.refreshCommits(50, $git.commits.length);
		loadingMore = false;
	}

	function formatDate(dateStr: string): string {
		try {
			const date = new Date(dateStr);
			return date.toLocaleDateString('en-US', {
				month: 'short',
				day: 'numeric',
				year: 'numeric',
				hour: 'numeric',
				minute: '2-digit',
			});
		} catch {
			return dateStr;
		}
	}

	function getGraphChar(index: number, parents: string[]): string {
		if (parents.length > 1) return '├';
		if (index === $git.commits.length - 1) return '└';
		return '│';
	}
</script>

<div class="history-tab">
	<div class="commit-list">
		<div class="list-header">
			<span>Commit History</span>
		</div>
		<div class="list-content">
			{#each $git.commits as commit, i (commit.sha)}
				<button 
					class="commit-item"
					class:selected={selectedCommit?.commit.sha === commit.sha}
					onclick={() => selectCommit(commit)}
				>
					<div class="graph-line">
						<span class="graph-char">{getGraphChar(i, commit.parents)}</span>
						<span class="commit-dot">●</span>
					</div>
					<div class="commit-info">
						<div class="commit-message">{commit.message}</div>
						<div class="commit-meta">
							<span class="commit-sha">{commit.shortSha}</span>
							<span class="commit-author">{commit.author}</span>
							<span class="commit-date">{commit.relativeDate}</span>
						</div>
					</div>
				</button>
			{/each}
			
			{#if $git.commits.length > 0}
				<button class="load-more" onclick={loadMore} disabled={loadingMore}>
					{loadingMore ? 'Loading...' : 'Load more commits'}
				</button>
			{/if}
		</div>
	</div>

	<div class="commit-detail">
		{#if loading}
			<div class="detail-placeholder">
				<span>Loading commit details...</span>
			</div>
		{:else if selectedCommit}
			<div class="detail-header">
				<h2 class="detail-message">{selectedCommit.commit.message}</h2>
				<div class="detail-meta">
					<div class="meta-row">
						<span class="meta-label">Author</span>
						<span class="meta-value">{selectedCommit.commit.author} &lt;{selectedCommit.commit.authorEmail}&gt;</span>
					</div>
					<div class="meta-row">
						<span class="meta-label">Date</span>
						<span class="meta-value">{formatDate(selectedCommit.commit.date)}</span>
					</div>
					<div class="meta-row">
						<span class="meta-label">SHA</span>
						<span class="meta-value sha">{selectedCommit.commit.sha}</span>
					</div>
					{#if selectedCommit.commit.parents.length > 0}
						<div class="meta-row">
							<span class="meta-label">Parents</span>
							<span class="meta-value sha">{selectedCommit.commit.parents.map(p => p.slice(0, 7)).join(', ')}</span>
						</div>
					{/if}
				</div>
			</div>

			<div class="detail-files">
				<div class="files-header">
					Files Changed ({selectedCommit.files.added.length + selectedCommit.files.modified.length + selectedCommit.files.deleted.length})
				</div>
				<div class="files-list">
					{#each selectedCommit.files.added as file (file)}
						<div class="file-item added">
							<span class="file-status">A</span>
							<span class="file-path">{file}</span>
						</div>
					{/each}
					{#each selectedCommit.files.modified as file (file)}
						<div class="file-item modified">
							<span class="file-status">M</span>
							<span class="file-path">{file}</span>
						</div>
					{/each}
					{#each selectedCommit.files.deleted as file (file)}
						<div class="file-item deleted">
							<span class="file-status">D</span>
							<span class="file-path">{file}</span>
						</div>
					{/each}
				</div>
			</div>

			<div class="detail-diff">
				<div class="diff-header">Diff</div>
				<pre class="diff-content">{selectedCommit.diff}</pre>
			</div>
		{:else}
			<div class="detail-placeholder">
				<span>Select a commit to view details</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.history-tab {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.commit-list {
		width: 380px;
		flex-shrink: 0;
		border-right: 1px solid var(--color-border-muted);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.list-header {
		padding: 12px 16px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
	}

	.list-content {
		flex: 1;
		overflow-y: auto;
	}

	.commit-item {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		width: 100%;
		padding: 10px 16px;
		background: none;
		border: none;
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-muted) 25%, transparent);
		cursor: pointer;
		text-align: left;
	}

	.commit-item:hover {
		background: var(--color-border-muted);
	}

	.commit-item.selected {
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
	}

	.graph-line {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0;
		width: 16px;
		flex-shrink: 0;
	}

	.graph-char {
		color: var(--color-border);
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 12px;
		line-height: 1;
		height: 8px;
	}

	.commit-dot {
		color: var(--color-accent);
		font-size: 10px;
	}

	.commit-info {
		flex: 1;
		min-width: 0;
	}

	.commit-message {
		color: var(--color-text);
		font-size: 13px;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-bottom: 4px;
	}

	.commit-meta {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.commit-sha {
		font-family: 'SF Mono', 'Fira Code', monospace;
		color: var(--color-accent);
	}

	.commit-author {
		max-width: 120px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.load-more {
		display: block;
		width: 100%;
		padding: 12px;
		background: none;
		border: none;
		color: var(--color-accent);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
	}

	.load-more:hover:not(:disabled) {
		background: var(--color-border-muted);
	}

	.load-more:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.commit-detail {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.detail-placeholder {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-subtle);
		font-size: 14px;
	}

	.detail-header {
		padding: 20px 24px;
		border-bottom: 1px solid var(--color-border-muted);
		background: var(--color-surface);
	}

	.detail-message {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 16px;
	}

	.detail-meta {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.meta-row {
		display: flex;
		align-items: center;
		gap: 12px;
		font-size: 12px;
	}

	.meta-label {
		width: 60px;
		color: var(--color-text-subtle);
		font-weight: 500;
	}

	.meta-value {
		color: var(--color-text);
	}

	.meta-value.sha {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 11px;
		color: var(--color-accent);
	}

	.detail-files {
		border-bottom: 1px solid var(--color-border-muted);
	}

	.files-header {
		padding: 10px 24px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		background: var(--color-surface);
	}

	.files-list {
		padding: 8px 0;
		max-height: 200px;
		overflow-y: auto;
	}

	.file-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 24px;
		font-size: 12px;
	}

	.file-status {
		width: 14px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-weight: 600;
	}

	.file-item.added .file-status {
		color: var(--color-success);
	}

	.file-item.modified .file-status {
		color: var(--color-warning);
	}

	.file-item.deleted .file-status {
		color: var(--color-error);
	}

	.file-path {
		color: var(--color-text);
	}

	.detail-diff {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.diff-header {
		padding: 10px 24px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
	}

	.diff-content {
		flex: 1;
		margin: 0;
		padding: 16px 24px;
		overflow: auto;
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 12px;
		line-height: 1.5;
		color: var(--color-text);
		background: var(--color-base);
		white-space: pre-wrap;
		word-break: break-all;
	}
</style>
