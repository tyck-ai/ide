<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { open } from '@tauri-apps/plugin-dialog';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import AwarenessBar from '$lib/components/AwarenessBar.svelte';
	import ContextZone from '$lib/components/ContextZone.svelte';
	import FocusZone from '$lib/components/FocusZone.svelte';
	import InsightZone from '$lib/components/InsightZone.svelte';
	import CommandRail from '$lib/components/CommandRail.svelte';
	import TerminalPanel from '$lib/components/TerminalPanel.svelte';
	import SettingsView from '$lib/components/SettingsView.svelte';
	import WelcomeView from '$lib/components/WelcomeView.svelte';
	import GitView from '$lib/components/GitView.svelte';
	import GitStatusBar from '$lib/components/GitStatusBar.svelte';
	import BranchSwitcher from '$lib/components/BranchSwitcher.svelte';
	import QuickCommitModal from '$lib/components/QuickCommitModal.svelte';
	import AppLauncher from '$lib/components/AppLauncher.svelte';
	import PermissionReview from '$lib/components/PermissionReview.svelte';
	import ToastContainer from '$lib/components/ToastContainer.svelte';
	import LspMissingNotification from '$lib/components/LspMissingNotification.svelte';
	import SessionBar from '$lib/components/SessionBar.svelte';
	import SessionSidebar from '$lib/components/SessionSidebar.svelte';
	import QuickOpenPalette from '$lib/components/QuickOpenPalette.svelte';
	import { isAgentMode } from '$lib/stores/settings';
	import { activeSessionId } from '$lib/stores/activeSession';
	import { agentModeSessions } from '$lib/stores/agentTerminal';
	import NewSessionModal from '$lib/components/NewSessionModal.svelte';
	import { TappContainer } from '$lib/components/tapp';
	import { projectRoot, resetWorkspace } from '$lib/stores/editor';
	import { showContext, showInsight, showSettings, showGitView, showBranchSwitcher, showQuickCommit, showAppLauncher, pendingInstall, gitViewTab, gitAgentSessionId, showSessionSidebar, showQuickOpen, quickOpenMode, contextZoneTab, showProblems } from '$lib/stores/layout';
	import { activeApp, tapp } from '$lib/stores/tapp';
	import { toggleTerminal, terminalVisible } from '$lib/stores/terminal';
	import { matchesBinding } from '$lib/stores/keybindings';
	import { startAgentStatusListener, stopAgentStatusListener } from '$lib/stores/agentStatus';
	import { startGitPoller, git } from '$lib/stores/git';
	import '$lib/stores/agentGit'; // bootstrap agent git auto-tracking
	import { checkProjectOnOpen } from '$lib/lsp/serverDiscovery';
	import { lspClientManager } from '$lib/lsp/LspClientManager';
	import { initSettings, settings } from '$lib/stores/settings';
	import { applyDefaultProvider } from '$lib/stores/agentProvider';
	import { get } from 'svelte/store';
	import { activeThemeId, applyTheme, loadCustomThemes, allThemes, builtinThemes } from '$lib/themes';
	import { log } from '$lib/log';
	import { initDiagnostics } from '$lib/diagnostics';

	let ready = $state(false);
	let showAgentWelcomeModal = $state(false);

	// In agent mode, only show ContextZone when there's an active session
	const showContextZone = $derived($showContext && (!$isAgentMode || !!$activeSessionId));
	// Show the unified welcome view when agent mode has no sessions yet
	const agentWelcome = $derived($isAgentMode && $agentModeSessions.length === 0);
	let unlistenFns: (() => void)[] = [];
	let contextWidth = $state(240);
	let appWidth = $state(300);
	let insightWidth = $state(320);
	let dragging = $state<'context' | 'insight' | 'app' | null>(null);
	let dragStartX = 0;
	let dragStartWidth = 0;

	function onMouseDown(handle: 'context' | 'insight' | 'app', e: MouseEvent) {
		dragging = handle;
		dragStartX = e.clientX;
		dragStartWidth = handle === 'context' ? contextWidth : handle === 'app' ? appWidth : insightWidth;
	}

	function onMouseMove(e: MouseEvent) {
		if (!dragging) return;
		const delta = e.clientX - dragStartX;
		if (dragging === 'context') {
			contextWidth = Math.max(160, Math.min(500, dragStartWidth + delta));
		} else if (dragging === 'app') {
			appWidth = Math.max(200, Math.min(600, dragStartWidth + delta));
		} else {
			insightWidth = Math.max(200, Math.min(600, dragStartWidth - delta));
		}
	}

	function onMouseUp() {
		dragging = null;
	}

	function onKeydown(e: KeyboardEvent) {
		// Cmd+T — toggle terminal (also handled inside Monaco via addCommand)
		if (matchesBinding(e, 'cmd+t')) {
			e.preventDefault();
			toggleTerminal();
			return;
		}
		// Ctrl+` — also toggles terminal
		if (e.ctrlKey && e.key === '`') {
			e.preventDefault();
			toggleTerminal();
		}
		// Git shortcuts
		if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'G') {
			e.preventDefault();
			showQuickCommit.set(true);
		}
		if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'B') {
			e.preventDefault();
			showBranchSwitcher.set(true);
		}
		// Cmd+G - Open GitView (context-aware: agent session in agent mode, changes in dev mode)
		// On macOS: metaKey+g. On Windows/Linux: ctrlKey+g (but NOT bare ctrlKey without meta on macOS)
		const isMacOs = navigator.platform.toUpperCase().includes('MAC');
		const gitModifier = isMacOs ? e.metaKey : (e.ctrlKey && !e.metaKey);
		if (gitModifier && e.key === 'g' && !e.shiftKey) {
			e.preventDefault();
			if (get(isAgentMode) && get(activeSessionId)) {
				gitViewTab.set('agent');
				gitAgentSessionId.set(get(activeSessionId));
			} else {
				gitViewTab.set('changes');
			}
			showGitView.set(true);
		}
		// Cmd+Shift+A (Mac) / Ctrl+Shift+A (Windows/Linux) - Open App Launcher
		if (e.shiftKey && (e.key === 'A' || e.key === 'a')) {
			const isMac = navigator.platform.toUpperCase().includes('MAC');
			const modifierPressed = isMac ? e.metaKey : e.ctrlKey;
			if (modifierPressed) {
				e.preventDefault();
				showAppLauncher.set(true);
			}
		}
		// Cmd+P — Quick open (go to file)
		if ((e.metaKey || e.ctrlKey) && e.key === 'p' && !e.shiftKey) {
			e.preventDefault();
			quickOpenMode.set('file');
			showQuickOpen.set(true);
		}
		// Ctrl+G — Go to line (macOS: Ctrl without Meta; Windows/Linux: not used here to avoid conflict with Git)
		if (e.ctrlKey && !e.metaKey && e.key === 'g' && isMacOs) {
			e.preventDefault();
			quickOpenMode.set('line');
			showQuickOpen.set(true);
		}
		// Cmd+Shift+F — Find in files (open search panel)
		if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'F' || e.key === 'f')) {
			e.preventDefault();
			contextZoneTab.set('search');
			showContext.set(true);
		}
		// Cmd+Shift+H — Replace in files (same as search panel but with replace mode)
		if ((e.metaKey || e.ctrlKey) && e.shiftKey && (e.key === 'H' || e.key === 'h')) {
			e.preventDefault();
			contextZoneTab.set('search');
			showContext.set(true);
		}
		// Cmd+Shift+M — Toggle Problems panel
		if (matchesBinding(e, 'cmd+shift+m')) {
			e.preventDefault();
			const open = !get(showProblems);
			showProblems.set(open);
			if (open) terminalVisible.set(false);
		}
	}

	async function setWorkspace(cwd: string) {
		// Stop any servers from the previous workspace before switching.
		lspClientManager.stopAll().catch((e) => log.warn('[setWorkspace] lspClientManager.stopAll', e));
		resetWorkspace();
		projectRoot.set(cwd);
		startGitPoller(cwd);
		// Non-blocking: detect project languages and warn about missing servers
		checkProjectOnOpen(cwd).catch((e) => log.warn('[setWorkspace] checkProjectOnOpen', e));
		// Update window title to reflect the active project.
		const folderName = cwd.split('/').pop() ?? cwd;
		getCurrentWindow().setTitle(`${folderName} — tyck`).catch((e) => log.warn('[setWorkspace] setTitle', e));
	}

	async function openFolder() {
		const selected = await open({ directory: true, multiple: false });
		if (selected && typeof selected === 'string') {
			await setWorkspace(selected);
		}
	}

	function closeActiveApp() {
		const app = $activeApp;
		if (app) {
			tapp.stop(app.id);
			tapp.setActiveApp(null);
		}
	}

	onMount(async () => {
		initDiagnostics();
		log.info('[app] tyck starting');
		await initSettings();
		applyDefaultProvider();

		// Initialize theme system
		await loadCustomThemes();
		const currentSettings = get(settings);
		const themeId = currentSettings.activeTheme || 'catppuccin-mocha';
		activeThemeId.set(themeId);
		const themes = get(allThemes);
		const theme = themes.find(t => t.id === themeId) ?? builtinThemes[0];
		applyTheme(theme);

		// Startup cleanup and checks
		try {
			// Clean up orphaned worktrees from previous sessions
			const cleaned = await invoke<number>('cleanup_stale_worktrees');
			if (cleaned > 0) {
				console.log(`[startup] Cleaned up ${cleaned} stale worktree(s)`);
			}
		} catch (e) {
			console.warn('[startup] Failed to cleanup stale worktrees:', e);
		}

		try {
			// Check git version meets minimum requirements
			const gitOk = await invoke<boolean>('check_git_version');
			if (!gitOk) {
				console.warn('[startup] Git version is below 2.17. Worktree features may not work correctly.');
			}
		} catch (e) {
			console.warn('[startup] Failed to check git version:', e);
		}

		// Each window receives its workspace path as a URL query param injected by Tauri.
		const params = new URLSearchParams(window.location.search);
		const workspaceParam = params.get('workspace');
		if (workspaceParam) {
			await setWorkspace(decodeURIComponent(workspaceParam));
		}

		startAgentStatusListener();
		ready = true;

		const myLabel = getCurrentWindow().label;
		unlistenFns.push(await listen<string>('open-settings', (event) => {
			if (event.payload === myLabel) {
				showSettings.set(true);
			}
		}));
	});

	onDestroy(() => {
		log.info('[app] tyck window closing');
		lspClientManager.stopAll().catch(() => {});
		stopAgentStatusListener();
		for (const unlisten of unlistenFns) unlisten();
	});
</script>

<svelte:window onmousemove={onMouseMove} onmouseup={onMouseUp} onkeydown={onKeydown} />

{#if !ready}
	<div class="startup-loading">
		<div class="startup-dot"></div>
	</div>
{:else if !$projectRoot}
	<WelcomeView onOpen={openFolder} onOpenRecent={setWorkspace} />
{:else}
	<div class="app-layout" class:resizing={dragging !== null}>
		<div class="mode-row">
			{#if $isAgentMode}
				<SessionBar />
			{/if}
		</div>
		<div class="awareness-row">
			<AwarenessBar />
		</div>

		<div class="main-area">
		{#if agentWelcome}
			<!-- Agent mode, no sessions: full-width welcome -->
			<div class="agent-welcome">
				<div class="agent-welcome-wordmark">tyck</div>
				<button class="agent-welcome-btn" onclick={() => showAgentWelcomeModal = true}>
					+ New Session
				</button>
			</div>
		{:else}
		{#if $showSessionSidebar && $isAgentMode}
			<SessionSidebar onClose={() => showSessionSidebar.set(false)} />
		{/if}
		<div class="main-row" style="grid-template-columns: {
			$activeApp
				? $activeApp.layout === 'sidebar'
					? `${appWidth}px 4px 1fr 4px ${$showInsight ? insightWidth : 0}px`
					: `1fr 4px ${$showInsight ? insightWidth : 0}px`
				: `${showContextZone ? `${contextWidth}px 4px ` : ''}1fr 4px ${$showInsight ? insightWidth : 0}px`
		}">
			{#if $activeApp && $activeApp.layout === 'sidebar'}
				<div class="zone tapp-zone">
					<TappContainer appId={$activeApp.id} layout="sidebar" onClose={closeActiveApp} />
				</div>
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="drag-handle" onmousedown={(e) => onMouseDown('app', e)}></div>
				<div class="zone focus">
					<FocusZone />
				</div>
			{:else if $activeApp}
				<div class="zone tapp-zone">
					<TappContainer appId={$activeApp.id} onClose={closeActiveApp} />
				</div>
			{:else}
				{#if showContextZone}
					<div class="zone context">
						<ContextZone />
					</div>
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="drag-handle" onmousedown={(e) => onMouseDown('context', e)}></div>
				{/if}

				<div class="zone focus">
					<FocusZone />
				</div>
			{/if}

			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="drag-handle"
				class:drag-hidden={!$showInsight}
				onmousedown={(e) => onMouseDown('insight', e)}
			></div>
			<div class="zone insight" class:zone-hidden={!$showInsight}>
				<InsightZone />
			</div>

			<TerminalPanel />
		</div>
		{/if}
		</div>

		<div class="command-row">
			<CommandRail />
		</div>

		{#if $git.isRepo}
			<GitStatusBar />
		{/if}
	</div>

	{#if showAgentWelcomeModal}
		<NewSessionModal onClose={() => showAgentWelcomeModal = false} />
	{/if}

	{#if $showBranchSwitcher}
		<BranchSwitcher />
	{/if}

	{#if $showQuickCommit}
		<QuickCommitModal />
	{/if}

{/if}

{#if $showAppLauncher}
	<AppLauncher />
{/if}

{#if $pendingInstall}
	<PermissionReview />
{/if}

{#if ready && $showSettings}
	<SettingsView />
{/if}

{#if ready && $showGitView}
	<GitView />
{/if}

<QuickOpenPalette />
<ToastContainer />
<LspMissingNotification />

<style>
	:global(*) {
		margin: 0;
		padding: 0;
		box-sizing: border-box;
	}
	:global(body) {
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
		background: var(--color-base);
		color: var(--color-text);
		overflow: hidden;
	}
	/* Prevent macOS WebKit from showing the grey "no" circle on disabled elements */
	:global(:disabled),
	:global([disabled]) {
		cursor: default !important;
	}
	.app-layout {
		display: flex;
		flex-direction: column;
		height: 100vh;
		width: 100vw;
		overflow: hidden;
	}
	.app-layout.resizing {
		cursor: col-resize;
		user-select: none;
	}
	.mode-row {
		flex-shrink: 0;
	}
	.awareness-row {
		height: 36px;
		flex-shrink: 0;
	}
	.main-area {
		flex: 1;
		display: flex;
		overflow: hidden;
	}
	.main-row {
		flex: 1;
		display: grid;
		overflow: hidden;
		position: relative;
	}
	.command-row {
		flex-shrink: 0;
	}
	.zone {
		overflow: hidden;
	}
	.zone.context {
		border-right: 1px solid var(--color-border-muted);
	}
	.zone.insight {
		border-left: 1px solid var(--color-border-muted);
	}
	.zone-hidden {
		overflow: hidden;
		pointer-events: none;
		opacity: 0;
		border: none !important;
	}
	.drag-handle {
		background: transparent;
		cursor: col-resize;
		z-index: 10;
		transition: background 0.15s;
	}
	.drag-handle:hover {
		background: var(--color-accent);
		opacity: 0.25;
	}
	.drag-hidden {
		pointer-events: none;
		cursor: default;
	}

	.zone.tapp-zone {
		overflow: auto;
	}

	.agent-welcome {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 20px;
	}
	.agent-welcome-wordmark {
		font-size: 48px;
		font-weight: 700;
		letter-spacing: -1px;
		color: var(--color-text-subtle);
		opacity: 0.15;
		user-select: none;
	}
	.agent-welcome-btn {
		padding: 9px 24px;
		background: var(--color-accent);
		border: none;
		border-radius: 6px;
		color: white;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
	}
	.agent-welcome-btn:hover { filter: brightness(1.1); }
	.startup-loading {
		position: fixed;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--color-base);
	}
	.startup-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-text-subtle);
		opacity: 0.3;
		animation: pulse 1s ease-in-out infinite;
	}
	@keyframes pulse {
		0%, 100% { opacity: 0.15; transform: scale(1); }
		50% { opacity: 0.6; transform: scale(1.4); }
	}
</style>
