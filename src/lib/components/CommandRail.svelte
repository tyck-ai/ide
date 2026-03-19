<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { get } from 'svelte/store';
	import { activeFilePath, selection, cursorLine, projectRoot } from '$lib/stores/editor';
	import { terminalVisible, toggleTerminal } from '$lib/stores/terminal';
	import { activeSessionId } from '$lib/stores/agentTerminal';
	import { isAgentMode, isDevMode, settings } from '$lib/stores/settings';
	import { switchToMode } from '$lib/stores/modeSwitch';
	import { startGitPoller } from '$lib/stores/git';
	import { toast } from '$lib/stores/toast';
	import { log } from '$lib/log';

	let showDropdown = $state(false);
	let showGitInit = $state(false);
	let initializingGit = $state(false);

	async function selectMode(mode: 'dev' | 'agent') {
		showDropdown = false;
		if (mode === $settings.workspaceMode) return;

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

		await switchToMode(mode);
	}

	async function initGitAndSwitch() {
		const cwd = get(projectRoot);
		if (!cwd) return;
		initializingGit = true;
		try {
			await invoke('git_init_repo', { path: cwd });
			toast.success('Git initialized. Agent Mode ready.');
			startGitPoller(cwd);
			await switchToMode('agent');
		} catch (e) {
			toast.error(`Failed to initialize git: ${e}`);
		}
		initializingGit = false;
		showGitInit = false;
	}

	function injectContext() {
		const termId = $activeSessionId;
		if (!termId) return;

		const fp = $activeFilePath;
		const sel = $selection;
		const line = $cursorLine;

		if (!fp && !sel) return;

		let ctx = '';
		if (fp) ctx += `[file: ${fp}`;
		if (line) ctx += `, line: ${line}`;
		if (fp) ctx += ']';
		if (sel) ctx += ` [selection: ${sel}]`;

		invoke('write_terminal', { id: termId, data: ctx + ' ' }).catch((e) => log.warn('[CommandRail] write_terminal', e));
	}

	function focusAgent() {
		const xtermEl = document.querySelector('.insight-zone .agent-terminal .xterm-helper-textarea') as HTMLTextAreaElement;
		if (xtermEl) xtermEl.focus();
	}
</script>

<svelte:window onkeydown={(e) => {
	if ((e.ctrlKey || e.metaKey) && e.key === 'i') {
		e.preventDefault();
		focusAgent();
	}
	if ((e.ctrlKey || e.metaKey) && e.key === 'l') {
		e.preventDefault();
		injectContext();
		focusAgent();
	}
}} />

<div class="command-rail">
	<div class="mode-btn-wrapper">
		<button
			class="mode-indicator"
			class:agent={$isAgentMode}
			onclick={() => showDropdown = !showDropdown}
		>
			<span class="mode-dot" class:agent={$isAgentMode}></span>
			{$isAgentMode ? 'Agent' : 'Dev'}
			<span class="mode-chevron">▾</span>
		</button>

		{#if showDropdown}
			<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
			<div class="dropdown-backdrop" onclick={() => showDropdown = false}></div>
			<div class="mode-dropdown">
				<button class="dropdown-item" class:selected={$isDevMode} onclick={() => selectMode('dev')}>
					<span class="dropdown-radio">{$isDevMode ? '◉' : '○'}</span>
					<div class="dropdown-info">
						<span class="dropdown-label">Dev Mode</span>
						<span class="dropdown-desc">Agent edits your files directly. You control git.</span>
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

	<div class="rail-divider"></div>

	<button class="context-btn" onclick={() => { injectContext(); focusAgent(); }} title="Send context to agent (Cmd+L)">
		&#9672; Context
	</button>
	<button class="focus-btn" onclick={focusAgent} title="Focus agent (Cmd+I)">
		AI &#8599;
	</button>
	<button class="terminal-btn" class:active={$terminalVisible} onclick={toggleTerminal} title="Toggle terminal (Ctrl+`)">
		&#9002; Terminal
	</button>
</div>

{#if showGitInit}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="modal-backdrop" onclick={() => showGitInit = false}>
		<div class="modal" onclick={(e) => e.stopPropagation()}>
			<div class="modal-title">Agent Mode requires git</div>
			<div class="modal-text">
				This project isn't a git repository. Initialize git to use Agent Mode?
				<br /><br />
				This will run <code>git init</code> and create an initial commit with all existing files.
			</div>
			<div class="modal-actions">
				<button class="modal-btn cancel" onclick={() => showGitInit = false}>Stay in Dev Mode</button>
				<button class="modal-btn confirm" onclick={initGitAndSwitch} disabled={initializingGit}>
					{initializingGit ? 'Initializing...' : 'Initialize Git'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.command-rail {
		background: var(--color-surface);
		border-top: 1px solid var(--color-border-muted);
		padding: 6px 12px;
		display: flex;
		gap: 4px;
		align-items: center;
	}
	.mode-btn-wrapper {
		position: relative;
		flex-shrink: 0;
	}
	.rail-divider {
		width: 1px;
		height: 16px;
		background: var(--color-border-muted);
		margin: 0 4px;
	}
	.mode-indicator {
		display: flex;
		align-items: center;
		gap: 5px;
		background: none;
		border: none;
		color: var(--color-text-secondary);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		padding: 3px 8px;
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
	.context-btn, .focus-btn, .terminal-btn {
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 600;
		padding: 4px 10px;
		cursor: pointer;
		white-space: nowrap;
	}
	.context-btn:hover, .terminal-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
	}
	.focus-btn {
		color: var(--color-accent);
		border-color: color-mix(in srgb, var(--color-accent) 25%, transparent);
	}
	.focus-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
	}
	.terminal-btn.active {
		color: var(--color-success);
		border-color: color-mix(in srgb, var(--color-success) 30%, transparent);
	}

	.dropdown-backdrop {
		position: fixed;
		inset: 0;
		z-index: 99;
	}
	.mode-dropdown {
		position: absolute;
		bottom: calc(100% + 8px);
		left: 0;
		z-index: 100;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		box-shadow: 0 -4px 16px rgba(0,0,0,0.3);
		padding: 4px;
		min-width: 280px;
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
	.dropdown-item:hover { background: var(--color-overlay); }
	.dropdown-item.selected {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}
	.dropdown-radio { color: var(--color-accent); font-size: 14px; margin-top: 1px; }
	.dropdown-info { display: flex; flex-direction: column; gap: 2px; }
	.dropdown-label { font-size: 13px; font-weight: 600; }
	.dropdown-desc { font-size: 11px; color: var(--color-text-subtle); line-height: 1.4; }

	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}
	.modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 10px;
		padding: 24px;
		max-width: 380px;
		width: 90%;
	}
	.modal-title { font-size: 15px; font-weight: 600; margin-bottom: 12px; }
	.modal-text { font-size: 13px; color: var(--color-text-secondary); line-height: 1.5; margin-bottom: 20px; }
	.modal-actions { display: flex; justify-content: flex-end; gap: 8px; }
	.modal-btn { padding: 7px 16px; border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; }
	.modal-btn.cancel { background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text); }
	.modal-btn.confirm { background: var(--color-warning, #fbbf24); border: none; color: #1a1a1a; }
	.modal-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
