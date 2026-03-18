<script lang="ts">
	import { onMount } from 'svelte';
	import { tapp, installedApps, runningApps, recentApps, enabledApps, type AppInfo } from '$lib/stores/tapp';
	import { showAppLauncher } from '$lib/stores/layout';

	let searchQuery = $state('');
	let selectedIndex = $state(0);
	let launching = $state<string | null>(null);
	let error = $state<string | null>(null);
	let inputEl: HTMLInputElement;

	const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().indexOf('MAC') >= 0;
	const shortcutHint = isMac ? '⌘⇧A' : 'Ctrl+Shift+A';

	onMount(() => {
		tapp.refresh();
		inputEl?.focus();
	});

	const filteredApps = $derived(() => {
		const query = searchQuery.toLowerCase();
		const enabled = $enabledApps;
		if (!query) return enabled;
		return enabled.filter(app => 
			app.name.toLowerCase().includes(query) ||
			app.id.toLowerCase().includes(query) ||
			(app.description?.toLowerCase().includes(query))
		);
	});

	const sections = $derived(() => {
		const all = filteredApps();
		const running = all.filter(a => a.running);
		const recent = searchQuery ? [] : $recentApps.filter(a => !a.running && a.enabled);
		const other = all.filter(a => !a.running && !recent.some(r => r.id === a.id));
		
		const result: { title: string; apps: AppInfo[] }[] = [];
		if (running.length > 0) result.push({ title: 'Running', apps: running });
		if (recent.length > 0) result.push({ title: 'Recent', apps: recent });
		if (other.length > 0) result.push({ title: 'All Apps', apps: other });
		return result;
	});

	const flatApps = $derived(() => {
		return sections().flatMap(s => s.apps);
	});

	function close() {
		showAppLauncher.set(false);
	}

	function handleKeydown(e: KeyboardEvent) {
		const apps = flatApps();
		
		if (e.key === 'Escape') {
			close();
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, apps.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			const app = apps[selectedIndex];
			if (app) toggleApp(app);
		}
	}

	async function toggleApp(app: AppInfo) {
		if (launching) return;
		launching = app.id;
		error = null;
		
		try {
			if (app.running) {
				await tapp.stop(app.id);
			} else {
				await tapp.start(app.id);
				tapp.trackRecent(app.id);
				close();
			}
		} catch (e) {
			const msg = e instanceof Error ? e.message : String(e);
			error = `Failed to ${app.running ? 'stop' : 'start'} "${app.name}": ${msg}`;
			console.error('Failed to toggle app:', e);
		}
		
		launching = null;
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			close();
		}
	}

	$effect(() => {
		const apps = flatApps();
		if (selectedIndex >= apps.length) {
			selectedIndex = Math.max(0, apps.length - 1);
		}
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={handleBackdropClick}>
	<div class="modal">
		<div class="modal-header">
			<input
				bind:this={inputEl}
				bind:value={searchQuery}
				type="text"
				placeholder="Search apps..."
				class="search-input"
			/>
			<span class="shortcut-hint">{shortcutHint}</span>
		</div>

		{#if error}
			<div class="error-banner">
				<span class="error-text">{error}</span>
				<button class="error-dismiss" onclick={() => error = null}>×</button>
			</div>
		{/if}

		<div class="app-list">
			{#each sections() as section, sectionIdx}
				{@const sectionStartIndex = sections()
					.slice(0, sectionIdx)
					.reduce((acc, s) => acc + s.apps.length, 0)}
				<div class="section">
					<div class="section-header">{section.title}</div>
					{#each section.apps as app, appIdx}
						{@const globalIndex = sectionStartIndex + appIdx}
						<button
							class="app-item"
							class:selected={selectedIndex === globalIndex}
							class:running={app.running}
							onclick={() => toggleApp(app)}
							onmouseenter={() => selectedIndex = globalIndex}
							disabled={launching === app.id}
						>
							<span class="app-indicator">{app.running ? '●' : '○'}</span>
							<span class="app-name">{app.name}</span>
							{#if app.description}
								<span class="app-desc">{app.description}</span>
							{/if}
							<span class="app-action">
								{#if launching === app.id}
									...
								{:else if app.running}
									Stop
								{:else}
									Open
								{/if}
							</span>
						</button>
					{/each}
				</div>
			{/each}

			{#if flatApps().length === 0}
				<div class="empty-state">
					{#if searchQuery}
						No apps match "{searchQuery}"
					{:else}
						No apps installed
					{/if}
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
		z-index: 1000;
	}

	.modal {
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 12px;
		width: 480px;
		max-height: 60vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		padding: 12px 16px;
		border-bottom: 1px solid var(--color-border-muted);
		gap: 12px;
	}

	.search-input {
		flex: 1;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		padding: 10px 14px;
		color: var(--color-text);
		font-size: 14px;
		outline: none;
	}

	.search-input:focus {
		border-color: var(--color-accent);
	}

	.search-input::placeholder {
		color: var(--color-text-subtle);
	}

	.shortcut-hint {
		font-size: 11px;
		color: var(--color-text-subtle);
		background: var(--color-surface);
		padding: 4px 8px;
		border-radius: 4px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		flex-shrink: 0;
	}

	.app-list {
		flex: 1;
		overflow-y: auto;
		padding: 8px 0;
	}

	.section {
		margin-bottom: 4px;
	}

	.section-header {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		padding: 8px 16px 4px;
	}

	.app-item {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 10px 16px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 13px;
		cursor: pointer;
		text-align: left;
	}

	.app-item:hover,
	.app-item.selected {
		background: var(--color-overlay);
	}

	.app-item.running {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.app-item.running.selected {
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
	}

	.app-item:disabled {
		opacity: 0.7;
		cursor: default;
	}

	.app-indicator {
		font-size: 10px;
		color: var(--color-text-subtle);
		flex-shrink: 0;
	}

	.app-item.running .app-indicator {
		color: var(--color-success);
	}

	.app-name {
		font-weight: 500;
		flex-shrink: 0;
	}

	.app-desc {
		flex: 1;
		color: var(--color-text-subtle);
		font-size: 12px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.app-action {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-accent);
		padding: 3px 10px;
		border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
		border-radius: 5px;
		flex-shrink: 0;
	}

	.app-item.running .app-action {
		color: var(--color-text-muted);
		border-color: var(--color-border-muted);
	}

	.empty-state {
		padding: 32px 16px;
		text-align: center;
		color: var(--color-text-subtle);
		font-size: 13px;
	}

	.error-banner {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		margin: 8px 12px;
		padding: 10px 12px;
		background: color-mix(in srgb, var(--color-error) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-error) 30%, transparent);
		border-radius: 8px;
	}

	.error-text {
		flex: 1;
		font-size: 12px;
		color: var(--color-error);
		line-height: 1.4;
		word-break: break-word;
	}

	.error-dismiss {
		background: none;
		border: none;
		color: var(--color-error);
		font-size: 16px;
		cursor: pointer;
		padding: 0;
		line-height: 1;
		opacity: 0.7;
	}

	.error-dismiss:hover {
		opacity: 1;
	}
</style>
