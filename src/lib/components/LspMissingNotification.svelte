<script lang="ts">
	import { lspMissingServers, dismissedLspNotifications } from '$lib/stores/lsp';
	import { lspStatuses, lspClientManager } from '$lib/lsp/LspClientManager';
	import { sendCommandToTerminal } from '$lib/stores/terminal';
	import { updateSettings } from '$lib/stores/settings';
	import { projectRoot } from '$lib/stores/editor';
	import { log } from '$lib/log';

	let running = $state<string | null>(null);
	let retrying = $state<string | null>(null);

	const erroredServers = $derived(
		[...$lspStatuses.values()].filter((s) => s.state === 'error'),
	);
	const hasContent = $derived($lspMissingServers.length > 0 || erroredServers.length > 0);

	function dismiss(language: string) {
		lspMissingServers.update((list) => list.filter((s) => s.language !== language));
		dismissedLspNotifications.update((set) => {
			const next = new Set(set);
			next.add(language);
			updateSettings({ lspDismissed: [...next] }).catch((e) => log.warn('[LspMissingNotification] updateSettings', e));
			return next;
		});
	}

	function dismissAll() {
		lspMissingServers.update((list) => {
			dismissedLspNotifications.update((set) => {
				const next = new Set(set);
				for (const s of list) next.add(s.language);
				updateSettings({ lspDismissed: [...next] }).catch((e) => log.warn('[LspMissingNotification] updateSettings', e));
				return next;
			});
			return [];
		});
	}

	async function runInstall(server: { language: string; installHint: string }) {
		running = server.language;
		await sendCommandToTerminal(server.installHint);
		running = null;
		dismiss(server.language);
	}

	async function retry(language: string) {
		const root = $projectRoot;
		if (!root || retrying === language) return;
		retrying = language;
		await lspClientManager.stop(language);
		await lspClientManager.getOrStart(language, root).catch((e) => log.warn('[LspMissingNotification] getOrStart', e));
		retrying = null;
	}
</script>

{#if hasContent}
	<div class="lsp-notification">
		{#if $lspMissingServers.length > 0}
			<div class="notification-header">
				<span class="notification-icon">⚠</span>
				<span class="notification-title">
					{$lspMissingServers.length === 1
						? '1 language server missing'
						: `${$lspMissingServers.length} language servers missing`}
				</span>
				<button class="dismiss-all-btn" onclick={dismissAll}>Dismiss all</button>
			</div>
			<div class="server-list">
				{#each $lspMissingServers as server (server.language)}
					<div class="server-row">
						<div class="server-info">
							<span class="server-name">{server.displayName}</span>
							<code class="install-hint">{server.installHint}</code>
						</div>
						<div class="server-actions">
							<button
								class="run-btn"
								onclick={() => runInstall(server)}
								disabled={running === server.language}
								title="Run install command in terminal"
							>
								{running === server.language ? '...' : 'Install'}
							</button>
							<button class="dismiss-btn" onclick={() => dismiss(server.language)} title="Dismiss">✕</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}

		{#if erroredServers.length > 0}
			<div class="notification-header error-header" class:with-border={$lspMissingServers.length > 0}>
				<span class="notification-icon error-icon">✕</span>
				<span class="notification-title error-title">
					{erroredServers.length === 1
						? '1 language server error'
						: `${erroredServers.length} language server errors`}
				</span>
			</div>
			<div class="server-list">
				{#each erroredServers as server (server.language)}
					<div class="server-row">
						<div class="server-info">
							<span class="server-name">{server.displayName}</span>
							{#if server.error}
								<code class="install-hint error-detail">{server.error}</code>
							{/if}
						</div>
						<div class="server-actions">
							<button
								class="run-btn retry-btn"
								onclick={() => retry(server.language)}
								disabled={retrying === server.language}
								title="Restart language server"
							>
								{retrying === server.language ? '...' : 'Retry'}
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
{/if}

<style>
	.lsp-notification {
		position: fixed;
		bottom: 36px; /* above GitStatusBar */
		right: 12px;
		width: 380px;
		background: var(--color-surface);
		border: 1px solid color-mix(in srgb, var(--color-warning) 40%, var(--color-border));
		border-radius: 8px;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
		z-index: 500;
		overflow: hidden;
	}

	.notification-header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		background: color-mix(in srgb, var(--color-warning) 8%, var(--color-surface));
		border-bottom: 1px solid color-mix(in srgb, var(--color-warning) 20%, var(--color-border-muted));
	}

	.notification-icon {
		font-size: 13px;
		color: var(--color-warning);
		flex-shrink: 0;
	}

	.notification-title {
		flex: 1;
		font-size: 12px;
		font-weight: 600;
		color: var(--color-warning);
	}

	.dismiss-all-btn {
		background: none;
		border: none;
		font-size: 11px;
		color: var(--color-text-subtle);
		cursor: pointer;
		padding: 2px 6px;
		border-radius: 4px;
		flex-shrink: 0;
	}

	.dismiss-all-btn:hover {
		color: var(--color-text);
		background: var(--color-overlay);
	}

	.server-list {
		display: flex;
		flex-direction: column;
	}

	.server-row {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-muted);
	}

	.server-row:last-child {
		border-bottom: none;
	}

	.server-info {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}

	.server-name {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
	}

	.install-hint {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 10px;
		color: var(--color-text-subtle);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.server-actions {
		display: flex;
		align-items: center;
		gap: 4px;
		flex-shrink: 0;
	}

	.run-btn {
		background: var(--color-accent);
		border: none;
		border-radius: 4px;
		color: var(--color-base);
		font-size: 10px;
		font-weight: 700;
		padding: 3px 8px;
		cursor: pointer;
		flex-shrink: 0;
	}

	.run-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.run-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.retry-btn {
		background: color-mix(in srgb, var(--color-error) 80%, black);
	}

	.error-header {
		background: color-mix(in srgb, var(--color-error) 8%, var(--color-surface));
		border-bottom-color: color-mix(in srgb, var(--color-error) 20%, var(--color-border-muted));
	}

	.error-header.with-border {
		border-top: 1px solid var(--color-border-muted);
	}

	.error-icon {
		color: var(--color-error);
	}

	.error-title {
		color: var(--color-error);
	}

	.error-detail {
		color: var(--color-error);
		opacity: 0.8;
	}

	.dismiss-btn {
		background: none;
		border: none;
		font-size: 11px;
		color: var(--color-text-subtle);
		cursor: pointer;
		padding: 2px 5px;
		border-radius: 3px;
		flex-shrink: 0;
		line-height: 1;
	}

	.dismiss-btn:hover {
		color: var(--color-text);
		background: var(--color-overlay);
	}
</style>
