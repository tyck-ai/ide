<script lang="ts">
	import { isAgentMode, updateSettings } from '$lib/stores/settings';
	import {
		agentSessions,
		activeSession,
		switchAgentSession,
		closeAgentSession,
		pauseAgent,
		resumeAgent,
		type SessionStatus,
	} from '$lib/stores/agentTerminal';
	import { activeSessionId } from '$lib/stores/activeSession';
	import { peekingMain } from '$lib/stores/editor';
	import NewSessionModal from './NewSessionModal.svelte';
	import NewMissionModal from './NewMissionModal.svelte';
	import SessionHistory from './SessionHistory.svelte';
	import { activeMission } from '$lib/stores/missions';

	let showNewModal = $state(false);
	let showMissionModal = $state(false);
	let showNewMenu = $state(false);
	let showHistory = $state(false);
	let showModeDropdown = $state(false);
	let showSwitchWarning = $state(false);
	let contextMenuSession = $state<string | null>(null);
	let contextMenuPos = $state({ x: 0, y: 0 });

	function statusIcon(status: SessionStatus): string {
		switch (status) {
			case 'working': return '●';
			case 'idle': return '◐';
			case 'done': return '✓';
			case 'error': return '✕';
			case 'paused': return '⏸';
			default: return '●';
		}
	}

	function statusColor(status: SessionStatus): string {
		switch (status) {
			case 'working': return 'var(--color-success, #3fb950)';
			case 'idle': return 'var(--color-warning, #d29922)';
			case 'done': return 'var(--color-success, #3fb950)';
			case 'error': return 'var(--color-error, #f85149)';
			case 'paused': return 'var(--color-text-subtle, #666)';
			default: return 'var(--color-text-subtle)';
		}
	}

	function truncateBranch(name: string, max = 24): string {
		if (name.length <= max) return name;
		return name.slice(0, max - 1) + '…';
	}

	function onTabClick(id: string) {
		peekingMain.set(false);
		switchAgentSession(id);
	}

	function onTabClose(e: MouseEvent, id: string) {
		e.stopPropagation();
		closeAgentSession(id);
	}

	function onContextMenu(e: MouseEvent, id: string) {
		e.preventDefault();
		contextMenuSession = id;
		contextMenuPos = { x: e.clientX, y: e.clientY };
	}

	function closeContextMenu() {
		contextMenuSession = null;
	}

	function switchToDevMode() {
		if ($agentSessions.length > 0) {
			showSwitchWarning = true;
			return;
		}
		updateSettings({ workspaceMode: 'dev' });
	}

	function confirmSwitchToDev() {
		showSwitchWarning = false;
		updateSettings({ workspaceMode: 'dev' });
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="session-bar">
	<button class="mode-indicator agent" onclick={() => showModeDropdown = !showModeDropdown}>
		<span class="mode-dot"></span>
		Agent
		<span class="mode-chevron">▾</span>
	</button>

	{#if showModeDropdown}
		<div class="mode-dropdown-backdrop" onclick={() => showModeDropdown = false}></div>
		<div class="mode-dropdown">
			<div class="mode-dropdown-item selected">
				<span class="mode-radio">◉</span>
				<div class="mode-dropdown-info">
					<span class="mode-dropdown-label">Agent Mode</span>
					<span class="mode-dropdown-desc">Agents work on isolated branches.</span>
				</div>
			</div>
			<button class="mode-dropdown-item" onclick={() => { showModeDropdown = false; switchToDevMode(); }}>
				<span class="mode-radio">○</span>
				<div class="mode-dropdown-info">
					<span class="mode-dropdown-label">Dev Mode</span>
					<span class="mode-dropdown-desc">Agent edits your files directly.</span>
				</div>
			</button>
		</div>
	{/if}

	<button
		class="peek-btn"
		class:active={$peekingMain}
		onclick={() => peekingMain.update(v => !v)}
		title="Peek at main workspace (read-only)"
	>
		main ↗
	</button>

	<div class="session-divider"></div>

	<div class="session-tabs">
		{#each $agentSessions as session (session.id)}
			<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
			<div
				class="session-tab"
				class:active={session.id === $activeSessionId}
				onclick={() => onTabClick(session.id)}
				oncontextmenu={(e) => onContextMenu(e, session.id)}
				title="{session.branchName} ({session.status})"
				role="tab"
				tabindex="0"
			>
				<span class="session-status" style="color: {statusColor(session.status)}">
					{statusIcon(session.status)}
				</span>
				<span class="session-name">{truncateBranch(session.label || session.branchName)}</span>
				<button
					class="session-close"
					onclick={(e) => onTabClose(e, session.id)}
					title="Close session"
				>×</button>
			</div>
		{/each}
	</div>

	<div class="session-actions">
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

	{#if $activeMission}
		<div class="mission-indicator" title={$activeMission.description}>
			🎯 {$activeMission.tasks.filter(t => t.status === 'done').length}/{$activeMission.tasks.length}
		</div>
	{/if}
</div>

{#if contextMenuSession}
	<div class="context-backdrop" onclick={closeContextMenu}></div>
	{@const menuSession = $agentSessions.find(s => s.id === contextMenuSession)}
	<div class="context-menu" style="left: {contextMenuPos.x}px; top: {contextMenuPos.y}px">
		{#if menuSession?.status === 'paused'}
			<button class="context-item" onclick={() => { resumeAgent(contextMenuSession!); closeContextMenu(); }}>
				Resume Agent
			</button>
		{:else if menuSession?.status === 'working'}
			<button class="context-item" onclick={() => { pauseAgent(contextMenuSession!); closeContextMenu(); }}>
				Pause Agent
			</button>
		{/if}
		<button class="context-item danger" onclick={() => { closeAgentSession(contextMenuSession!); closeContextMenu(); }}>
			Close Session
		</button>
	</div>
{/if}

{#if showNewModal}
	<NewSessionModal onClose={() => showNewModal = false} />
{/if}

{#if showMissionModal}
	<NewMissionModal onClose={() => showMissionModal = false} />
{/if}

{#if showHistory}
	<SessionHistory onClose={() => showHistory = false} />
{/if}

{#if showSwitchWarning}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="switch-backdrop" onclick={() => showSwitchWarning = false}>
		<div class="switch-modal" onclick={(e) => e.stopPropagation()}>
			<div class="switch-title">Switch to Dev Mode?</div>
			<div class="switch-text">
				You have {$agentSessions.length} active agent session{$agentSessions.length > 1 ? 's' : ''}.
				Switching will close them and clean up their worktrees.
			</div>
			<div class="switch-actions">
				<button class="switch-btn cancel" onclick={() => showSwitchWarning = false}>Cancel</button>
				<button class="switch-btn confirm" onclick={confirmSwitchToDev}>Switch Anyway</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.session-bar {
		display: flex;
		align-items: center;
		height: 32px;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
		padding: 0 8px;
		gap: 0;
		position: relative;
	}
	.mode-indicator {
		display: flex;
		align-items: center;
		gap: 5px;
		background: none;
		border: none;
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		padding: 2px 8px;
		flex-shrink: 0;
		border-radius: 4px;
	}
	.mode-indicator:hover {
		background: var(--color-overlay);
	}
	.mode-chevron {
		font-size: 8px;
		opacity: 0.5;
	}
	.mode-dropdown-backdrop {
		position: fixed;
		inset: 0;
		z-index: 99;
	}
	.mode-dropdown {
		position: absolute;
		top: 100%;
		left: 8px;
		z-index: 100;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0,0,0,0.3);
		padding: 4px;
		min-width: 260px;
		margin-top: 4px;
	}
	.mode-dropdown-item {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		width: 100%;
		padding: 10px 12px;
		background: none;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		text-align: left;
		color: var(--color-text);
	}
	.mode-dropdown-item:hover { background: var(--color-overlay); }
	.mode-dropdown-item.selected {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		cursor: default;
	}
	.mode-radio { color: var(--color-accent); font-size: 14px; margin-top: 1px; }
	.mode-dropdown-info { display: flex; flex-direction: column; gap: 2px; }
	.mode-dropdown-label { font-size: 13px; font-weight: 600; }
	.mode-dropdown-desc { font-size: 11px; color: var(--color-text-subtle); }

	.switch-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}
	.switch-modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 10px;
		padding: 24px;
		max-width: 380px;
		width: 90%;
	}
	.switch-title { font-size: 15px; font-weight: 600; margin-bottom: 12px; }
	.switch-text { font-size: 13px; color: var(--color-text-secondary); line-height: 1.5; margin-bottom: 20px; }
	.switch-actions { display: flex; justify-content: flex-end; gap: 8px; }
	.switch-btn { padding: 7px 16px; border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; }
	.switch-btn.cancel { background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text); }
	.switch-btn.confirm { background: var(--color-warning, #fbbf24); border: none; color: #1a1a1a; }
	.mode-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-accent);
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
	.session-divider {
		width: 1px;
		height: 16px;
		background: var(--color-border-muted);
		margin: 0 4px;
		flex-shrink: 0;
	}
	.session-tabs {
		display: flex;
		align-items: center;
		gap: 1px;
		overflow-x: auto;
		flex: 1;
		scrollbar-width: none;
	}
	.session-tabs::-webkit-scrollbar { display: none; }
	.session-tab {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 3px 8px;
		background: none;
		border: none;
		border-radius: 4px;
		color: var(--color-text-subtle);
		font-size: 11px;
		cursor: pointer;
		white-space: nowrap;
		flex-shrink: 0;
	}
	.session-tab:hover {
		background: var(--color-overlay);
		color: var(--color-text);
	}
	.session-tab.active {
		background: var(--color-base);
		color: var(--color-text);
	}
	.session-status {
		font-size: 10px;
		flex-shrink: 0;
	}
	.session-name {
		max-width: 160px;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.session-close {
		background: none;
		border: none;
		color: inherit;
		font-size: 12px;
		cursor: pointer;
		opacity: 0;
		padding: 0 2px;
		margin-left: 2px;
	}
	.session-tab:hover .session-close {
		opacity: 0.5;
	}
	.session-close:hover {
		opacity: 1 !important;
	}
	.session-actions {
		display: flex;
		align-items: center;
		gap: 2px;
		margin-left: auto;
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
	.mission-indicator {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		font-size: 10px;
		color: var(--color-accent);
		font-weight: 600;
		flex-shrink: 0;
		margin-left: 4px;
	}

	.context-backdrop {
		position: fixed;
		inset: 0;
		z-index: 199;
	}
	.context-menu {
		position: fixed;
		z-index: 200;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0,0,0,0.3);
		padding: 4px;
		min-width: 160px;
	}
	.context-item {
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
	.context-item:hover {
		background: var(--color-overlay);
	}
	.context-item.danger {
		color: var(--color-error, #f85149);
	}
</style>
