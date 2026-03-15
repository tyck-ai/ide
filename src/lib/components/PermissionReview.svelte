<script lang="ts">
	import { pendingInstall } from '$lib/stores/layout';
	import { tapp } from '$lib/stores/tapp';
	import { invoke } from '@tauri-apps/api/core';

	let installing = $state(false);
	let error = $state<string | null>(null);

	const riskLevels: Record<string, 'low' | 'medium' | 'high'> = {
		'fs:read': 'low',
		'fs:write': 'medium',
		'fs:system': 'high',
		'network:fetch': 'medium',
		'network:unrestricted': 'high',
		'storage:session': 'low',
		'storage:persistent': 'low',
		'agent:inject': 'low',
		'agent:tools': 'medium',
		'agent:hooks': 'high',
		'agent:spawn': 'medium',
	};

	const permissionNames: Record<string, string> = {
		'fs:read': 'Read files in project',
		'fs:write': 'Write files in project',
		'fs:system': 'Access system files',
		'network:fetch': 'Network access',
		'network:unrestricted': 'Unrestricted network',
		'storage:session': 'Session storage',
		'storage:persistent': 'Persistent storage',
		'agent:inject': 'Inject context to AI',
		'agent:tools': 'Register AI tools',
		'agent:hooks': 'Register AI hooks',
		'agent:spawn': 'Spawn AI agents',
	};

	const riskLabels: Record<string, string> = {
		low: 'low risk',
		medium: 'medium risk',
		high: 'high risk',
	};

	function getRisk(perm: string): 'low' | 'medium' | 'high' {
		return riskLevels[perm] || 'medium';
	}

	function getPermissionName(perm: string): string {
		return permissionNames[perm] || perm;
	}

	function getRiskIcon(risk: 'low' | 'medium' | 'high'): string {
		switch (risk) {
			case 'low': return '✓';
			case 'medium': return '⚠';
			case 'high': return '⚠';
		}
	}

	function close() {
		pendingInstall.set(null);
		error = null;
	}

	async function confirmInstall() {
		const pending = $pendingInstall;
		if (!pending || installing) return;

		installing = true;
		error = null;

		try {
			if (pending.source === 'file' && pending.path) {
				await tapp.install(pending.path);
			} else if (pending.source === 'store' && pending.appId) {
				await tapp.installFromStore(pending.appId);
			}
			close();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		}

		installing = false;
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			close();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			close();
		} else if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
			confirmInstall();
		}
	}

	const appName = $derived(() => {
		const pending = $pendingInstall;
		if (!pending) return 'Unknown App';
		if (pending.listing) return pending.listing.name;
		if (pending.path) {
			const parts = pending.path.split('/');
			return parts[parts.length - 2] || 'Local App';
		}
		return 'Unknown App';
	});

	const permissions = $derived(() => {
		const pending = $pendingInstall;
		if (!pending) return [];
		if (pending.listing) return pending.listing.permissions;
		return [];
	});

	const networkHosts = $derived(() => {
		const pending = $pendingInstall;
		if (!pending?.listing) return [];
		return [];
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={handleBackdropClick}>
	<div class="modal">
		<div class="modal-header">
			<span class="modal-title">Install "{appName()}"?</span>
			<button class="close-btn" onclick={close}>×</button>
		</div>

		<div class="modal-content">
			{#if permissions().length > 0}
				<p class="section-intro">This app requests the following permissions:</p>
				
				<div class="permissions-list">
					{#each permissions() as perm}
						{@const risk = getRisk(perm)}
						<div class="permission-row" class:low={risk === 'low'} class:medium={risk === 'medium'} class:high={risk === 'high'}>
							<span class="permission-icon">{getRiskIcon(risk)}</span>
							<span class="permission-name">{getPermissionName(perm)}</span>
							<span class="permission-risk">({riskLabels[risk]})</span>
						</div>
					{/each}
				</div>

				{#if networkHosts().length > 0}
					<div class="network-hosts">
						<span class="network-label">Network restricted to:</span>
						<span class="network-list">{networkHosts().join(', ')}</span>
					</div>
				{/if}
			{:else if $pendingInstall?.source === 'file'}
				<p class="section-intro">Install this app from a local manifest file?</p>
				<div class="file-path">
					<span class="file-path-label">Path:</span>
					<span class="file-path-value">{$pendingInstall.path}</span>
				</div>
			{:else}
				<p class="section-intro">This app requires no special permissions.</p>
			{/if}

			{#if error}
				<div class="error-message">
					{error}
				</div>
			{/if}
		</div>

		<div class="modal-footer">
			<button class="cancel-btn" onclick={close} disabled={installing}>
				Cancel
			</button>
			<button class="install-btn" onclick={confirmInstall} disabled={installing}>
				{installing ? 'Installing...' : 'Install'}
			</button>
		</div>
	</div>
</div>

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1001;
	}

	.modal {
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 12px;
		width: 440px;
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border-muted);
	}

	.modal-title {
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
	}

	.close-btn {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 20px;
		cursor: pointer;
		padding: 0;
		line-height: 1;
	}

	.close-btn:hover {
		color: var(--color-text);
	}

	.modal-content {
		padding: 20px;
		flex: 1;
		overflow-y: auto;
	}

	.section-intro {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0 0 16px;
	}

	.permissions-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.permission-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		border-radius: 8px;
	}

	.permission-row.low {
		background: color-mix(in srgb, var(--color-success) 8%, transparent);
	}

	.permission-row.medium {
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
	}

	.permission-row.high {
		background: color-mix(in srgb, var(--color-error) 10%, transparent);
	}

	.permission-icon {
		font-size: 12px;
		flex-shrink: 0;
	}

	.permission-row.low .permission-icon {
		color: var(--color-success);
	}

	.permission-row.medium .permission-icon,
	.permission-row.high .permission-icon {
		color: var(--color-warning);
	}

	.permission-name {
		flex: 1;
		font-size: 13px;
		color: var(--color-text);
	}

	.permission-risk {
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.network-hosts {
		margin-top: 16px;
		padding: 12px;
		background: var(--color-surface);
		border-radius: 8px;
	}

	.network-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-subtle);
		display: block;
		margin-bottom: 4px;
	}

	.network-list {
		font-size: 12px;
		color: var(--color-text-muted);
		font-family: 'SF Mono', 'Fira Code', monospace;
	}

	.file-path {
		padding: 12px;
		background: var(--color-surface);
		border-radius: 8px;
	}

	.file-path-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		display: block;
		margin-bottom: 4px;
	}

	.file-path-value {
		font-size: 12px;
		color: var(--color-text);
		font-family: 'SF Mono', 'Fira Code', monospace;
		word-break: break-all;
	}

	.error-message {
		margin-top: 16px;
		padding: 10px 12px;
		background: color-mix(in srgb, var(--color-error) 10%, transparent);
		border: 1px solid var(--color-error);
		border-radius: 8px;
		color: var(--color-error);
		font-size: 12px;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding: 16px 20px;
		border-top: 1px solid var(--color-border-muted);
	}

	.cancel-btn, .install-btn {
		padding: 8px 18px;
		font-size: 13px;
		font-weight: 600;
		border-radius: 8px;
		cursor: pointer;
	}

	.cancel-btn {
		background: none;
		border: 1px solid var(--color-border-muted);
		color: var(--color-text-muted);
	}

	.cancel-btn:hover:not(:disabled) {
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.install-btn {
		background: var(--color-accent);
		border: none;
		color: var(--color-base);
	}

	.install-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.cancel-btn:disabled,
	.install-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
