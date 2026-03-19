<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { showGitView, gitViewTab, gitAgentSessionId } from '$lib/stores/layout';
	import { git } from '$lib/stores/git';
	import { agentModeSessions } from '$lib/stores/agentTerminal';
	import { activeSessionId } from '$lib/stores/activeSession';
	import { isAgentMode } from '$lib/stores/settings';
	import ChangesTab from './git/ChangesTab.svelte';
	import HistoryTab from './git/HistoryTab.svelte';
	import BranchesTab from './git/BranchesTab.svelte';
	import StashesTab from './git/StashesTab.svelte';
	import AgentChangesTab from './git/AgentChangesTab.svelte';

	const mainTabs = [
		{ id: 'changes', label: 'Changes' },
		{ id: 'history', label: 'History' },
		{ id: 'branches', label: 'Branches' },
		{ id: 'stashes', label: 'Stashes' },
	] as const;

	// The agent session currently shown in the panel
	const activeAgentSession = $derived($agentModeSessions.find(s => s.id === $gitAgentSessionId) ?? null);

	// Count of agent mode sessions (for showing the section)
	const hasAgentSessions = $derived($agentModeSessions.length > 0);

	onMount(() => {
		// Refresh main workspace git data
		git.refresh();
		git.refreshBranches();
		git.refreshCommits();
		git.refreshStashes();

		// Auto-select active agent session if already in agent tab or if in agent mode
		const agentMode = get(isAgentMode);
		const currentTab = get(gitViewTab);
		const activeId = get(activeSessionId);
		const sessions = get(agentModeSessions);

		if (currentTab === 'agent') {
			// Tab already set externally (e.g. from GitStatusBar) — make sure session id is set
			if (!get(gitAgentSessionId) && activeId) {
				gitAgentSessionId.set(activeId);
			}
		} else if (agentMode && activeId && sessions.find(s => s.id === activeId)) {
			// In agent mode with an active session — preselect it
			gitViewTab.set('agent');
			gitAgentSessionId.set(activeId);
		}
	});

	function selectAgentSession(sessionId: string) {
		gitViewTab.set('agent');
		gitAgentSessionId.set(sessionId);
	}

	function close() {
		showGitView.set(false);
	}
</script>

<div class="git-view">
	<nav class="sidebar">
		<div class="sidebar-header">
			<span class="sidebar-title">Git</span>
		</div>

		<div class="sidebar-nav">
			{#each mainTabs as tab (tab.id)}
				<button
					class="nav-item"
					class:active={$gitViewTab === tab.id}
					onclick={() => gitViewTab.set(tab.id)}
				>{tab.label}</button>
			{/each}
		</div>

		{#if hasAgentSessions}
			<div class="section-divider"></div>

			<div class="sidebar-section-header">Agent Git</div>

			<div class="sidebar-nav agent-nav">
				{#each $agentModeSessions as session (session.id)}
					<button
						class="nav-item agent-item"
						class:active={$gitViewTab === 'agent' && $gitAgentSessionId === session.id}
						onclick={() => selectAgentSession(session.id)}
					>
						<span class="session-dot" class:active-dot={session.id === $activeSessionId}>●</span>
						<span class="session-label">{session.label}</span>
					</button>
				{/each}
			</div>
		{/if}

		<div class="sidebar-footer">
			<button class="back-btn" onclick={close}>
				Back to Code
			</button>
		</div>
	</nav>

	<main class="content">
		{#if $gitViewTab === 'changes'}
			<ChangesTab />
		{:else if $gitViewTab === 'history'}
			<HistoryTab />
		{:else if $gitViewTab === 'branches'}
			<BranchesTab />
		{:else if $gitViewTab === 'stashes'}
			<StashesTab />
		{:else if $gitViewTab === 'agent' && activeAgentSession}
			<AgentChangesTab session={activeAgentSession} />
		{:else if $gitViewTab === 'agent' && !activeAgentSession}
			<div class="empty-agent">
				<span>No agent session selected</span>
			</div>
		{/if}
	</main>
</div>

<style>
	.git-view {
		position: fixed;
		inset: 0;
		z-index: 100;
		display: flex;
		background: var(--color-base);
		overflow: hidden;
	}

	.sidebar {
		width: 180px;
		flex-shrink: 0;
		background: var(--color-surface);
		border-right: 1px solid var(--color-border-muted);
		display: flex;
		flex-direction: column;
	}

	.sidebar-header {
		padding: 16px 16px 12px;
	}

	.sidebar-title {
		font-size: 11px;
		font-weight: 700;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.8px;
	}

	.sidebar-nav {
		display: flex;
		flex-direction: column;
		gap: 1px;
		padding: 0 8px;
	}

	.section-divider {
		height: 1px;
		background: var(--color-border-muted);
		margin: 10px 8px 0;
	}

	.sidebar-section-header {
		padding: 10px 16px 4px;
		font-size: 11px;
		font-weight: 700;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.8px;
	}

	.agent-nav {
		flex: 1;
	}

	.nav-item {
		display: flex;
		align-items: center;
		width: 100%;
		padding: 8px 12px;
		background: none;
		border: none;
		border-radius: 6px;
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		text-align: left;
		cursor: pointer;
		gap: 0;
	}

	.nav-item:hover {
		background: var(--color-base);
		color: var(--color-text);
	}

	.nav-item.active {
		background: var(--color-overlay);
		color: var(--color-text);
		font-weight: 600;
	}

	.agent-item {
		gap: 8px;
	}

	.session-dot {
		font-size: 7px;
		color: var(--color-text-subtle);
		flex-shrink: 0;
	}

	.session-dot.active-dot {
		color: var(--color-success);
	}

	.session-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.sidebar-footer {
		padding: 12px;
		border-top: 1px solid var(--color-border-muted);
	}

	.back-btn {
		display: block;
		width: 100%;
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 600;
		padding: 6px 0;
		cursor: pointer;
		text-align: center;
	}

	.back-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.content {
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.empty-agent {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-subtle);
		font-size: 14px;
	}
</style>
