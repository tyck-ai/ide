<script lang="ts">
	import { onMount } from 'svelte';
	import { git, localBranches, remoteBranches, type GitBranch } from '$lib/stores/git';

	let selectedBranch = $state<GitBranch | null>(null);
	let newBranchName = $state('');
	let creating = $state(false);
	let switching = $state(false);
	let deleting = $state(false);
	let branchCommits = $state<{ sha: string; message: string; relativeDate: string }[]>([]);

	onMount(() => {
		git.refreshBranches();
	});

	async function selectBranch(branch: GitBranch) {
		selectedBranch = branch;
		// Load recent commits for this branch would require additional backend support
		// For now, we'll just show branch info
	}

	async function checkoutBranch(branch: GitBranch) {
		if (switching || branch.isCurrent) return;
		switching = true;
		
		const isRemote = branch.isRemote;
		const localName = isRemote ? branch.name.replace('origin/', '') : branch.name;
		
		await git.checkout(localName, isRemote);
		switching = false;
	}

	async function deleteBranch(branch: GitBranch) {
		if (deleting || branch.isCurrent) return;
		
		if (!confirm(`Delete branch "${branch.name}"?`)) return;
		
		deleting = true;
		await git.deleteBranch(branch.name);
		deleting = false;
		
		if (selectedBranch?.name === branch.name) {
			selectedBranch = null;
		}
	}

	async function createBranch() {
		if (!newBranchName.trim() || creating) return;
		creating = true;
		await git.createBranch(newBranchName.trim());
		creating = false;
		newBranchName = '';
	}

	async function fetch() {
		await git.fetch();
	}
</script>

<div class="branches-tab">
	<div class="branch-list">
		<div class="list-header">
			<span>Branches</span>
			<button class="fetch-btn" onclick={fetch}>⟳ Fetch</button>
		</div>

		<div class="list-content">
			{#if $localBranches.length > 0}
				<div class="section">
					<div class="section-header">Local Branches</div>
					{#each $localBranches as branch (branch.name)}
						<button 
							class="branch-item"
							class:current={branch.isCurrent}
							class:selected={selectedBranch?.name === branch.name}
							onclick={() => selectBranch(branch)}
						>
							<span class="branch-indicator">{branch.isCurrent ? '●' : '○'}</span>
							<span class="branch-name">{branch.name}</span>
							{#if branch.ahead > 0 || branch.behind > 0}
								<span class="branch-sync">
									{#if branch.ahead > 0}<span class="ahead">↑{branch.ahead}</span>{/if}
									{#if branch.behind > 0}<span class="behind">↓{branch.behind}</span>{/if}
								</span>
							{/if}
						</button>
					{/each}
				</div>
			{/if}

			{#if $remoteBranches.length > 0}
				<div class="section">
					<div class="section-header">Remote Branches</div>
					{#each $remoteBranches as branch (branch.name)}
						<button 
							class="branch-item"
							class:selected={selectedBranch?.name === branch.name}
							onclick={() => selectBranch(branch)}
						>
							<span class="branch-indicator">○</span>
							<span class="branch-name">{branch.name}</span>
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<div class="create-section">
			<div class="section-header">Create Branch</div>
			<div class="create-row">
				<input
					bind:value={newBranchName}
					type="text"
					placeholder="new-branch-name"
					class="create-input"
					onkeydown={(e) => e.key === 'Enter' && createBranch()}
				/>
				<button 
					class="create-btn"
					onclick={createBranch}
					disabled={!newBranchName.trim() || creating}
				>
					{creating ? '...' : '+'}
				</button>
			</div>
		</div>
	</div>

	<div class="branch-detail">
		{#if selectedBranch}
			<div class="detail-header">
				<h2 class="detail-name">
					<span class="branch-icon">⎇</span>
					{selectedBranch.name}
				</h2>
				{#if selectedBranch.isCurrent}
					<span class="current-badge">Current Branch</span>
				{/if}
			</div>

			<div class="detail-info">
				<div class="info-row">
					<span class="info-label">Type</span>
					<span class="info-value">{selectedBranch.isRemote ? 'Remote' : 'Local'}</span>
				</div>
				{#if selectedBranch.lastCommit}
					<div class="info-row">
						<span class="info-label">Last Commit</span>
						<span class="info-value">{selectedBranch.lastCommit}</span>
					</div>
				{/if}
				{#if selectedBranch.ahead > 0 || selectedBranch.behind > 0}
					<div class="info-row">
						<span class="info-label">Status</span>
						<span class="info-value">
							{#if selectedBranch.ahead > 0}
								<span class="ahead">{selectedBranch.ahead} ahead</span>
							{/if}
							{#if selectedBranch.ahead > 0 && selectedBranch.behind > 0}
								<span class="separator">, </span>
							{/if}
							{#if selectedBranch.behind > 0}
								<span class="behind">{selectedBranch.behind} behind</span>
							{/if}
						</span>
					</div>
				{/if}
			</div>

			<div class="detail-actions">
				{#if !selectedBranch.isCurrent}
					<button 
						class="action-btn checkout"
						onclick={() => checkoutBranch(selectedBranch!)}
						disabled={switching}
					>
						{switching ? 'Switching...' : 'Checkout'}
					</button>
				{/if}
				{#if !selectedBranch.isCurrent && !selectedBranch.isRemote}
					<button 
						class="action-btn delete"
						onclick={() => deleteBranch(selectedBranch!)}
						disabled={deleting}
					>
						{deleting ? 'Deleting...' : 'Delete'}
					</button>
				{/if}
			</div>
		{:else}
			<div class="detail-placeholder">
				<span>Select a branch to view details</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.branches-tab {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.branch-list {
		width: 300px;
		flex-shrink: 0;
		border-right: 1px solid var(--color-border-muted);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.list-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
	}

	.fetch-btn {
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		color: var(--color-accent);
		font-size: 10px;
		font-weight: 600;
		padding: 2px 8px;
		cursor: pointer;
	}

	.fetch-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.list-content {
		flex: 1;
		overflow-y: auto;
	}

	.section {
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-muted) 25%, transparent);
	}

	.section-header {
		padding: 10px 16px 6px;
		font-size: 10px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.branch-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 16px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 13px;
		cursor: pointer;
		text-align: left;
	}

	.branch-item:hover {
		background: var(--color-border-muted);
	}

	.branch-item.selected {
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
	}

	.branch-item.current {
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
	}

	.branch-indicator {
		font-size: 10px;
		color: var(--color-text-subtle);
	}

	.branch-item.current .branch-indicator {
		color: var(--color-accent);
	}

	.branch-name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.branch-sync {
		display: flex;
		gap: 4px;
		font-size: 11px;
	}

	.ahead {
		color: var(--color-success);
	}

	.behind {
		color: var(--color-warning);
	}

	.create-section {
		border-top: 1px solid var(--color-border-muted);
		padding: 12px 16px;
		background: var(--color-surface);
	}

	.create-row {
		display: flex;
		gap: 8px;
		margin-top: 8px;
	}

	.create-input {
		flex: 1;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		padding: 8px 12px;
		color: var(--color-text);
		font-size: 12px;
		outline: none;
	}

	.create-input:focus {
		border-color: var(--color-accent);
	}

	.create-input::placeholder {
		color: var(--color-text-subtle);
	}

	.create-btn {
		background: var(--color-accent);
		border: none;
		border-radius: 6px;
		color: var(--color-base);
		font-size: 16px;
		font-weight: 600;
		padding: 0 12px;
		cursor: pointer;
	}

	.create-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.create-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.branch-detail {
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
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 24px;
		border-bottom: 1px solid var(--color-border-muted);
		background: var(--color-surface);
	}

	.detail-name {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 18px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.branch-icon {
		color: var(--color-accent);
	}

	.current-badge {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
		padding: 4px 10px;
		border-radius: 6px;
	}

	.detail-info {
		padding: 24px;
		border-bottom: 1px solid var(--color-border-muted);
	}

	.info-row {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 8px 0;
	}

	.info-label {
		width: 100px;
		font-size: 12px;
		color: var(--color-text-subtle);
		font-weight: 500;
	}

	.info-value {
		font-size: 13px;
		color: var(--color-text);
	}

	.separator {
		color: var(--color-text-subtle);
	}

	.detail-actions {
		display: flex;
		gap: 8px;
		padding: 24px;
	}

	.action-btn {
		padding: 10px 20px;
		border: none;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.action-btn.checkout {
		background: var(--color-accent);
		color: var(--color-base);
	}

	.action-btn.checkout:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.action-btn.delete {
		background: color-mix(in srgb, var(--color-error) 12%, transparent);
		color: var(--color-error);
		border: 1px solid color-mix(in srgb, var(--color-error) 25%, transparent);
	}

	.action-btn.delete:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-error) 18%, transparent);
	}
</style>
