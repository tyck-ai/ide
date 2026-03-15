<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { activeFilePath, selection, cursorLine } from '$lib/stores/editor';
	import { activeTerminalId, terminalVisible, addTerminal, toggleTerminal } from '$lib/stores/terminal';
	import { activeSessionId } from '$lib/stores/agentTerminal';

	let input = $state('');
	let inputEl: HTMLInputElement;

	async function send() {
		const msg = input.trim();
		if (!msg) return;

		terminalVisible.set(true);

		let termId = $activeTerminalId;
		if (!termId) {
			const id = crypto.randomUUID();
			addTerminal(id, 'zsh');
			await new Promise(r => setTimeout(r, 500));
			termId = id;
		}

		input = '';
		await invoke('write_terminal', { id: termId, data: msg + '\n' });
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

		invoke('write_terminal', { id: termId, data: ctx + ' ' });
	}

	function focusAgent() {
		// Focus the agent terminal in InsightZone
		const xtermEl = document.querySelector('.insight-zone .agent-terminal .xterm-helper-textarea') as HTMLTextAreaElement;
		if (xtermEl) xtermEl.focus();
	}

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			send();
		}
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
	<div class="rail-left">
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

	<div class="input-wrapper">
		<span class="prompt-icon">$</span>
		<input
			bind:this={inputEl}
			bind:value={input}
			onkeydown={onKeydown}
			placeholder="Run a shell command..."
		/>
		<button
			class="send-btn"
			onclick={send}
			disabled={!input.trim()}
		>&#8629;</button>
	</div>
</div>

<style>
	.command-rail {
		background: var(--color-surface);
		border-top: 1px solid var(--color-border-muted);
		padding: 6px 12px;
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.rail-left {
		display: flex;
		gap: 4px;
		flex-shrink: 0;
	}
	.context-btn, .focus-btn {
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
	.context-btn:hover, .focus-btn:hover {
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
	.terminal-btn {
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
	.terminal-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
	}
	.terminal-btn.active {
		color: var(--color-success);
		border-color: color-mix(in srgb, var(--color-success) 30%, transparent);
	}
	.input-wrapper {
		display: flex;
		align-items: center;
		gap: 8px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		padding: 6px 12px;
		flex: 1;
	}
	.input-wrapper:focus-within {
		border-color: var(--color-accent);
	}
	.prompt-icon {
		color: var(--color-success);
		font-size: 14px;
		font-weight: bold;
		font-family: 'SF Mono', 'Fira Code', monospace;
		flex-shrink: 0;
	}
	input {
		flex: 1;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 14px;
		font-family: inherit;
		outline: none;
	}
	input::placeholder {
		color: var(--color-text-subtle);
	}
	.send-btn {
		background: var(--color-success);
		color: var(--color-base);
		border: none;
		border-radius: 4px;
		padding: 4px 10px;
		font-size: 14px;
		font-weight: bold;
		cursor: pointer;
		flex-shrink: 0;
	}
	.send-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}
	.send-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}
</style>
