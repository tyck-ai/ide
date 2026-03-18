<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { openFiles, activeFilePath, closeFile, markFileSaved, visibleFiles } from '$lib/stores/editor';
	import { agentStatus, agentStatusConnected } from '$lib/stores/agentStatus';
	import { activeSessionId } from '$lib/stores/agentTerminal';
	import { isDevMode } from '$lib/stores/settings';
	import { pendingEditCount } from '$lib/stores/devModeEdits';
	import { toast } from '$lib/stores/toast';

	let pendingClosePath = $state<string | null>(null);

	const pendingCloseFile = $derived(
		pendingClosePath ? $openFiles.find(f => f.path === pendingClosePath) ?? null : null
	);

	function switchTo(path: string) {
		activeFilePath.set(path);
	}

	function requestClose(path: string, modified: boolean) {
		if (modified) {
			pendingClosePath = path;
		} else {
			closeFile(path);
		}
	}

	function discard() {
		if (pendingClosePath) closeFile(pendingClosePath);
		pendingClosePath = null;
	}

	async function saveAndClose() {
		const file = pendingCloseFile;
		if (!file) return;
		try {
			await invoke('write_file', { path: file.path, content: file.content });
			markFileSaved(file.path);
			closeFile(file.path);
		} catch (e) {
			toast.error(`Failed to save ${file.name}: ${e}`);
		}
		pendingClosePath = null;
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
	<!-- Left: Pending edits badge (dev mode only) -->
	<div class="section-left">
		{#if $isDevMode && $pendingEditCount > 0}
			<span class="pending-badge">{$pendingEditCount} pending edit{$pendingEditCount > 1 ? 's' : ''}</span>
		{/if}
	</div>

	<!-- Center: File tabs -->
	<div class="tabs">
		{#each ($isDevMode ? $openFiles : $visibleFiles) as file (file.path)}
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
					onclick={(e) => { e.stopPropagation(); requestClose(file.path, file.modified); }}
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

{#if pendingClosePath && pendingCloseFile}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="close-backdrop" onclick={() => pendingClosePath = null}>
		<div class="close-modal" onclick={(e) => e.stopPropagation()}>
			<div class="close-title">Unsaved changes</div>
			<div class="close-text">
				<strong>{pendingCloseFile.name}</strong> has unsaved changes. What would you like to do?
			</div>
			<div class="close-actions">
				<button class="close-btn discard" onclick={discard}>Discard</button>
				<button class="close-btn save" onclick={saveAndClose}>Save</button>
			</div>
		</div>
	</div>
{/if}

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
	.pending-badge {
		font-size: 10px;
		color: var(--color-accent);
		font-weight: 600;
		padding: 2px 8px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		border-radius: 10px;
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

	.close-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}
	.close-modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 10px;
		padding: 24px;
		max-width: 360px;
		width: 90%;
	}
	.close-title {
		font-size: 15px;
		font-weight: 600;
		margin-bottom: 10px;
	}
	.close-text {
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.5;
		margin-bottom: 20px;
	}
	.close-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}
	.close-btn {
		padding: 7px 18px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		border: none;
	}
	.close-btn.discard {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		color: var(--color-text);
	}
	.close-btn.discard:hover {
		background: color-mix(in srgb, var(--color-error) 10%, transparent);
		border-color: var(--color-error);
		color: var(--color-error);
	}
	.close-btn.save {
		background: var(--color-accent);
		color: var(--color-base);
	}
	.close-btn.save:hover {
		filter: brightness(1.1);
	}
</style>
