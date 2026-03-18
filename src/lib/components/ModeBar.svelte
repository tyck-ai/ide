<script lang="ts">
	import { get } from 'svelte/store';
	import { invoke } from '@tauri-apps/api/core';
	import { isAgentMode, isDevMode, settings, updateSettings } from '$lib/stores/settings';
	import { projectRoot } from '$lib/stores/editor';
	import { toast } from '$lib/stores/toast';
	import { startGitPoller } from '$lib/stores/git';

	let showDropdown = $state(false);
	let showGitInit = $state(false);
	let initializingGit = $state(false);

	function toggleDropdown() {
		showDropdown = !showDropdown;
	}

	async function selectMode(mode: 'dev' | 'agent') {
		showDropdown = false;

		if (mode === $settings.workspaceMode) return;

		// Check for git repo when switching to agent mode
		if (mode === 'agent') {
			const cwd = get(projectRoot);
			if (cwd) {
				const isRepo = await invoke<boolean>('git_is_repo', { path: cwd });
				if (!isRepo) {
					showGitInit = true;
					return;
				}
			}
		}

		// Sessions and worktrees are always preserved — switching mode is non-destructive.
		updateSettings({ workspaceMode: mode });
	}

	async function initGitAndSwitch() {
		const cwd = get(projectRoot);
		if (!cwd) return;
		initializingGit = true;
		try {
			await invoke('git_init_repo', { path: cwd });
			toast.success('Git initialized. Agent Mode ready.');
			startGitPoller(cwd);
			updateSettings({ workspaceMode: 'agent' });
		} catch (e) {
			toast.error(`Failed to initialize git: ${e}`);
		}
		initializingGit = false;
		showGitInit = false;
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="mode-bar">
	<button class="mode-indicator" class:agent={$isAgentMode} onclick={toggleDropdown}>
		<span class="mode-dot" class:agent={$isAgentMode}></span>
		{$isAgentMode ? 'Agent' : 'Dev'} Mode
		<span class="mode-chevron">▾</span>
	</button>

	{#if showDropdown}
		<div class="dropdown-backdrop" onclick={() => showDropdown = false}></div>
		<div class="dropdown">
			<button class="dropdown-item" class:selected={$isDevMode} onclick={() => selectMode('dev')}>
				<span class="dropdown-radio">{$isDevMode ? '◉' : '○'}</span>
				<div class="dropdown-info">
					<span class="dropdown-label">Dev Mode</span>
					<span class="dropdown-desc">Edit files directly. Agent sessions remain open in the background.</span>
				</div>
			</button>
			<button class="dropdown-item" class:selected={$isAgentMode} onclick={() => selectMode('agent')}>
				<span class="dropdown-radio">{$isAgentMode ? '◉' : '○'}</span>
				<div class="dropdown-info">
					<span class="dropdown-label">Agent Mode</span>
					<span class="dropdown-desc">Agents work on isolated branches. You review before merging.</span>
				</div>
			</button>
		</div>
	{/if}
</div>

{#if showGitInit}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="warning-backdrop" onclick={() => showGitInit = false}>
		<div class="warning-modal" onclick={(e) => e.stopPropagation()}>
			<div class="warning-title">Agent Mode requires git</div>
			<div class="warning-text">
				This project isn't a git repository. Initialize git to use Agent Mode?
				<br /><br />
				This will run <code>git init</code> and create an initial commit with all existing files.
			</div>
			<div class="warning-actions">
				<button class="warning-btn cancel" onclick={() => showGitInit = false}>Stay in Dev Mode</button>
				<button class="warning-btn confirm" onclick={initGitAndSwitch} disabled={initializingGit}>
					{initializingGit ? 'Initializing...' : 'Initialize Git'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.mode-bar {
		display: flex;
		align-items: center;
		height: 28px;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
		padding: 0 12px;
		position: relative;
	}
	.mode-indicator {
		display: flex;
		align-items: center;
		gap: 6px;
		background: none;
		border: none;
		color: var(--color-text-secondary);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		padding: 2px 8px;
		border-radius: 4px;
	}
	.mode-indicator:hover {
		background: var(--color-overlay);
		color: var(--color-text);
	}
	.mode-indicator.agent {
		color: var(--color-accent);
	}
	.mode-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-success);
	}
	.mode-dot.agent {
		background: var(--color-accent);
	}
	.mode-chevron {
		font-size: 8px;
		opacity: 0.5;
	}

	.dropdown-backdrop {
		position: fixed;
		inset: 0;
		z-index: 99;
	}
	.dropdown {
		position: absolute;
		top: 100%;
		left: 8px;
		z-index: 100;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0,0,0,0.3);
		padding: 4px;
		min-width: 280px;
		margin-top: 4px;
	}
	.dropdown-item {
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
	.dropdown-item:hover {
		background: var(--color-overlay);
	}
	.dropdown-item.selected {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}
	.dropdown-radio {
		color: var(--color-accent);
		font-size: 14px;
		margin-top: 1px;
	}
	.dropdown-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.dropdown-label {
		font-size: 13px;
		font-weight: 600;
	}
	.dropdown-desc {
		font-size: 11px;
		color: var(--color-text-subtle);
		line-height: 1.4;
	}

	.warning-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}
	.warning-modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 10px;
		padding: 24px;
		max-width: 380px;
		width: 90%;
	}
	.warning-title {
		font-size: 15px;
		font-weight: 600;
		margin-bottom: 12px;
	}
	.warning-text {
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.5;
		margin-bottom: 20px;
	}
	.warning-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}
	.warning-btn {
		padding: 7px 16px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
	}
	.warning-btn.cancel {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		color: var(--color-text);
	}
	.warning-btn.confirm {
		background: var(--color-warning, #fbbf24);
		border: none;
		color: #1a1a1a;
	}
</style>
