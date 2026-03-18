<script lang="ts">
	import { lspMissingServers, dismissedLspNotifications } from '$lib/stores/lsp';

	function dismiss(language: string) {
		lspMissingServers.update((list) => list.filter((s) => s.language !== language));
		dismissedLspNotifications.update((set) => {
			set.add(language);
			return set;
		});
	}

	function dismissAll() {
		lspMissingServers.update((list) => {
			dismissedLspNotifications.update((set) => {
				for (const s of list) set.add(s.language);
				return set;
			});
			return [];
		});
	}
</script>

{#if $lspMissingServers.length > 0}
	<div class="lsp-notification">
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
					<button class="dismiss-btn" onclick={() => dismiss(server.language)} title="Dismiss">✕</button>
				</div>
			{/each}
		</div>
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
