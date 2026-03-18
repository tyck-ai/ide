<script lang="ts">
	import AgentTerminal from './AgentTerminal.svelte';
	import NewDevSessionModal from './NewDevSessionModal.svelte';
	import { agentProviders } from '$lib/stores/agentProvider';
	import {
		devSessions,
		agentModeSessions,
		activeSessionId,
		closeAgentSession,
	} from '$lib/stores/agentTerminal';
	import { isAgentMode } from '$lib/stores/settings';

	let showDevModal = $state(false);

	// In dev mode: track the active dev session separately
	// Falls back to first dev session if activeSessionId is an agent-mode session
	const activeDevSessionId = $derived(
		$devSessions.find(s => s.id === $activeSessionId)?.id ?? $devSessions[0]?.id ?? null
	);

	async function closeDevSession(e: MouseEvent, id: string) {
		e.stopPropagation();
		await closeAgentSession(id);
	}
</script>

<div class="insight-zone">
	{#if !$isAgentMode}
		<!-- Dev mode -->
		{#if $devSessions.length === 0}
			<!-- No sessions yet: start panel -->
			<div class="dev-start">
				<div class="dev-start-icon">
					<svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
						<path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2z"/>
						<path d="M12 8v8M8 12h8"/>
					</svg>
				</div>
				<p class="dev-start-label">Start an agent</p>
				{#if $agentProviders.length > 0}
					<button class="dev-start-btn" onclick={() => showDevModal = true}>
						+ Start agent
					</button>
				{:else}
					<p class="dev-start-hint">No AI provider installed.<br/>Install an AI agent to get started.</p>
				{/if}
			</div>
		{:else}
			<!-- Sessions exist: tab bar + terminals -->
			<div class="tab-bar">
				{#each $devSessions as session (session.id)}
					<button
						class="session-tab"
						class:active={session.id === activeDevSessionId}
						onclick={() => activeSessionId.set(session.id)}
					>
						<span class="tab-label">{session.label}</span>
						<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
						<span class="tab-close" onclick={(e) => closeDevSession(e, session.id)}>×</span>
					</button>
				{/each}
				{#if $agentProviders.length > 0}
					<button class="tab-add" onclick={() => showDevModal = true} title="New agent session">+</button>
				{/if}
			</div>
			<div class="terminal-container">
				{#each $devSessions as session (session.id)}
					<div class="terminal-pane" class:active={session.id === activeDevSessionId}>
						<AgentTerminal sessionId={session.id} />
					</div>
				{/each}
			</div>
		{/if}
	{:else}
		<!-- Agent mode: show agent-mode sessions only (welcome screen handled by +page.svelte) -->
		<div class="terminal-container">
			{#each $agentModeSessions as session (session.id)}
				<div class="terminal-pane" class:active={session.id === $activeSessionId}>
					<AgentTerminal sessionId={session.id} />
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if showDevModal}
	<NewDevSessionModal onClose={() => showDevModal = false} />
{/if}

<style>
	.insight-zone {
		display: flex;
		flex-direction: column;
		background: var(--color-base);
		height: 100%;
		overflow: hidden;
	}

	/* Dev mode start panel */
	.dev-start {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		height: 100%;
		padding: 24px;
		text-align: center;
	}
	.dev-start-icon {
		color: var(--color-text-subtle);
		opacity: 0.4;
	}
	.dev-start-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-secondary);
		margin: 0;
	}
	.dev-start-btn {
		padding: 7px 16px;
		background: var(--color-accent);
		border: none;
		border-radius: 6px;
		color: white;
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
	}
	.dev-start-btn:hover { filter: brightness(1.1); }
	.dev-start-hint {
		font-size: 11px;
		color: var(--color-text-subtle);
		line-height: 1.5;
		margin: 0;
	}

	/* Tab bar for dev mode multi-session */
	.tab-bar {
		display: flex;
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
		overflow-x: auto;
		background: var(--color-surface);
	}
	.session-tab {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		white-space: nowrap;
		transition: color 0.1s, border-color 0.1s;
	}
	.session-tab:hover {
		color: var(--color-text-muted);
		background: var(--color-base);
	}
	.session-tab.active {
		color: var(--color-text);
		border-bottom-color: var(--color-accent);
		background: var(--color-base);
	}
	.tab-label {
		max-width: 120px;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.tab-close {
		font-size: 14px;
		opacity: 0;
		transition: opacity 0.1s;
		cursor: pointer;
		line-height: 1;
	}
	.session-tab:hover .tab-close { opacity: 0.5; }
	.tab-close:hover { opacity: 1 !important; color: var(--color-error); }
	.tab-add {
		padding: 4px 10px;
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 16px;
		cursor: pointer;
		line-height: 1;
		flex-shrink: 0;
	}
	.tab-add:hover { color: var(--color-text); }
	.tab-add:disabled { opacity: 0.4; cursor: default; }

	/* Terminal container */
	.terminal-container {
		flex: 1;
		overflow: hidden;
		position: relative;
	}
	.terminal-pane {
		position: absolute;
		inset: 0;
		display: none;
		z-index: 1;
	}
	.terminal-pane.active {
		display: block;
	}
</style>
