<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { open } from '@tauri-apps/plugin-dialog';
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
	import ModeBar from '$lib/components/ModeBar.svelte';
	import SessionBar from '$lib/components/SessionBar.svelte';
	import { isAgentMode } from '$lib/stores/settings';
	import { TappContainer } from '$lib/components/tapp';
	import { projectRoot, resetWorkspace } from '$lib/stores/editor';
	import { showContext, showInsight, showSettings, showGitView, showBranchSwitcher, showQuickCommit, showAppLauncher, pendingInstall, gitViewTab } from '$lib/stores/layout';
	import { activeApp, tapp } from '$lib/stores/tapp';
	import { toggleTerminal, terminalVisible } from '$lib/stores/terminal';
	import { startAgentStatusListener } from '$lib/stores/agentStatus';
	import { startGitPoller, git } from '$lib/stores/git';
	import { initSettings, updateSettings, settings } from '$lib/stores/settings';
	import { applyDefaultProvider } from '$lib/stores/agentProvider';
	import { get } from 'svelte/store';
	import { activeThemeId, applyTheme, loadCustomThemes, allThemes, builtinThemes } from '$lib/themes';

	let ready = $state(false);
	let unlistenFns: (() => void)[] = [];
	let contextWidth = $state(240);
	let appWidth = $state(300);
	let insightWidth = $state(320);
	let dragging = $state<'context' | 'insight' | 'app' | null>(null);

	function onMouseDown(handle: 'context' | 'insight' | 'app') {
		dragging = handle;
	}

	function onMouseMove(e: MouseEvent) {
		if (!dragging) return;
		if (dragging === 'context') {
			contextWidth = Math.max(160, Math.min(500, e.clientX));
		} else if (dragging === 'app') {
			appWidth = Math.max(200, Math.min(600, e.clientX));
		} else {
			insightWidth = Math.max(200, Math.min(600, window.innerWidth - e.clientX));
		}
	}

	function onMouseUp() {
		dragging = null;
	}

	function onKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === '`') {
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
		// Cmd+G - Open GitView Changes tab
		if ((e.ctrlKey || e.metaKey) && e.key === 'g' && !e.shiftKey) {
			e.preventDefault();
			gitViewTab.set('changes');
			showGitView.set(true);
		}
		// Cmd+Shift+A (Mac) / Ctrl+Shift+A (Windows/Linux) - Open App Launcher
		if (e.shiftKey && (e.key === 'A' || e.key === 'a')) {
			const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
			const modifierPressed = isMac ? e.metaKey : e.ctrlKey;
			if (modifierPressed) {
				e.preventDefault();
				showAppLauncher.set(true);
			}
		}
	}

	async function setWorkspace(cwd: string) {
		resetWorkspace();
		projectRoot.set(cwd);
		startGitPoller(cwd);
		await updateSettings({ lastOpenedFolder: cwd });
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

		// Restore last folder silently (don't prompt)
		const lastFolder = get(settings).lastOpenedFolder;
		if (lastFolder) {
			await setWorkspace(lastFolder);
		}

		startAgentStatusListener();
		ready = true;

		unlistenFns.push(await listen('open-settings', () => {
			showSettings.set(true);
		}));
		unlistenFns.push(await listen('open-folder', () => {
			openFolder();
		}));
	});

	onDestroy(() => {
		for (const unlisten of unlistenFns) unlisten();
	});
</script>

<svelte:window onmousemove={onMouseMove} onmouseup={onMouseUp} onkeydown={onKeydown} />

{#if !ready}
	<!-- Loading -->
{:else if $showSettings}
	<div class="app-layout">
		<SettingsView />
	</div>
{:else if $showGitView}
	<div class="app-layout">
		<GitView />
	</div>
{:else if !$projectRoot}
	<WelcomeView onOpen={openFolder} onOpenRecent={setWorkspace} />
{:else}
	<div class="app-layout" class:resizing={dragging !== null}>
		<div class="mode-row">
			{#if $isAgentMode}
				<SessionBar />
			{:else}
				<ModeBar />
			{/if}
		</div>
		<div class="awareness-row">
			<AwarenessBar />
		</div>

		<div class="main-row" style="grid-template-columns: {
			$activeApp
				? $activeApp.layout === 'sidebar'
					? `${appWidth}px 4px 1fr 4px ${$showInsight ? insightWidth : 0}px`
					: `1fr 4px ${$showInsight ? insightWidth : 0}px`
				: `${$showContext ? `${contextWidth}px 4px ` : ''}1fr 4px ${$showInsight ? insightWidth : 0}px`
		}">
			{#if $activeApp && $activeApp.layout === 'sidebar'}
				<div class="zone tapp-zone">
					<TappContainer appId={$activeApp.id} layout="sidebar" onClose={closeActiveApp} />
				</div>
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="drag-handle" onmousedown={() => onMouseDown('app')}></div>
				<div class="zone focus">
					<FocusZone />
				</div>
			{:else if $activeApp}
				<div class="zone tapp-zone">
					<TappContainer appId={$activeApp.id} onClose={closeActiveApp} />
				</div>
			{:else}
				{#if $showContext}
					<div class="zone context">
						<ContextZone />
					</div>
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div class="drag-handle" onmousedown={() => onMouseDown('context')}></div>
				{/if}

				<div class="zone focus">
					<FocusZone />
				</div>
			{/if}

			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="drag-handle"
				class:drag-hidden={!$showInsight}
				onmousedown={() => onMouseDown('insight')}
			></div>
			<div class="zone insight" class:zone-hidden={!$showInsight}>
				<InsightZone />
			</div>

			<TerminalPanel />
		</div>

		<div class="command-row">
			<CommandRail />
		</div>

		{#if $git.isRepo}
			<GitStatusBar />
		{/if}
	</div>

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

<ToastContainer />

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
</style>
