<script lang="ts">
	import {
		agentModeSessions,
		switchAgentSession,
		closeAgentSession,
		pauseAgent,
		resumeAgent,
		type SessionStatus,
	} from '$lib/stores/agentTerminal';
	import { activeSessionId } from '$lib/stores/activeSession';
	import { allSessionStatuses } from '$lib/stores/agentStatus';
	import { agentProviders } from '$lib/stores/agentProvider';
	import NewSessionModal from './NewSessionModal.svelte';

	interface Props {
		onClose: () => void;
	}

	let { onClose }: Props = $props();

	let showNewModal = $state(false);
	let contextMenuId = $state<string | null>(null);
	let contextMenuPos = $state({ x: 0, y: 0 });

	function formatProvider(id: string): string {
		return $agentProviders.find(p => p.id === id)?.displayName ?? id;
	}

	function formatElapsed(createdAt: number): string {
		const mins = Math.floor((Date.now() - createdAt) / 60000);
		if (mins < 1) return '<1m';
		if (mins < 60) return `${mins}m`;
		return `${Math.floor(mins / 60)}h ${mins % 60}m`;
	}

	function statusDot(status: SessionStatus): string {
		switch (status) {
			case 'working': return 'dot-working';
			case 'idle': return 'dot-idle';
			case 'done': return 'dot-done';
			case 'error': return 'dot-error';
			case 'paused': return 'dot-paused';
			default: return '';
		}
	}

	function onContextMenu(e: MouseEvent, id: string) {
		e.preventDefault();
		contextMenuId = id;
		contextMenuPos = { x: e.clientX, y: e.clientY };
	}

	const working = $derived($agentModeSessions.filter(s => s.status === 'working'));
	const queued  = $derived($agentModeSessions.filter(s => s.status === 'idle'));
	const paused  = $derived($agentModeSessions.filter(s => s.status === 'paused'));
	const done    = $derived($agentModeSessions.filter(s => s.status === 'done'));
	const errored = $derived($agentModeSessions.filter(s => s.status === 'error'));

	const totalCost = $derived(
		$agentModeSessions.reduce((sum, s) => sum + ($allSessionStatuses[s.id]?.totalCostUsd ?? 0), 0)
	);
	const totalLines = $derived(
		$agentModeSessions.reduce((sum, s) => {
			const st = $allSessionStatuses[s.id];
			return sum + (st ? st.linesAdded + st.linesRemoved : 0);
		}, 0)
	);
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
{#if contextMenuId}
	<div class="ctx-backdrop" onclick={() => contextMenuId = null}></div>
	{@const menuSession = $agentModeSessions.find(s => s.id === contextMenuId)}
	<div class="ctx-menu" style="left: {contextMenuPos.x}px; top: {contextMenuPos.y}px">
		{#if menuSession?.status === 'paused'}
			<button class="ctx-item" onclick={() => { resumeAgent(contextMenuId!); contextMenuId = null; }}>
				Resume Agent
			</button>
		{:else if menuSession?.status === 'working'}
			<button class="ctx-item" onclick={() => { pauseAgent(contextMenuId!); contextMenuId = null; }}>
				Pause Agent
			</button>
		{/if}
		<button class="ctx-item danger" onclick={() => { closeAgentSession(contextMenuId!); contextMenuId = null; }}>
			Close Session
		</button>
	</div>
{/if}

<div class="sidebar">
	<div class="sidebar-header">
		<span class="sidebar-title">Sessions</span>
		<div class="sidebar-stats">
			<span class="stat">{$agentModeSessions.length} agents</span>
			{#if totalLines > 0}
				<span class="stat-sep">·</span>
				<span class="stat">{totalLines} lines</span>
			{/if}
			{#if totalCost > 0}
				<span class="stat-sep">·</span>
				<span class="stat">${totalCost.toFixed(2)}</span>
			{/if}
		</div>
		<button class="sidebar-new" onclick={() => showNewModal = true} title="New session">
			<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M12 5v14M5 12h14"/></svg>
		</button>
		<button class="sidebar-close" onclick={onClose}>
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
		</button>
	</div>

	<div class="sidebar-body">
		{#if $agentModeSessions.length === 0}
			<div class="empty">
				<svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
					<circle cx="12" cy="12" r="10"/><path d="M8 12h4m0 0v4m0-4V8"/>
				</svg>
				<span class="empty-text">No active sessions</span>
				<button class="empty-btn" onclick={() => showNewModal = true}>New Session</button>
			</div>
		{:else}
			{#if working.length > 0}
				<div class="group">
					<div class="group-label">
						<span class="group-dot dot-working"></span>
						Working ({working.length})
					</div>
					{#each working as session (session.id)}
						{@const st = $allSessionStatuses[session.id]}
						<button
							class="session-card"
							class:active={session.id === $activeSessionId}
							onclick={() => switchAgentSession(session.id)}
							oncontextmenu={(e) => onContextMenu(e, session.id)}
						>
							<div class="card-top">
								<span class="card-name">{session.label}</span>
								<span class="card-time">{formatElapsed(session.createdAt)}</span>
							</div>
							<div class="card-branch">{session.branchName}</div>
							<div class="card-bottom">
								<span class="card-agent">{formatProvider(session.providerId)}</span>
								{#if st}
									<span class="card-meta">+{st.linesAdded}/-{st.linesRemoved} · ${st.totalCostUsd.toFixed(2)}</span>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{/if}

			{#if paused.length > 0}
				<div class="group">
					<div class="group-label">
						<span class="group-dot dot-paused"></span>
						Paused ({paused.length})
					</div>
					{#each paused as session (session.id)}
						{@const st = $allSessionStatuses[session.id]}
						<button
							class="session-card"
							class:active={session.id === $activeSessionId}
							onclick={() => switchAgentSession(session.id)}
							oncontextmenu={(e) => onContextMenu(e, session.id)}
						>
							<div class="card-top">
								<span class="card-name">{session.label}</span>
								<span class="card-time">{formatElapsed(session.createdAt)}</span>
							</div>
							<div class="card-branch">{session.branchName}</div>
							<div class="card-bottom">
								<span class="card-agent">{formatProvider(session.providerId)}</span>
								{#if st}
									<span class="card-meta">+{st.linesAdded}/-{st.linesRemoved} · ${st.totalCostUsd.toFixed(2)}</span>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{/if}

			{#if errored.length > 0}
				<div class="group">
					<div class="group-label">
						<span class="group-dot dot-error"></span>
						Error ({errored.length})
					</div>
					{#each errored as session (session.id)}
						{@const st = $allSessionStatuses[session.id]}
						<button
							class="session-card error"
							class:active={session.id === $activeSessionId}
							onclick={() => switchAgentSession(session.id)}
							oncontextmenu={(e) => onContextMenu(e, session.id)}
						>
							<div class="card-top">
								<span class="card-name">{session.label}</span>
								<span class="card-time">{formatElapsed(session.createdAt)}</span>
							</div>
							<div class="card-branch">{session.branchName}</div>
							<div class="card-bottom">
								<span class="card-agent">{formatProvider(session.providerId)}</span>
								{#if st}
									<span class="card-meta">+{st.linesAdded}/-{st.linesRemoved} · ${st.totalCostUsd.toFixed(2)}</span>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{/if}

			{#if queued.length > 0}
				<div class="group">
					<div class="group-label">
						<span class="group-dot dot-idle"></span>
						Queued ({queued.length})
					</div>
					{#each queued as session (session.id)}
						<button
							class="session-card queued"
							class:active={session.id === $activeSessionId}
							onclick={() => switchAgentSession(session.id)}
						>
							<div class="card-top">
								<span class="card-name">{session.label}</span>
							</div>
							<div class="card-branch">{session.branchName}</div>
							<div class="card-bottom">
								<span class="card-agent">{formatProvider(session.providerId)}</span>
							</div>
						</button>
					{/each}
				</div>
			{/if}

			{#if done.length > 0}
				<div class="group">
					<div class="group-label">
						<span class="group-dot dot-done"></span>
						Completed ({done.length})
					</div>
					{#each done as session (session.id)}
						{@const st = $allSessionStatuses[session.id]}
						<button
							class="session-card done"
							class:active={session.id === $activeSessionId}
							onclick={() => switchAgentSession(session.id)}
							oncontextmenu={(e) => onContextMenu(e, session.id)}
						>
							<div class="card-top">
								<span class="card-name">{session.label}</span>
								<span class="card-time">{formatElapsed(session.createdAt)}</span>
							</div>
							<div class="card-branch">{session.branchName}</div>
							<div class="card-bottom">
								<span class="card-agent">{formatProvider(session.providerId)}</span>
								{#if st}
									<span class="card-meta">+{st.linesAdded}/-{st.linesRemoved} · ${st.totalCostUsd.toFixed(2)}</span>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>

{#if showNewModal}
	<NewSessionModal onClose={() => showNewModal = false} />
{/if}

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--color-base);
		border-right: 1px solid var(--color-border-muted);
		width: 280px;
		flex-shrink: 0;
	}

	.sidebar-header {
		display: flex;
		align-items: center;
		padding: 10px 12px;
		border-bottom: 1px solid var(--color-border-muted);
		gap: 6px;
		flex-shrink: 0;
	}
	.sidebar-title {
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-secondary);
	}
	.sidebar-stats {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-left: auto;
		font-size: 10px;
		color: var(--color-text-subtle);
	}
	.stat-sep { opacity: 0.3; }
	.sidebar-new {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		cursor: pointer;
		padding: 2px 4px;
		border-radius: 3px;
		display: flex;
	}
	.sidebar-new:hover {
		background: var(--color-overlay);
		color: var(--color-text);
	}
	.sidebar-close {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		cursor: pointer;
		padding: 2px;
		border-radius: 3px;
		display: flex;
	}
	.sidebar-close:hover {
		background: var(--color-overlay);
		color: var(--color-text);
	}

	.sidebar-body {
		flex: 1;
		overflow-y: auto;
		padding: 8px 0;
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		padding: 48px 24px;
		color: var(--color-text-subtle);
	}
	.empty-text {
		font-size: 12px;
	}
	.empty-btn {
		background: var(--color-overlay);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 11px;
		font-weight: 500;
		padding: 5px 14px;
		cursor: pointer;
		margin-top: 4px;
	}
	.empty-btn:hover {
		background: var(--color-surface);
	}

	.group {
		margin-bottom: 4px;
	}
	.group-label {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 14px;
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-subtle);
	}
	.group-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.dot-working { background: var(--color-success, #3fb950); animation: pulse 2s infinite; }
	.dot-idle    { background: var(--color-text-subtle, #666); }
	.dot-done    { background: var(--color-success, #3fb950); }
	.dot-error   { background: var(--color-error, #f85149); }
	.dot-paused  { background: var(--color-warning, #d29922); }

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.4; }
	}

	.session-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		width: 100%;
		padding: 8px 14px;
		background: none;
		border: none;
		border-left: 2px solid transparent;
		cursor: pointer;
		text-align: left;
		color: var(--color-text);
		transition: background 0.1s;
	}
	.session-card:hover {
		background: var(--color-overlay);
	}
	.session-card.active {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		border-left-color: var(--color-accent);
	}
	.session-card.done    { opacity: 0.6; }
	.session-card.queued  { opacity: 0.7; }
	.session-card.error   { border-left-color: var(--color-error, #f85149); }

	.card-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.card-name {
		font-size: 12px;
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.card-time {
		font-size: 10px;
		color: var(--color-text-subtle);
		flex-shrink: 0;
	}
	.card-branch {
		font-size: 10px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		color: var(--color-text-subtle);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.card-bottom {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.card-agent {
		font-size: 10px;
		color: var(--color-text-subtle);
	}
	.card-meta {
		font-size: 10px;
		color: var(--color-text-subtle);
	}

	.ctx-backdrop {
		position: fixed;
		inset: 0;
		z-index: 199;
	}
	.ctx-menu {
		position: fixed;
		z-index: 200;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0,0,0,0.3);
		padding: 4px;
		min-width: 160px;
	}
	.ctx-item {
		display: block;
		width: 100%;
		padding: 6px 12px;
		background: none;
		border: none;
		border-radius: 4px;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
	}
	.ctx-item:hover { background: var(--color-overlay); }
	.ctx-item.danger { color: var(--color-error, #f85149); }
</style>
