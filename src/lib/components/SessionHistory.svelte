<script lang="ts">
	import {
		agentSessions,
		switchAgentSession,
		closeAgentSession,
		type AgentSession,
	} from '$lib/stores/agentTerminal';
	import { agentProviders } from '$lib/stores/agentProvider';

	interface Props {
		onClose: () => void;
	}

	let { onClose }: Props = $props();

	function getProviderName(id: string): string {
		const providers = $agentProviders;
		return providers.find(p => p.id === id)?.displayName ?? id;
	}

	function timeAgo(timestamp: number): string {
		const seconds = Math.floor((Date.now() - timestamp) / 1000);
		if (seconds < 60) return 'just now';
		const minutes = Math.floor(seconds / 60);
		if (minutes < 60) return `${minutes}m ago`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		return `${days}d ago`;
	}

	function statusIcon(status: string): string {
		switch (status) {
			case 'working': return '●';
			case 'idle': return '◐';
			case 'done': return '✓';
			case 'error': return '✕';
			case 'paused': return '⏸';
			default: return '●';
		}
	}

	function selectSession(id: string) {
		switchAgentSession(id);
		onClose();
	}

	const activeSessions = $derived($agentSessions.filter(s => s.status !== 'done'));
	const completedSessions = $derived($agentSessions.filter(s => s.status === 'done'));
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="history-backdrop" onclick={onClose}>
	<div class="history-panel" onclick={(e) => e.stopPropagation()}>
		<div class="history-header">
			<span class="history-title">Session History</span>
			<button class="history-close" onclick={onClose}>×</button>
		</div>

		{#if $agentSessions.length === 0}
			<div class="empty">
				<span class="empty-text">No sessions yet</span>
			</div>
		{:else}
			{#if activeSessions.length > 0}
				<div class="section">
					<div class="section-label">Active</div>
					{#each activeSessions as session (session.id)}
						<button class="session-row" onclick={() => selectSession(session.id)}>
							<span class="row-status">{statusIcon(session.status)}</span>
							<span class="row-name">{session.label || session.branchName}</span>
							<span class="row-meta">{getProviderName(session.providerId)} — {timeAgo(session.createdAt)}</span>
						</button>
					{/each}
				</div>
			{/if}

			{#if completedSessions.length > 0}
				<div class="section">
					<div class="section-label">Completed</div>
					{#each completedSessions as session (session.id)}
						<div class="session-row completed">
							<span class="row-status">✓</span>
							<span class="row-name">{session.label || session.branchName}</span>
							<span class="row-meta">{timeAgo(session.createdAt)}</span>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>

<style>
	.history-backdrop {
		position: fixed;
		inset: 0;
		z-index: 250;
	}
	.history-panel {
		position: absolute;
		top: 32px;
		right: 8px;
		width: 360px;
		max-height: 400px;
		overflow-y: auto;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 10px;
		box-shadow: 0 8px 24px rgba(0,0,0,0.3);
	}
	.history-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid var(--color-border-muted);
	}
	.history-title {
		font-size: 13px;
		font-weight: 600;
	}
	.history-close {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 16px;
		cursor: pointer;
	}
	.empty {
		padding: 24px;
		text-align: center;
	}
	.empty-text {
		color: var(--color-text-subtle);
		font-size: 13px;
	}
	.section {
		padding: 8px 0;
	}
	.section-label {
		padding: 4px 16px;
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-subtle);
	}
	.session-row {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 8px 16px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
	}
	.session-row:hover {
		background: var(--color-overlay);
	}
	.session-row.completed {
		cursor: default;
		opacity: 0.6;
	}
	.row-status {
		flex-shrink: 0;
		font-size: 10px;
	}
	.row-name {
		flex: 1;
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.row-meta {
		flex-shrink: 0;
		color: var(--color-text-subtle);
		font-size: 11px;
	}
</style>
