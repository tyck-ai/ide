<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	interface Props {
		onOpen: () => void;
		onOpenRecent: (path: string) => void;
	}

	let { onOpen, onOpenRecent }: Props = $props();

	let mounted = $state(false);
	// Multi-window mode tracks openWindows in Rust only; no single "last folder" hint.
	const recentFolder: string | undefined = undefined;
	let canvas: HTMLCanvasElement;
	let animFrame: number;

	function folderName(path: string): string {
		return path.split('/').filter(Boolean).pop() || path;
	}

	function shortenPath(path: string): string {
		const home = '/Users/';
		if (path.startsWith(home)) {
			const rest = path.slice(home.length);
			const slash = rest.indexOf('/');
			if (slash !== -1) return '~' + rest.slice(slash);
		}
		return path;
	}

	function onKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'o') {
			e.preventDefault();
			onOpen();
		}
	}

	function initRain() {
		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		const dpr = window.devicePixelRatio || 1;
		let w = window.innerWidth;
		let h = window.innerHeight;

		function resize() {
			w = window.innerWidth;
			h = window.innerHeight;
			canvas.width = w * dpr;
			canvas.height = h * dpr;
			canvas.style.width = w + 'px';
			canvas.style.height = h + 'px';
			ctx!.setTransform(dpr, 0, 0, dpr, 0, 0);
		}
		resize();
		window.addEventListener('resize', resize);

		const fontSize = 20;
		const lineHeight = fontSize * 1.6;
		const cols = Math.ceil(w / fontSize);
		const rows = Math.ceil(h / lineHeight) + 2;

		// Build a static grid of 0s and 1s, scrolling upward
		interface Column {
			chars: string[];
			offset: number;       // fractional y offset in px
			speed: number;        // px per frame
			alpha: number;        // column brightness
		}

		const columns: Column[] = Array.from({ length: cols }, () => ({
			chars: Array.from({ length: rows * 2 }, () => Math.random() < 0.5 ? '0' : '1'),
			offset: Math.random() * lineHeight * rows,
			speed: 0.15 + Math.random() * 0.35,
			alpha: 0.06 + Math.random() * 0.14,
		}));

		function draw() {
			ctx!.clearRect(0, 0, w, h);
			ctx!.fillStyle = '#11111b';
			ctx!.fillRect(0, 0, w, h);

			ctx!.font = `${fontSize}px "SF Mono", "Fira Code", "JetBrains Mono", monospace`;

			for (let i = 0; i < cols; i++) {
				const col = columns[i];
				const x = i * fontSize;

				ctx!.fillStyle = `rgba(137, 180, 250, ${col.alpha})`;

				for (let r = 0; r < col.chars.length; r++) {
					const y = r * lineHeight - col.offset;
					if (y < -lineHeight || y > h + lineHeight) continue;
					ctx!.fillText(col.chars[r], x, y);
				}

				// Scroll downward
				col.offset -= col.speed;

				// Wrap around seamlessly
				if (col.offset < 0) {
					col.offset += lineHeight * rows;
					// Refresh some chars for variety
					for (let r = 0; r < col.chars.length; r++) {
						if (Math.random() < 0.1) {
							col.chars[r] = Math.random() < 0.5 ? '0' : '1';
						}
					}
				}
			}

			animFrame = requestAnimationFrame(draw);
		}

		draw();

		return () => {
			window.removeEventListener('resize', resize);
		};
	}

	let cleanupResize: (() => void) | undefined;

	onMount(() => {
		cleanupResize = initRain();
		requestAnimationFrame(() => { mounted = true; });
	});

	onDestroy(() => {
		if (animFrame) cancelAnimationFrame(animFrame);
		cleanupResize?.();
	});
</script>

<svelte:window onkeydown={onKeydown} />

<div class="welcome" class:mounted>
	<canvas bind:this={canvas} class="rain"></canvas>

	<div class="content">
		<div class="brand">
			<div class="logo">tyck</div>
		</div>

		<div class="actions">
			<button class="open-btn" onclick={onOpen}>
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
				</svg>
				Open Folder
				<span class="shortcut">Cmd+O</span>
			</button>

			{#if recentFolder}
				<button class="recent-btn" onclick={() => onOpenRecent(recentFolder!)}>
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
						<polyline points="15 3 21 3 21 9"/>
						<path d="M21 3l-7 7"/>
						<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
					</svg>
					<span class="recent-name">{folderName(recentFolder)}</span>
					<span class="recent-path">{shortenPath(recentFolder)}</span>
				</button>
			{/if}
		</div>

		<div class="hints">
			<span class="hint">Cmd+O open folder</span>
			<span class="dot"></span>
			<span class="hint">Cmd+, settings</span>
		</div>
	</div>
</div>

<style>
	.welcome {
		height: 100vh;
		width: 100vw;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--color-surface);
		position: relative;
		overflow: hidden;
	}

	.rain {
		position: absolute;
		inset: 0;
		z-index: 0;
	}

	.content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 48px;
		z-index: 1;
		opacity: 0;
		transform: translateY(10px);
		transition: opacity 0.9s ease 0.3s, transform 0.9s ease 0.3s;
	}
	.mounted .content {
		opacity: 1;
		transform: translateY(0);
	}

	/* Brand — same style as FocusZone .logo */
	.brand {
		display: flex;
		flex-direction: column;
		align-items: center;
	}
	.logo {
		font-size: 48px;
		font-weight: 800;
		color: var(--color-accent);
		letter-spacing: 2px;
		user-select: none;
		text-shadow: 0 0 40px color-mix(in srgb, var(--color-accent) 30%, transparent);
	}

	/* Actions */
	.actions {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
	}

	.open-btn {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px 28px;
		background: color-mix(in srgb, var(--color-base) 85%, transparent);
		border: 1px solid var(--color-border-muted);
		border-radius: 10px;
		color: var(--color-text);
		font-size: 14px;
		cursor: pointer;
		backdrop-filter: blur(8px);
		transition: border-color 0.2s, background 0.2s, box-shadow 0.2s;
	}
	.open-btn:hover {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-surface) 90%, transparent);
		box-shadow: 0 0 30px color-mix(in srgb, var(--color-accent) 10%, transparent);
	}
	.shortcut {
		font-size: 11px;
		color: var(--color-text-subtle);
		padding: 2px 6px;
		background: var(--color-surface);
		border-radius: 4px;
		font-family: 'SF Mono', monospace;
	}

	.recent-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px;
		background: none;
		border: 1px solid transparent;
		border-radius: 8px;
		color: var(--color-text-subtle);
		font-size: 12px;
		cursor: pointer;
		transition: color 0.15s, border-color 0.15s, background 0.15s;
	}
	.recent-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border-muted);
		background: color-mix(in srgb, var(--color-base) 80%, transparent);
	}
	.recent-name {
		color: var(--color-text-muted);
		font-weight: 500;
	}
	.recent-btn:hover .recent-name {
		color: var(--color-text);
	}
	.recent-path {
		color: var(--color-border);
		font-family: 'SF Mono', monospace;
		font-size: 11px;
	}

	/* Hints */
	.hints {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.hint {
		font-size: 11px;
		color: var(--color-border);
		font-family: 'SF Mono', monospace;
	}
	.dot {
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: var(--color-border-muted);
	}
</style>
