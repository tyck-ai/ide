<script lang="ts">
	import { openFiles, activeFilePath, closeFile } from '$lib/stores/editor';
	import { agentStatus, agentStatusConnected } from '$lib/stores/agentStatus';
	import { activeSessionId } from '$lib/stores/agentTerminal';
	import { sessionReview, activeReview, hasActiveReview } from '$lib/stores/sessionReview';
	import { settings } from '$lib/stores/settings';

	function switchTo(path: string) {
		activeFilePath.set(path);
	}

	function formatCost(usd: number): string {
		if (usd === 0) return '$0.00';
		if (usd < 0.01) return `$${usd.toFixed(4)}`;
		return `$${usd.toFixed(2)}`;
	}

	function formatTokens(n: number): string {
		if (n === 0) return '0';
		if (n < 1000) return `${n}`;
		if (n < 1000000) return `${(n / 1000).toFixed(1)}K`;
		return `${(n / 1000000).toFixed(1)}M`;
	}

	function contextBarColor(pct: number): string {
		if (pct < 50) return 'var(--color-success)';
		if (pct < 75) return 'var(--color-warning)';
		if (pct < 90) return 'var(--color-warning)';
		return 'var(--color-error)';
	}
</script>

<div class="awareness-bar">
	<!-- Left: View tabs -->
	<div class="section-left">
		<div class="view-toggle" role="group">
			{#if $settings.reviewEnabled && $hasActiveReview}
				{@const review = $activeReview}
				<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
				<button
					class="view-toggle-btn"
					class:active={review?.reviewMode}
					class:has-changes={(review?.diffs.length ?? 0) > 0 && !review?.reviewMode}
					onclick={async () => { if (!review?.reviewMode && $activeSessionId) await sessionReview.enterReviewMode($activeSessionId); }}
				>Review{#if (review?.diffs.length ?? 0) > 0}<span class="toggle-count">{review?.diffs.length}</span>{/if}</button>
			{/if}
			<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
			<button
				class="view-toggle-btn"
				class:active={!$settings.reviewEnabled || !$hasActiveReview || !$activeReview?.reviewMode}
				onclick={() => { if ($activeReview?.reviewMode && $activeSessionId) sessionReview.exitReviewMode($activeSessionId); }}
			>Editor</button>
		</div>
	</div>

	<!-- Center: File tabs -->
	<div class="tabs">
		{#each $openFiles as file (file.path)}
			<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
			<div
				class="tab"
				class:active={file.path === $activeFilePath}
				onclick={() => switchTo(file.path)}
				role="tab"
				tabindex="0"
			>
				<span class="tab-name">{file.name}</span>
				{#if file.modified}
					<span class="modified-dot"></span>
				{/if}
				<button
					class="tab-close"
					onclick={(e) => { e.stopPropagation(); closeFile(file.path); }}
				>&times;</button>
			</div>
		{/each}
	</div>

	<div class="spacer"></div>

	<!-- Right: AI Status + Context + Cost -->
	<div class="section-right">
		<div class="ai-status" title={
			$agentStatusConnected
				? `${$agentStatus.modelId}\n${formatTokens($agentStatus.totalInputTokens + $agentStatus.totalOutputTokens)} tokens used`
				: $activeSessionId ? 'Agent running — waiting for first response' : 'No agent'
		}>
			<span class="ai-dot" class:active={!!$activeSessionId} class:connected={$agentStatusConnected}></span>
			<span class="ai-model">
				{#if $agentStatusConnected && $agentStatus.modelName}
					{$agentStatus.modelName}
				{:else if $activeSessionId}
					AI
				{:else}
					--
				{/if}
			</span>
		</div>

		<div class="context-meter" title={
			$agentStatusConnected
				? `Context: ${$agentStatus.contextUsedPercent.toFixed(1)}% used (${formatTokens($agentStatus.totalInputTokens + $agentStatus.totalOutputTokens)} / ${formatTokens($agentStatus.contextWindowSize)})`
				: 'Context window'
		}>
			<span class="meter-icon">ctx</span>
			<div class="meter-bar">
				<div
					class="meter-fill"
					style="width: {$agentStatusConnected ? Math.min($agentStatus.contextUsedPercent, 100) : 0}%; background: {contextBarColor($agentStatus.contextUsedPercent)}"
				></div>
			</div>
			<span class="meter-label">
				{$agentStatusConnected ? `${$agentStatus.contextUsedPercent.toFixed(0)}%` : '--'}
			</span>
		</div>

		<div class="cost-badge" title={
			$agentStatusConnected
				? `Session cost: ${formatCost($agentStatus.totalCostUsd)}`
				: 'Session cost'
		}>
			{$agentStatusConnected ? formatCost($agentStatus.totalCostUsd) : '$--'}
		</div>

		{#if $agentStatusConnected && ($agentStatus.linesAdded > 0 || $agentStatus.linesRemoved > 0)}
			<div class="lines-badge" title="{$agentStatus.linesAdded} added, {$agentStatus.linesRemoved} removed">
				<span class="lines-add">+{$agentStatus.linesAdded}</span>
				<span class="lines-del">-{$agentStatus.linesRemoved}</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.awareness-bar {
		display: flex;
		align-items: center;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
		padding: 0 10px;
		gap: 0;
		user-select: none;
		-webkit-app-region: drag;
		height: 100%;
		font-size: 11px;
	}
	.section-left {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
		padding-right: 8px;
		-webkit-app-region: no-drag;
	}
	.tabs {
		display: flex;
		gap: 1px;
		overflow-x: auto;
		-webkit-app-region: no-drag;
	}
	.tab {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 4px 10px;
		background: transparent;
		border: none;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		white-space: nowrap;
		transition: color 0.1s;
	}
	.tab:hover {
		color: var(--color-text-muted);
		background: var(--color-base);
	}
	.tab.active {
		color: var(--color-text);
		background: var(--color-base);
		border-bottom: 2px solid var(--color-accent);
	}
	.modified-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--color-warning);
	}
	.tab-close {
		background: none;
		border: none;
		color: inherit;
		font-size: 12px;
		padding: 0 1px;
		cursor: pointer;
		opacity: 0;
		transition: opacity 0.1s;
	}
	.tab:hover .tab-close { opacity: 0.5; }
	.tab-close:hover { opacity: 1 !important; }
	.spacer {
		flex: 1;
		-webkit-app-region: drag;
	}
	.section-right {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-shrink: 0;
		-webkit-app-region: no-drag;
	}
	.ai-status {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		cursor: default;
	}
	.ai-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-border);
		transition: background 0.3s;
	}
	.ai-dot.active {
		background: var(--color-warning);
		animation: pulse 2s infinite;
	}
	.ai-dot.connected {
		background: var(--color-success);
		animation: none;
	}
	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.4; }
	}
	.ai-model {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-text-muted);
	}
	.context-meter {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		cursor: default;
	}
	.meter-icon {
		font-size: 9px;
		font-weight: 700;
		color: var(--color-text-subtle);
		text-transform: uppercase;
	}
	.meter-bar {
		width: 48px;
		height: 6px;
		background: var(--color-border-muted);
		border-radius: 3px;
		overflow: hidden;
	}
	.meter-fill {
		height: 100%;
		border-radius: 3px;
		transition: width 0.5s ease, background 0.5s ease;
	}
	.meter-label {
		font-size: 9px;
		font-weight: 700;
		color: var(--color-text-subtle);
		min-width: 22px;
		text-align: right;
	}
	.cost-badge {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-success);
		padding: 2px 6px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		cursor: default;
	}
	.lines-badge {
		display: flex;
		gap: 3px;
		font-size: 9px;
		font-weight: 700;
		padding: 2px 6px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		cursor: default;
	}
	.lines-add { color: var(--color-success); }
	.lines-del { color: var(--color-error); }
	.view-toggle {
		display: flex;
		border: 1px solid var(--color-border-muted);
		border-radius: 5px;
		overflow: hidden;
		-webkit-app-region: no-drag;
	}
	.view-toggle-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 10px;
		background: none;
		border: none;
		font-size: 10px;
		font-weight: 600;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s;
	}
	.view-toggle-btn:hover {
		color: var(--color-text-muted);
		background: var(--color-base);
	}
	.view-toggle-btn.active {
		color: var(--color-text);
		background: var(--color-overlay);
	}
	.view-toggle-btn.has-changes {
		color: var(--color-warning);
	}
	.view-toggle-btn.has-changes:hover {
		color: var(--color-warning);
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
	}
	.toggle-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--color-warning);
		color: var(--color-base);
		font-size: 8px;
		font-weight: 800;
		min-width: 14px;
		height: 13px;
		border-radius: 7px;
		padding: 0 3px;
	}
	.view-toggle-btn.active .toggle-count {
		background: var(--color-accent);
	}
</style>
