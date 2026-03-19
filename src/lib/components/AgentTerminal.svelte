<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { activeTheme, getXtermTheme } from '$lib/themes';
	import { focusAgentTerminal, activeSessionId } from '$lib/stores/activeSession';

	let { sessionId = '' }: { sessionId: string } = $props();

	let termEl: HTMLDivElement;
	let terminal: any;
	let fitAddon: any;
	let unlistenOutput: (() => void) | null = null;
	let unlistenExit: (() => void) | null = null;
	let resizeObserver: ResizeObserver | null = null;
	let fitTimeout: ReturnType<typeof setTimeout> | null = null;
	let loaded = $state(false);

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
			}).catch(() => {});
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

		loaded = true;

		// Listen for live PTY output
		unlistenOutput = await listen<string>(`pty-output-${sessionId}`, (event) => {
			terminal.write(event.payload);
		});

		unlistenExit = await listen(`pty-exit-${sessionId}`, () => {
			terminal.writeln('\r\n[Agent process exited]');
		});

		// Write terminal input to PTY
		terminal.onData((data: string) => {
			invoke('write_terminal', { id: sessionId, data }).catch(() => { /* PTY closed */ });
		});

		// Handle resize with debounce to avoid corruption
		resizeObserver = new ResizeObserver(() => debouncedFit());
		resizeObserver.observe(termEl);
	});

	// Focus this terminal when signalled (only if this is the active session)
	const unsubFocus = focusAgentTerminal.subscribe(() => {
		if (terminal && sessionId === $activeSessionId) {
			terminal.focus();
		}
	});

	onDestroy(() => {
		unsubFocus();
		if (fitTimeout) clearTimeout(fitTimeout);
		unlistenOutput?.();
		unlistenExit?.();
		resizeObserver?.disconnect();
		terminal?.dispose();
		// NOTE: we do NOT kill the PTY here — the agent session survives
		// across InsightZone unmount/remount (e.g. mode switches)
	});
</script>

<div class="terminal-wrap">
	{#if !loaded}
		<div class="terminal-loading">
			<div class="loading-dots">
				<span></span><span></span><span></span>
			</div>
			<p class="loading-text">Starting agent...</p>
		</div>
	{/if}
	<div class="agent-terminal" class:invisible={!loaded} bind:this={termEl}></div>
</div>

<style>
	.terminal-wrap {
		width: 100%;
		height: 100%;
		position: relative;
		overflow: hidden;
	}
	.terminal-loading {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 16px;
		cursor: default;
		z-index: 2;
	}
	.loading-dots {
		display: flex;
		gap: 6px;
		align-items: center;
	}
	.loading-dots span {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-text-subtle);
		opacity: 0.4;
		animation: blink 1.2s infinite ease-in-out;
	}
	.loading-dots span:nth-child(2) { animation-delay: 0.2s; }
	.loading-dots span:nth-child(3) { animation-delay: 0.4s; }
	@keyframes blink {
		0%, 80%, 100% { opacity: 0.15; transform: scale(1); }
		40% { opacity: 0.8; transform: scale(1.2); }
	}
	.loading-text {
		font-size: 12px;
		color: var(--color-text-subtle);
		margin: 0;
	}
	.agent-terminal {
		width: 100%;
		height: 100%;
		overflow: hidden;
	}
	.agent-terminal.invisible {
		opacity: 0;
		pointer-events: none;
	}
	.agent-terminal :global(.xterm) {
		height: 100%;
		padding: 4px 8px;
	}
	.agent-terminal :global(.xterm-viewport) {
		overflow-y: auto !important;
	}
</style>
