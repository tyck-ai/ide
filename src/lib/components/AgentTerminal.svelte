<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { activeTheme, getXtermTheme } from '$lib/themes';

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
		// Skip if container has no dimensions (hidden/transitioning)
		const { width, height } = termEl.getBoundingClientRect();
		if (width < 10 || height < 10) return;

		try {
			fitAddon.fit();
			invoke('resize_terminal', {
				id: sessionId,
				cols: terminal.cols,
				rows: terminal.rows,
			});
		} catch { /* ignore fit errors during transitions */ }
	}

	function debouncedFit() {
		if (fitTimeout) clearTimeout(fitTimeout);
		fitTimeout = setTimeout(safeFit, 50);
	}

	onMount(async () => {
		const xterm = await import('@xterm/xterm');
		const { FitAddon } = await import('@xterm/addon-fit');
		const { WebLinksAddon } = await import('@xterm/addon-web-links');
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

		// Delay initial fit to ensure container is laid out
		setTimeout(safeFit, 100);

		// Replay backlog from Rust ring buffer (restores scrollback on remount)
		try {
			const backlog = await invoke<string>('get_terminal_backlog', { id: sessionId });
			if (backlog) {
				terminal.write(backlog);
			}
		} catch { /* no backlog yet */ }

		// Listen for live PTY output
		unlistenOutput = await listen<string>(`pty-output-${sessionId}`, (event) => {
			terminal.write(event.payload);
		});

		unlistenExit = await listen(`pty-exit-${sessionId}`, () => {
			terminal.writeln('\r\n[Agent process exited]');
		});

		// Write terminal input to PTY
		terminal.onData((data: string) => {
			invoke('write_terminal', { id: sessionId, data });
		});

		// Handle resize with debounce to avoid corruption
		resizeObserver = new ResizeObserver(() => debouncedFit());
		resizeObserver.observe(termEl);
	});

	onDestroy(() => {
		if (fitTimeout) clearTimeout(fitTimeout);
		unlistenOutput?.();
		unlistenExit?.();
		resizeObserver?.disconnect();
		terminal?.dispose();
		// NOTE: we do NOT kill the PTY here — the agent session survives
		// across InsightZone unmount/remount (e.g. mode switches)
	});
</script>

<div class="agent-terminal" bind:this={termEl}></div>

<style>
	.agent-terminal {
		width: 100%;
		height: 100%;
		overflow: hidden;
	}
	.agent-terminal :global(.xterm) {
		height: 100%;
		padding: 4px 8px;
	}
	.agent-terminal :global(.xterm-viewport) {
		overflow-y: auto !important;
	}
</style>
