<script lang="ts">
	import { agentModeSessions } from '$lib/stores/agentTerminal';
	import { peekingMain } from '$lib/stores/editor';
	import { showSessionSidebar } from '$lib/stores/layout';
	import NewSessionModal from './NewSessionModal.svelte';
	import NewMissionModal from './NewMissionModal.svelte';
	import SessionHistory from './SessionHistory.svelte';
	import { activeMission } from '$lib/stores/missions';

	let showNewModal = $state(false);
	let showMissionModal = $state(false);
	let showNewMenu = $state(false);
	let showHistory = $state(false);

	// Auto-open sidebar when the first session is spawned
	let prevCount = $state(0);
	$effect(() => {
		const len = $agentModeSessions.length;
		if (len > 0 && prevCount === 0) {
			showSessionSidebar.set(true);
		}
		prevCount = len;
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="session-bar">
	<button
		class="sidebar-toggle-btn"
		class:active={$showSessionSidebar}
		onclick={() => showSessionSidebar.update(v => !v)}
		title="Toggle sessions panel"
	>
		<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/>
		</svg>
		Sessions {#if $agentModeSessions.length > 0}<span class="session-count">{$agentModeSessions.length}</span>{/if}
	</button>

	<button
		class="peek-btn"
		class:active={$peekingMain}
		onclick={() => peekingMain.update(v => !v)}
		title="Peek at main workspace (read-only)"
	>
		main ↗
	</button>

	<div class="bar-spacer"></div>

	{#if $activeMission}
		<div class="mission-indicator" title={$activeMission.description}>
			🎯 {$activeMission.tasks.filter(t => t.status === 'done').length}/{$activeMission.tasks.length}
		</div>
	{/if}

	<div class="bar-actions">
		<div class="new-menu-wrapper">
			<button class="action-btn" onclick={() => showNewMenu = !showNewMenu} title="New session or mission">
				+
			</button>
			{#if showNewMenu}
				<div class="new-menu-backdrop" onclick={() => showNewMenu = false}></div>
				<div class="new-menu">
					<button class="new-menu-item" onclick={() => { showNewMenu = false; showNewModal = true; }}>
						<span class="new-menu-icon">▶</span> New Session
					</button>
					<button class="new-menu-item" onclick={() => { showNewMenu = false; showMissionModal = true; }}>
						<span class="new-menu-icon">🎯</span> New Mission
					</button>
				</div>
			{/if}
		</div>
		<button class="action-btn" onclick={() => showHistory = !showHistory} title="Session history">
			⏱
		</button>
	</div>
</div>

{#if showNewModal}
	<NewSessionModal onClose={() => showNewModal = false} />
{/if}

{#if showMissionModal}
	<NewMissionModal onClose={() => showMissionModal = false} />
{/if}

{#if showHistory}
	<SessionHistory onClose={() => showHistory = false} />
{/if}

<style>
	.session-bar {
		display: flex;
		align-items: center;
		height: 32px;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
		padding: 0 8px;
		gap: 2px;
		position: relative;
	}
	.sidebar-toggle-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		background: none;
		border: 1px solid var(--color-border-muted);
		color: var(--color-text-subtle);
		font-size: 10px;
		font-weight: 600;
		cursor: pointer;
		padding: 3px 10px;
		border-radius: 4px;
	}
	.sidebar-toggle-btn:hover {
		background: var(--color-overlay);
		color: var(--color-text);
	}
	.sidebar-toggle-btn.active {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		border-color: var(--color-accent);
		color: var(--color-accent);
	}
	.session-count {
		background: var(--color-accent);
		color: var(--color-base);
		border-radius: 8px;
		padding: 0 5px;
		font-size: 9px;
		font-weight: 700;
		line-height: 14px;
	}
	.peek-btn {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 10px;
		cursor: pointer;
		padding: 2px 8px;
		border-radius: 3px;
		margin-left: 4px;
	}
	.peek-btn:hover {
		color: var(--color-text);
		background: var(--color-overlay);
	}
	.peek-btn.active {
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}
	.bar-spacer {
		flex: 1;
	}
	.mission-indicator {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		font-size: 10px;
		color: var(--color-accent);
		font-weight: 600;
		flex-shrink: 0;
	}
	.bar-actions {
		display: flex;
		align-items: center;
		gap: 2px;
		flex-shrink: 0;
	}
	.action-btn {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 14px;
		cursor: pointer;
		padding: 2px 8px;
		border-radius: 4px;
	}
	.action-btn:hover {
		background: var(--color-overlay);
		color: var(--color-text);
	}
	.new-menu-wrapper {
		position: relative;
	}
	.new-menu-backdrop {
		position: fixed;
		inset: 0;
		z-index: 99;
	}
	.new-menu {
		position: absolute;
		top: 100%;
		right: 0;
		z-index: 100;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0,0,0,0.3);
		padding: 4px;
		min-width: 160px;
		margin-top: 4px;
	}
	.new-menu-item {
		display: flex;
		align-items: center;
		gap: 8px;
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
	.new-menu-item:hover {
		background: var(--color-overlay);
	}
	.new-menu-icon {
		font-size: 11px;
	}
</style>
