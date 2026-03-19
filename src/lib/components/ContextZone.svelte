<script lang="ts">
	import { isAgentMode } from '$lib/stores/settings';
	import { activeSessionId } from '$lib/stores/agentTerminal';
	import { activeReview } from '$lib/stores/sessionReview';
	import { contextTab } from '$lib/stores/sessionContext';
	import ExplorerPanel from './ExplorerPanel.svelte';
	import ReviewPanel from './ReviewPanel.svelte';
	import ReviewFileList from './ReviewFileList.svelte';

	const reviewCount = $derived($activeReview?.diffs.length ?? 0);
</script>

<div class="context-zone">
	{#if $isAgentMode && $activeSessionId}
		<!-- Agent mode with active session: Review / Editor tabs -->
		<div class="context-tabs">
			<button class="context-tab" class:active={$contextTab === 'review'} onclick={() => contextTab.set('review')}>
				Review{#if reviewCount > 0}<span class="tab-badge">{reviewCount}</span>{/if}
			</button>
			<button class="context-tab" class:active={$contextTab === 'editor'} onclick={() => contextTab.set('editor')}>
				Editor
			</button>
		</div>
		{#if $contextTab === 'review'}
			<ReviewFileList />
		{:else}
			<ExplorerPanel />
		{/if}
	{:else if $isAgentMode && $activeReview?.reviewMode}
		<ReviewPanel />
	{:else if !$isAgentMode}
		<ExplorerPanel />
	{/if}
</div>

<style>
	.context-zone {
		display: flex;
		flex-direction: column;
		background: var(--color-base);
		height: 100%;
		overflow: hidden;
	}
	.context-tabs {
		display: flex;
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
	}
	.context-tab {
		flex: 1;
		padding: 8px 12px;
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
	}
	.context-tab:hover { color: var(--color-text); }
	.context-tab.active {
		color: var(--color-text);
		border-bottom-color: var(--color-accent);
	}
	.tab-badge {
		background: var(--color-accent);
		color: white;
		font-size: 9px;
		padding: 1px 5px;
		border-radius: 8px;
		font-weight: 700;
	}
</style>
