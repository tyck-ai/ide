<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import Terminal from './Terminal.svelte';
	import {
		terminalSessions,
		activeTerminalId,
		terminalVisible,
		addTerminal,
		removeTerminal,
	} from '$lib/stores/terminal';
	import { projectRoot } from '$lib/stores/editor';

	let panelHeight = $state(250);
	let dragging = $state(false);

	function newTerminal() {
		const id = crypto.randomUUID();
		addTerminal(id, 'zsh');
	}

	function closeTerminal(id: string) {
		invoke('kill_terminal', { id });
		removeTerminal(id);
	}

	function onDragStart(e: MouseEvent) {
		dragging = true;
		const startY = e.clientY;
		const startHeight = panelHeight;

		function onMove(e: MouseEvent) {
			panelHeight = Math.max(100, Math.min(600, startHeight + (startY - e.clientY)));
		}
		function onUp() {
			dragging = false;
			window.removeEventListener('mousemove', onMove);
			window.removeEventListener('mouseup', onUp);
		}
		window.addEventListener('mousemove', onMove);
		window.addEventListener('mouseup', onUp);
	}

	// Auto-create first terminal if none exist when panel becomes visible
	$effect(() => {
		if ($terminalVisible && $terminalSessions.length === 0) {
			newTerminal();
		}
	});
</script>

{#if $terminalVisible}
	<div class="terminal-panel" style="height: {panelHeight}px" class:dragging>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="drag-handle-h" onmousedown={onDragStart}>
			<div class="handle-bar"></div>
		</div>

		<div class="tabs-row">
			<div class="tabs">
				{#each $terminalSessions as session (session.id)}
					<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
					<div
						class="tab"
						class:active={session.id === $activeTerminalId}
						onclick={() => activeTerminalId.set(session.id)}
						role="tab"
						tabindex="0"
					>
						<span class="tab-icon">&#9002;</span>
						<span class="tab-name">{session.title}</span>
						<button
							class="tab-close"
							onclick={(e) => { e.stopPropagation(); closeTerminal(session.id); }}
						>&times;</button>
					</div>
				{/each}
			</div>
			<button class="new-btn" onclick={newTerminal}>+</button>
		</div>

		<div class="terminal-content">
			{#each $terminalSessions as session (session.id)}
				<div class="terminal-tab-content" class:active={session.id === $activeTerminalId}>
					<Terminal sessionId={session.id} />
				</div>
			{/each}
		</div>
	</div>
{/if}

<style>
	.terminal-panel {
		display: flex;
		flex-direction: column;
		background: var(--color-base);
		border-top: 1px solid var(--color-border-muted);
		overflow: hidden;
	}
	.terminal-panel.dragging {
		user-select: none;
	}
	.drag-handle-h {
		height: 4px;
		cursor: row-resize;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}
	.drag-handle-h:hover {
		background: color-mix(in srgb, var(--color-accent) 25%, transparent);
	}
	.handle-bar {
		width: 40px;
		height: 2px;
		background: var(--color-border);
		border-radius: 1px;
	}
	.tabs-row {
		display: flex;
		align-items: center;
		padding: 0 8px;
		gap: 4px;
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
	}
	.tabs {
		display: flex;
		gap: 1px;
		overflow-x: auto;
	}
	.tab {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		background: var(--color-surface);
		border: none;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		white-space: nowrap;
	}
	.tab:hover {
		background: var(--color-overlay);
		color: var(--color-text);
	}
	.tab.active {
		background: var(--color-base);
		color: var(--color-text);
		border-bottom: 2px solid var(--color-accent);
	}
	.tab-icon {
		font-size: 10px;
		color: var(--color-success);
	}
	.tab-close {
		background: none;
		border: none;
		color: inherit;
		font-size: 12px;
		padding: 0 2px;
		cursor: pointer;
		opacity: 0.5;
	}
	.tab-close:hover {
		opacity: 1;
	}
	.new-btn {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 16px;
		cursor: pointer;
		padding: 2px 8px;
	}
	.new-btn:hover {
		color: var(--color-text);
	}
	.terminal-content {
		flex: 1;
		overflow: hidden;
		position: relative;
	}
	.terminal-tab-content {
		position: absolute;
		inset: 0;
		display: none;
	}
	.terminal-tab-content.active {
		display: block;
	}
</style>
