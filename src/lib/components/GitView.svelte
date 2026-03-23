<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/core';
	import { showGitView, gitViewTab, gitAgentSessionId } from '$lib/stores/layout';
	import { git } from '$lib/stores/git';
	import { agentModeSessions } from '$lib/stores/agentTerminal';
	import { activeSessionId } from '$lib/stores/activeSession';
	import { isAgentMode } from '$lib/stores/settings';
	import { projectRoot } from '$lib/stores/editor';
	import { activeGitRoot, gitRootPinned, pinGitRoot, unpinGitRoot } from '$lib/stores/activeGitRoot';
	import { startGitPoller } from '$lib/stores/git';
	import SubRepoPicker, { type SubRepo } from './SubRepoPicker.svelte';
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

	const activeAgentSession = $derived($agentModeSessions.find(s => s.id === $gitAgentSessionId) ?? null);
	const hasAgentSessions = $derived($agentModeSessions.length > 0);

	// Current repo display name
	const activeRepoName = $derived(
		$activeGitRoot?.split('/').pop() ?? $projectRoot?.split('/').pop() ?? ''
	);

	// Whether we're in a "parent folder with no git" situation
	const noRepo = $derived(!$git.isRepo);

	// Repo picker dropdown
	let showPicker = $state(false);
	let subRepos = $state<SubRepo[]>([]);
	let loadingRepos = $state(false);
	let initializingGit = $state(false);

	async function openPicker() {
		showPicker = !showPicker;
		if (showPicker && subRepos.length === 0) {
			loadingRepos = true;
			try {
				const root = get(projectRoot);
				if (root) {
					const result = await invoke<{ isRepo: boolean; subRepos: SubRepo[] }>('find_git_context', { cwd: root });
					subRepos = result.subRepos;
				}
			} catch { /* ignore */ }
			loadingRepos = false;
		}
	}

	function selectRepo(path: string) {
		pinGitRoot(path);
		showPicker = false;
	}

	async function initGitHere() {
		const path = get(activeGitRoot) ?? get(projectRoot);
		if (!path) return;
		initializingGit = true;
		try {
			await invoke('git_init_repo', { path });
			startGitPoller(path);
			showPicker = false;
		} catch { /* ignore */ }
		initializingGit = false;
	}

	// Load sub-repos whenever git reports no repo (parent folder scenario)
	$effect(() => {
		if (!$git.isRepo && subRepos.length === 0 && !loadingRepos) {
			loadingRepos = true;
			const root = get(projectRoot);
			if (root) {
				invoke<{ isRepo: boolean; subRepos: SubRepo[] }>('find_git_context', { cwd: root })
					.then(r => { subRepos = r.subRepos; })
					.catch(() => {})
					.finally(() => { loadingRepos = false; });
			} else {
				loadingRepos = false;
			}
		}
	});

	onMount(() => {
		git.refresh();
		git.refreshBranches();
		git.refreshCommits();
		git.refreshStashes();

		const agentMode = get(isAgentMode);
		const currentTab = get(gitViewTab);
		const activeId = get(activeSessionId);
		const sessions = get(agentModeSessions);

		if (currentTab === 'agent') {
			if (!get(gitAgentSessionId) && activeId) {
				gitAgentSessionId.set(activeId);
			}
		} else if (agentMode && activeId && sessions.find(s => s.id === activeId)) {
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

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="git-view">
	<nav class="sidebar">
		<div class="sidebar-header">
			<span class="sidebar-title">Git</span>
			<div class="repo-chip-wrap">
				<button class="repo-chip" class:pinned={$gitRootPinned} onclick={openPicker} title="Switch project">
					<span class="repo-chip-name">{activeRepoName}</span>
					<span class="repo-chip-chevron">▾</span>
				</button>
				{#if $gitRootPinned}
					<button class="unpin-btn" onclick={unpinGitRoot} title="Follow active file">
						<svg width="10" height="10" viewBox="0 0 16 16" fill="none">
							<path d="M8 1a7 7 0 100 14A7 7 0 008 1zM4.5 7.5h7" stroke="currentColor" stroke-width="1.75" stroke-linecap="round"/>
						</svg>
					</button>
				{/if}

				{#if showPicker}
					<div class="picker-backdrop" onclick={() => showPicker = false}></div>
					<div class="picker-dropdown">
						{#if loadingRepos}
							<div class="picker-loading">
								<div class="spinner"></div>
							</div>
						{:else if subRepos.length > 0}
							<div class="picker-section-label">Projects</div>
							<SubRepoPicker
								repos={subRepos}
								selected={$activeGitRoot}
								onSelect={selectRepo}
							/>
							<div class="picker-divider"></div>
						{/if}
						<button class="picker-init-btn" onclick={initGitHere} disabled={initializingGit}>
							{initializingGit ? 'Initializing...' : 'Initialize git here'}
						</button>
					</div>
				{/if}
			</div>
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
			<button class="back-btn" onclick={close}>Back to Code</button>
		</div>
	</nav>

	<main class="content">
		{#if noRepo && ($gitViewTab === 'changes' || $gitViewTab === 'history' || $gitViewTab === 'branches' || $gitViewTab === 'stashes')}
			<div class="no-repo">
				{#if subRepos.length > 0 || loadingRepos}
					<p class="no-repo-title">Choose a project</p>
					<p class="no-repo-sub">This folder contains multiple projects. Select one to view its git status.</p>
					{#if loadingRepos}
						<div class="spinner"></div>
					{:else}
						<SubRepoPicker repos={subRepos} selected={$activeGitRoot} onSelect={selectRepo} />
					{/if}
				{:else}
					<p class="no-repo-title">No git repository</p>
					<p class="no-repo-sub">Initialize git in this folder to track changes and use Agent Mode.</p>
					<button class="init-btn" onclick={initGitHere} disabled={initializingGit}>
						{initializingGit ? 'Initializing...' : 'Initialize Git'}
					</button>
				{/if}
			</div>
		{:else if $gitViewTab === 'changes'}
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
		padding: 16px 12px 10px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.sidebar-title {
		font-size: 11px;
		font-weight: 700;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.8px;
		padding: 0 4px;
	}

	/* ── Repo chip ── */
	.repo-chip-wrap {
		position: relative;
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.repo-chip {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 4px 8px;
		border-radius: 5px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		cursor: pointer;
		flex: 1;
		min-width: 0;
	}

	.repo-chip:hover {
		border-color: var(--color-border);
		background: var(--color-overlay);
	}

	.repo-chip.pinned {
		border-color: color-mix(in srgb, var(--color-accent) 40%, transparent);
	}

	.repo-chip-name {
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
	}

	.repo-chip-chevron {
		font-size: 8px;
		color: var(--color-text-subtle);
		flex-shrink: 0;
	}

	.unpin-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: 4px;
		border: none;
		background: none;
		color: var(--color-text-subtle);
		cursor: pointer;
		flex-shrink: 0;
		padding: 0;
	}

	.unpin-btn:hover {
		background: var(--color-overlay);
		color: var(--color-text);
	}

	/* ── Picker dropdown ── */
	.picker-backdrop {
		position: fixed;
		inset: 0;
		z-index: 49;
	}

	.picker-dropdown {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		right: 0;
		z-index: 50;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0,0,0,0.25);
		padding: 6px;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.picker-dropdown :global(.repo-list) {
		max-height: 200px;
		overflow-y: auto;
	}

	.picker-section-label {
		font-size: 10px;
		font-weight: 700;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.06em;
		padding: 2px 6px 4px;
	}

	.picker-loading {
		display: flex;
		justify-content: center;
		padding: 8px 0;
	}

	.picker-divider {
		height: 1px;
		background: var(--color-border-muted);
		margin: 4px 0;
	}

	.picker-init-btn {
		display: flex;
		align-items: center;
		padding: 7px 10px;
		border-radius: 5px;
		border: none;
		background: none;
		cursor: pointer;
		font-size: 12px;
		color: var(--color-text-subtle);
		text-align: left;
		width: 100%;
	}

	.picker-init-btn:hover { color: var(--color-text); background: var(--color-overlay); }
	.picker-init-btn:disabled { opacity: 0.4; cursor: not-allowed; }

	/* ── Sidebar nav ── */
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

	.agent-nav { flex: 1; }

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

	.nav-item:hover { background: var(--color-base); color: var(--color-text); }
	.nav-item.active { background: var(--color-overlay); color: var(--color-text); font-weight: 600; }
	.agent-item { gap: 8px; }

	.session-dot { font-size: 7px; color: var(--color-text-subtle); flex-shrink: 0; }
	.session-dot.active-dot { color: var(--color-success); }
	.session-label { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

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

	.back-btn:hover { color: var(--color-text); border-color: var(--color-border); }

	/* ── Content ── */
	.content {
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	/* ── No-repo empty state ── */
	.no-repo {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 24px;
		gap: 12px;
		text-align: center;
	}

	.no-repo-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.no-repo-sub {
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.5;
		max-width: 280px;
		margin: 0;
	}

	.init-btn {
		padding: 7px 18px;
		border-radius: 6px;
		background: var(--color-accent);
		border: none;
		color: white;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		margin-top: 4px;
	}

	.init-btn:disabled { opacity: 0.5; cursor: not-allowed; }

	.empty-agent {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-subtle);
		font-size: 14px;
	}

	/* ── Spinner ── */
	.spinner {
		width: 16px;
		height: 16px;
		border: 1.5px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}

	@keyframes spin { to { transform: rotate(360deg); } }
</style>
