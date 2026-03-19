<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { projectRoot } from '$lib/stores/editor';
	import { activeTheme, getXtermTheme } from '$lib/themes';
	import { activeSessionId, focusAgentTerminal } from '$lib/stores/activeSession';
	import { get } from 'svelte/store';
	import { log } from '$lib/log';

	let { sessionId = '' }: { sessionId: string } = $props();

	let termEl: HTMLDivElement;
	let terminal: any;
	let fitAddon: any;
	let unlistenOutput: (() => void) | null = null;
	let unlistenExit: (() => void) | null = null;
	let resizeObserver: ResizeObserver | null = null;
	let fitTimeout: ReturnType<typeof setTimeout> | null = null;

	function safeFit() {
		if (!fitAddon || !terminal || !termEl) return;
		const { width, height } = termEl.getBoundingClientRect();
		if (width < 10 || height < 10) return;
		try {
			fitAddon.fit();
			invoke('resize_terminal', {
				id: sessionId,
				cols: terminal.cols,
				rows: terminal.rows,
			}).catch(() => { /* PTY closed */ });
		} catch { /* ignore */ }
	}

	function debouncedFit() {
		if (fitTimeout) clearTimeout(fitTimeout);
		fitTimeout = setTimeout(safeFit, 50);
	}

	onMount(async () => {
		const xterm = await import('@xterm/xterm');
		const { FitAddon } = await import('@xterm/addon-fit');
		const { WebLinksAddon } = await import('@xterm/addon-web-links');

		// Import xterm CSS
		await import('@xterm/xterm/css/xterm.css');

		const xtermTheme = getXtermTheme($activeTheme);
		terminal = new xterm.Terminal({
			theme: xtermTheme,
			fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
			fontSize: 13,
			cursorBlink: true,
			allowProposedApi: true,
		});

		fitAddon = new FitAddon();
		terminal.loadAddon(fitAddon);
		terminal.loadAddon(new WebLinksAddon());
		terminal.open(termEl);

		setTimeout(safeFit, 100);

		// Get project root for cwd
		let cwd: string | null = null;
		projectRoot.subscribe(v => cwd = v)();

		// Spawn PTY
		try {
			await invoke('spawn_terminal', { id: sessionId, cwd });
		} catch (e) {
			terminal.writeln(`\r\nFailed to spawn terminal: ${e}`);
			return;
		}

		// Listen for PTY output
		unlistenOutput = await listen<string>(`pty-output-${sessionId}`, (event) => {
			terminal.write(event.payload);
		});

		// Listen for PTY exit
		unlistenExit = await listen(`pty-exit-${sessionId}`, () => {
			terminal.writeln('\r\n[Process exited]');
		});

		// Write terminal input to PTY
		terminal.onData((data: string) => {
			invoke('write_terminal', { id: sessionId, data }).catch(() => { /* PTY closed */ });
		});

		// Cmd+Shift+Enter: send selected terminal text to agent
		terminal.attachCustomKeyEventHandler((e: KeyboardEvent) => {
			const isMac = navigator.platform.toUpperCase().includes('MAC');
			const cmdHeld = isMac ? e.metaKey : e.ctrlKey;
			if (cmdHeld && e.shiftKey && e.code === 'Enter' && e.type === 'keydown') {
				const selection = terminal.getSelection();
				if (selection) {
					const agentId = get(activeSessionId);
					if (agentId) {
						invoke('write_terminal', { id: agentId, data: selection + '\n' }).catch((e) => log.warn('[Terminal] write_terminal to agent', e));
						focusAgentTerminal.update(n => n + 1);
					}
				}
				return false; // prevent xterm from handling
			}
			return true;
		});

		// Handle resize with debounce
		resizeObserver = new ResizeObserver(() => debouncedFit());
		resizeObserver.observe(termEl);
	});

	onDestroy(() => {
		if (fitTimeout) clearTimeout(fitTimeout);
		unlistenOutput?.();
		unlistenExit?.();
		resizeObserver?.disconnect();
		terminal?.dispose();
		invoke('kill_terminal', { id: sessionId }).catch(() => { /* already dead */ });
	});
</script>

<div class="terminal-wrapper" bind:this={termEl}></div>

<style>
	.terminal-wrapper {
		width: 100%;
		height: 100%;
		overflow: hidden;
	}
	.terminal-wrapper :global(.xterm) {
		height: 100%;
		padding: 4px 8px;
	}
	.terminal-wrapper :global(.xterm-viewport) {
		overflow-y: auto !important;
	}
</style>
