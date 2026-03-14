<script lang="ts">
	import AgentTerminal from './AgentTerminal.svelte';
	import { agentProviders, activeProviderId, activeProvider } from '$lib/stores/agentProvider';
	import {
		agentSessions,
		activeSessionId,
		spawnAgentSession,
		switchAgentSession,
		closeAgentSession,
		resumeAgentSession,
	} from '$lib/stores/agentTerminal';
	import { projectRoot } from '$lib/stores/editor';
	import { invoke } from '@tauri-apps/api/core';

	interface SessionInfo {
		sessionId: string;
		firstMessage: string;
		timestamp: string;
		model: string;
		isActive: boolean;
		provider: string;
		providerName: string;
		slug: string;
		sessionPath: string;
	}

	let showSessionList = $state(false);
	let sessions = $state<SessionInfo[]>([]);
	let reconciling = $state(false);
	let showNewMenu = $state(false);

	async function newSessionWith(providerId: string) {
		showNewMenu = false;
		activeProviderId.set(providerId);
		await spawnAgentSession(undefined, providerId);
	}

	async function toggleSessionList() {
		if (showSessionList) {
			showSessionList = false;
			return;
		}
		const cwd = $projectRoot;
		if (cwd) {
			try {
				sessions = await invoke<SessionInfo[]>('list_sessions', { cwd });
			} catch (e) {
				console.warn('list_sessions failed:', e);
				sessions = [];
			}
		} else {
			sessions = [];
		}
		showSessionList = true;
	}

	async function reconcileSessions() {
		reconciling = true;
		const cwd = $projectRoot;
		if (cwd) {
			try {
				sessions = await invoke<SessionInfo[]>('reconcile_sessions', { cwd });
			} catch (e) {
				console.warn('reconcile_sessions failed:', e);
			}
		}
		reconciling = false;
	}

	async function onResumeSession(sessionId: string, provider: string, sessionPath: string) {
		showSessionList = false;
		await resumeAgentSession(sessionId, provider, sessionPath);
	}

	function formatTime(ts: string): string {
		if (!ts) return '';
		try {
			const d = new Date(ts);
			const now = new Date();
			const diff = now.getTime() - d.getTime();
			const mins = Math.floor(diff / 60000);
			if (mins < 60) return `${mins}m ago`;
			const hours = Math.floor(mins / 60);
			if (hours < 24) return `${hours}h ago`;
			const days = Math.floor(hours / 24);
			return `${days}d ago`;
		} catch {
			return '';
		}
	}

	function truncate(s: string, max: number): string {
		return s.length > max ? s.slice(0, max) + '...' : s;
	}

	// Spawn the first session when projectRoot is ready and no sessions exist
	$effect(() => {
		if ($agentSessions.length === 0 && $projectRoot) {
			spawnAgentSession();
		}
	});
</script>

<div class="insight-zone">
	<div class="header">
		<div class="header-actions">
			<button
				class="icon-btn"
				class:active={showSessionList}
				onclick={toggleSessionList}
				title="Past sessions"
			>&#9776;</button>
			<div class="new-session-wrapper">
				<button
					class="icon-btn"
					class:active={showNewMenu}
					onclick={() => showNewMenu = !showNewMenu}
					title="New session"
				>&#43;</button>
				{#if showNewMenu}
					<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
					<div class="new-menu-backdrop" onclick={() => showNewMenu = false}></div>
					<div class="new-menu">
						{#each $agentProviders as provider (provider.id)}
							<button class="new-menu-item" onclick={() => newSessionWith(provider.id)}>
								{provider.displayName}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- Session tabs -->
	{#if $agentSessions.length > 1}
		<div class="tab-bar">
			{#each $agentSessions as session (session.id)}
				<button
					class="session-tab"
					class:active={session.id === $activeSessionId}
					onclick={() => switchAgentSession(session.id)}
					title={session.label}
				>
					<span class="tab-label">{session.label}</span>
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<span
						class="tab-close"
						role="button"
						tabindex="0"
						onclick={(e) => { e.stopPropagation(); closeAgentSession(session.id); }}
					>&times;</span>
				</button>
			{/each}
		</div>
	{/if}

	<div class="terminal-container">
		{#if $agentSessions.length === 0}
			<div class="empty">
				<p>Starting agent...</p>
			</div>
		{/if}

		{#each $agentSessions as session (session.id)}
			<div class="terminal-pane" class:active={session.id === $activeSessionId}>
				<AgentTerminal sessionId={session.id} />
			</div>
		{/each}

		<!-- Session list overlay -->
		{#if showSessionList}
			<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
			<div class="session-backdrop" onclick={() => showSessionList = false}>
				<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
				<div class="session-modal" onclick={(e) => e.stopPropagation()}>
					<div class="session-header">
						<span class="session-title">Past Sessions</span>
						<div class="session-header-actions">
							<button
								class="session-reconcile"
								class:spinning={reconciling}
								onclick={reconcileSessions}
								disabled={reconciling}
								title="Reconcile — remove stale sessions"
							>&#x21bb;</button>
							<button class="session-close" onclick={() => showSessionList = false}>&times;</button>
						</div>
					</div>
					<div class="session-body">
						{#if sessions.length === 0}
							<div class="session-empty">No past sessions</div>
						{:else}
							{#each sessions as s (s.sessionId)}
								<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
								<div class="session-item" role="button" tabindex="0" onclick={() => onResumeSession(s.sessionId, s.provider, s.sessionPath)}>
									<div class="session-msg">{truncate(s.firstMessage, 80)}</div>
									<div class="session-meta">
										<span class="session-provider">{s.providerName}</span>
										<span class="session-time">{formatTime(s.timestamp)}</span>
									</div>
								</div>
							{/each}
						{/if}
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.insight-zone {
		display: flex;
		flex-direction: column;
		background: var(--color-base);
		height: 100%;
		overflow: hidden;
	}
	.header {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		padding: 6px 12px;
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
	}
	.header-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.new-session-wrapper {
		position: relative;
	}
	.new-menu-backdrop {
		position: fixed;
		inset: 0;
		z-index: 29;
	}
	.new-menu {
		position: absolute;
		top: 100%;
		right: 0;
		margin-top: 4px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
		z-index: 30;
		min-width: 140px;
		overflow: hidden;
	}
	.new-menu-item {
		display: block;
		width: 100%;
		padding: 8px 12px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 12px;
		text-align: left;
		cursor: pointer;
		white-space: nowrap;
	}
	.new-menu-item:hover {
		background: var(--color-overlay);
	}
	.icon-btn {
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		color: var(--color-text-subtle);
		font-size: 13px;
		padding: 1px 6px;
		cursor: pointer;
		line-height: 1;
	}
	.icon-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
	}
	.icon-btn.active {
		color: var(--color-accent);
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}
	/* Session tabs */
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
		font-size: 12px;
		opacity: 0;
		transition: opacity 0.1s;
		cursor: pointer;
	}
	.session-tab:hover .tab-close {
		opacity: 0.5;
	}
	.tab-close:hover {
		opacity: 1 !important;
		color: var(--color-error);
	}

	/* Terminal panes — all rendered, only active visible */
	.terminal-pane {
		position: absolute;
		inset: 0;
		display: none;
		z-index: 1;
	}
	.terminal-pane.active {
		display: block;
	}

	.terminal-container {
		flex: 1;
		overflow: hidden;
		position: relative;
	}

	/* Session overlay modal */
	.session-backdrop {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		z-index: 20;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 24px;
	}
	.session-modal {
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		width: 90%;
		max-width: 360px;
		max-height: 70%;
		display: flex;
		flex-direction: column;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
		overflow: hidden;
	}
	.session-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 12px;
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
	}
	.session-title {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
	}
	.session-header-actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.session-reconcile {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 15px;
		cursor: pointer;
		padding: 0 4px;
		line-height: 1;
		transition: color 0.15s;
	}
	.session-reconcile:hover {
		color: var(--color-text);
	}
	.session-reconcile:disabled {
		cursor: default;
	}
	.session-reconcile.spinning {
		animation: spin 0.8s linear infinite;
	}
	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
	.session-close {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 18px;
		cursor: pointer;
		line-height: 1;
		padding: 0 4px;
	}
	.session-close:hover {
		color: var(--color-text);
	}
	.session-body {
		overflow-y: auto;
		flex: 1;
	}
	.session-empty {
		padding: 12px;
		color: var(--color-text-subtle);
		font-size: 12px;
		text-align: center;
	}
	.session-item {
		padding: 8px 12px;
		cursor: pointer;
		border-bottom: 1px solid var(--color-surface);
		transition: background 0.1s;
	}
	.session-item:hover {
		background: var(--color-overlay);
	}
	.session-msg {
		font-size: 12px;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.session-meta {
		display: flex;
		gap: 8px;
		margin-top: 2px;
	}
	.session-provider {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-accent);
	}
	.session-time {
		font-size: 10px;
		color: var(--color-text-subtle);
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--color-text-subtle);
		text-align: center;
		gap: 4px;
	}
	.empty p {
		font-size: 13px;
		margin: 0;
	}
</style>
