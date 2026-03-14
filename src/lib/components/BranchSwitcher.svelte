<script lang="ts">
	import { onMount } from 'svelte';
	import { git, localBranches, remoteBranches, hasChanges } from '$lib/stores/git';
	import { showBranchSwitcher } from '$lib/stores/layout';

	let searchQuery = $state('');
	let newBranchName = $state('');
	let creating = $state(false);
	let switching = $state(false);
	let stashFirst = $state(false);
	let inputEl: HTMLInputElement;

	onMount(() => {
		git.refreshBranches();
		inputEl?.focus();
	});

	const filteredLocal = $derived(
		$localBranches.filter(b => 
			b.name.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);

	const filteredRemote = $derived(
		$remoteBranches.filter(b => 
			b.name.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);

	function close() {
		showBranchSwitcher.set(false);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			close();
		}
	}

	async function switchToBranch(branchName: string) {
		if (switching) return;
		
		if ($hasChanges && !stashFirst) {
			// Could show a warning here
		}

		switching = true;
		
		if (stashFirst && $hasChanges) {
			await git.stashCreate('Auto-stash before switching branches');
		}

		const isRemote = branchName.startsWith('origin/');
		const localName = isRemote ? branchName.replace('origin/', '') : branchName;
		
		const success = await git.checkout(localName, isRemote);
		
		switching = false;
		if (success) {
			close();
		}
	}

	async function createBranch() {
		if (!newBranchName.trim() || creating) return;
		
		creating = true;
		const success = await git.createBranch(newBranchName.trim());
		creating = false;
		
		if (success) {
			newBranchName = '';
			close();
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			close();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={handleBackdropClick}>
	<div class="modal">
		<div class="modal-header">
			<span class="modal-title">Switch Branch</span>
			<button class="close-btn" onclick={close}>×</button>
		</div>

		<div class="search-row">
			<input
				bind:this={inputEl}
				bind:value={searchQuery}
				type="text"
				placeholder="Search branches..."
				class="search-input"
			/>
		</div>

		{#if $hasChanges}
			<div class="warning-row">
				<label class="stash-option">
					<input type="checkbox" bind:checked={stashFirst} />
					<span>Stash uncommitted changes before switching</span>
				</label>
			</div>
		{/if}

		<div class="branch-list">
			{#if filteredLocal.length > 0}
				<div class="section-header">Local Branches</div>
				{#each filteredLocal as branch (branch.name)}
					<button 
						class="branch-item" 
						class:current={branch.isCurrent}
						onclick={() => switchToBranch(branch.name)}
						disabled={branch.isCurrent || switching}
					>
						<span class="branch-indicator">{branch.isCurrent ? '●' : '○'}</span>
						<span class="branch-name">{branch.name}</span>
						{#if branch.ahead > 0 || branch.behind > 0}
							<span class="branch-sync">
								{#if branch.ahead > 0}<span class="ahead">↑{branch.ahead}</span>{/if}
								{#if branch.behind > 0}<span class="behind">↓{branch.behind}</span>{/if}
							</span>
						{/if}
						{#if branch.isCurrent}
							<span class="current-badge">current</span>
						{/if}
					</button>
				{/each}
			{/if}

			{#if filteredRemote.length > 0}
				<div class="section-header">Remote Branches</div>
				{#each filteredRemote as branch (branch.name)}
					<button 
						class="branch-item" 
						onclick={() => switchToBranch(branch.name)}
						disabled={switching}
					>
						<span class="branch-indicator">○</span>
						<span class="branch-name">{branch.name}</span>
					</button>
				{/each}
			{/if}

			{#if filteredLocal.length === 0 && filteredRemote.length === 0}
				<div class="empty-state">No branches found</div>
			{/if}
		</div>

		<div class="create-section">
			<div class="section-header">Create New Branch</div>
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
					{creating ? '...' : 'Create'}
				</button>
			</div>
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
		width: 420px;
		max-height: 70vh;
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

	.search-row {
		padding: 12px 20px;
		border-bottom: 1px solid var(--color-border-muted);
	}

	.search-input {
		width: 100%;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		padding: 8px 12px;
		color: var(--color-text);
		font-size: 13px;
		outline: none;
	}

	.search-input:focus {
		border-color: var(--color-accent);
	}

	.search-input::placeholder {
		color: var(--color-text-subtle);
	}

	.warning-row {
		padding: 10px 20px;
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
		border-bottom: 1px solid var(--color-border-muted);
	}

	.stash-option {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--color-warning);
		font-size: 12px;
		cursor: pointer;
	}

	.stash-option input {
		accent-color: var(--color-warning);
	}

	.branch-list {
		flex: 1;
		overflow-y: auto;
		padding: 8px 0;
	}

	.section-header {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		padding: 8px 20px 4px;
	}

	.branch-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 20px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 13px;
		cursor: pointer;
		text-align: left;
	}

	.branch-item:hover:not(:disabled) {
		background: var(--color-overlay);
	}

	.branch-item:disabled {
		cursor: default;
	}

	.branch-item.current {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.branch-indicator {
		color: var(--color-text-subtle);
		font-size: 10px;
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
		gap: 6px;
		font-size: 11px;
		font-weight: 500;
	}

	.ahead {
		color: var(--color-success);
	}

	.behind {
		color: var(--color-warning);
	}

	.current-badge {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		padding: 2px 6px;
		border-radius: 4px;
	}

	.empty-state {
		padding: 20px;
		text-align: center;
		color: var(--color-text-subtle);
		font-size: 13px;
	}

	.create-section {
		padding: 12px 20px 16px;
		border-top: 1px solid var(--color-border-muted);
	}

	.create-row {
		display: flex;
		gap: 8px;
	}

	.create-input {
		flex: 1;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		padding: 8px 12px;
		color: var(--color-text);
		font-size: 13px;
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
		font-size: 12px;
		font-weight: 600;
		padding: 8px 16px;
		cursor: pointer;
	}

	.create-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.create-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
