<script lang="ts">
	import { onMount } from 'svelte';
	import { showGitView, gitViewTab } from '$lib/stores/layout';
	import { git } from '$lib/stores/git';
	import ChangesTab from './git/ChangesTab.svelte';
	import HistoryTab from './git/HistoryTab.svelte';
	import BranchesTab from './git/BranchesTab.svelte';
	import StashesTab from './git/StashesTab.svelte';

	const tabs = [
		{ id: 'changes', label: 'Changes' },
		{ id: 'history', label: 'History' },
		{ id: 'branches', label: 'Branches' },
		{ id: 'stashes', label: 'Stashes' },
	] as const;

	onMount(() => {
		git.refresh();
		git.refreshBranches();
		git.refreshCommits();
		git.refreshStashes();
	});

	function close() {
		showGitView.set(false);
	}
</script>

<div class="git-view">
	<nav class="sidebar">
		<div class="sidebar-header">
			<span class="sidebar-title">Git</span>
		</div>
		<div class="sidebar-nav">
			{#each tabs as tab (tab.id)}
				<button
					class="nav-item"
					class:active={$gitViewTab === tab.id}
					onclick={() => gitViewTab.set(tab.id)}
				>{tab.label}</button>
			{/each}
		</div>
		<div class="sidebar-footer">
			<button class="back-btn" onclick={close}>
				Back to Code
			</button>
		</div>
	</nav>

	<main class="content">
		{#if $gitViewTab === 'changes'}
			<ChangesTab />
		{:else if $gitViewTab === 'history'}
			<HistoryTab />
		{:else if $gitViewTab === 'branches'}
			<BranchesTab />
		{:else if $gitViewTab === 'stashes'}
			<StashesTab />
		{/if}
	</main>
</div>

<style>
	.git-view {
		display: flex;
		height: 100%;
		background: var(--color-base);
		overflow: hidden;
	}

	.sidebar {
		width: 180px;
		flex-shrink: 0;
		background: var(--color-surface);
		border-right: 1px solid var(--color-border-muted);
		display: flex;
		flex-direction: column;
	}

	.sidebar-header {
		padding: 16px 16px 12px;
	}

	.sidebar-title {
		font-size: 11px;
		font-weight: 700;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.8px;
	}

	.sidebar-nav {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 1px;
		padding: 0 8px;
	}

	.nav-item {
		display: block;
		width: 100%;
		padding: 8px 12px;
		background: none;
		border: none;
		border-radius: 6px;
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		text-align: left;
		cursor: pointer;
	}

	.nav-item:hover {
		background: var(--color-base);
		color: var(--color-text);
	}

	.nav-item.active {
		background: var(--color-overlay);
		color: var(--color-text);
		font-weight: 600;
	}

	.sidebar-footer {
		padding: 12px;
		border-top: 1px solid var(--color-border-muted);
	}

	.back-btn {
		display: block;
		width: 100%;
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 600;
		padding: 6px 0;
		cursor: pointer;
		text-align: center;
	}

	.back-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.content {
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}
</style>
