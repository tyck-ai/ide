<script lang="ts">
	import { git, totalChanges } from '$lib/stores/git';
	import { showBranchSwitcher, showGitView, gitViewTab } from '$lib/stores/layout';
	import { lspStatuses } from '$lib/lsp/LspClientManager';

	let syncing = $state(false);
	let pushing = $state(false);
	let pulling = $state(false);

	function openBranchSwitcher() {
		showBranchSwitcher.set(true);
	}

	function openChanges() {
		gitViewTab.set('changes');
		showGitView.set(true);
	}

	async function onPull() {
		pulling = true;
		await git.pull();
		pulling = false;
	}

	async function onPush() {
		pushing = true;
		await git.push();
		pushing = false;
	}

	async function onSync() {
		syncing = true;
		await git.pull();
		await git.push();
		syncing = false;
	}

	async function onFetch() {
		await git.fetch();
	}
</script>

<div class="git-status-bar">
	<div class="left-section">
		<button class="branch-btn" onclick={openBranchSwitcher} title="Switch branch">
			<span class="branch-icon">⎇</span>
			<span class="branch-name">{
				!$git.branch ? 'No branch' :
				$git.branch.startsWith('(HEAD detached') ? 'Detached HEAD' :
				$git.branch
			}</span>
			<span class="dropdown-arrow">▾</span>
		</button>

		{#if $git.ahead > 0 || $git.behind > 0}
			<div class="sync-status" title="{$git.ahead} ahead, {$git.behind} behind">
				{#if $git.ahead > 0}
					<span class="ahead">↑{$git.ahead}</span>
				{/if}
				{#if $git.behind > 0}
					<span class="behind">↓{$git.behind}</span>
				{/if}
			</div>
		{/if}

		{#if $totalChanges > 0}
			<button class="changes-badge" onclick={openChanges} title="View changes">
				<span class="changes-dot">●</span>
				<span class="changes-count">{$totalChanges} change{$totalChanges !== 1 ? 's' : ''}</span>
			</button>
		{/if}
	</div>

	<div class="right-section">
		{#each [...$lspStatuses.values()] as status (status.language)}
			<span
				class="lsp-indicator"
				class:lsp-running={status.state === 'running'}
				class:lsp-starting={status.state === 'starting'}
				class:lsp-error={status.state === 'error'}
				title="{status.displayName}: {status.state}"
			>
				<span class="lsp-dot"></span>
				<span class="lsp-lang">{status.language}</span>
			</span>
		{/each}
		<div class="right-divider"></div>
		<button
			class="action-btn"
			onclick={onFetch}
			title="Fetch remote updates"
		>
			⟳
		</button>
		<button 
			class="action-btn" 
			onclick={onPull} 
			disabled={pulling || $git.behind === 0}
			title="Pull from remote"
		>
			{pulling ? '...' : '↓ Pull'}
		</button>
		<button 
			class="action-btn" 
			onclick={onPush} 
			disabled={pushing || $git.ahead === 0}
			title="Push to remote"
		>
			{pushing ? '...' : '↑ Push'}
		</button>
		<button 
			class="action-btn sync-btn" 
			onclick={onSync} 
			disabled={syncing}
			title="Sync (pull then push)"
		>
			{syncing ? '...' : '⟳ Sync'}
		</button>
	</div>
</div>

<style>
	.git-status-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		background: var(--color-surface);
		border-top: 1px solid var(--color-border-muted);
		padding: 4px 12px;
		height: 28px;
		font-size: 12px;
	}

	.left-section {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.right-section {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.right-divider {
		width: 1px;
		height: 14px;
		background: var(--color-border-muted);
		margin: 0 4px;
	}

	.lsp-indicator {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 1px 6px;
		border-radius: 3px;
		font-size: 10px;
		font-weight: 500;
		color: var(--color-text-subtle);
		cursor: default;
		user-select: none;
	}

	.lsp-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--color-border);
		flex-shrink: 0;
	}

	.lsp-indicator.lsp-running .lsp-dot {
		background: var(--color-success);
	}

	.lsp-indicator.lsp-starting .lsp-dot {
		background: var(--color-warning);
		animation: pulse 1.2s ease-in-out infinite;
	}

	.lsp-indicator.lsp-error .lsp-dot {
		background: var(--color-error);
	}

	.lsp-lang {
		text-transform: uppercase;
		letter-spacing: 0.3px;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.3; }
	}

	.branch-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		padding: 2px 6px;
		border-radius: 4px;
	}

	.branch-btn:hover {
		background: var(--color-overlay);
	}

	.branch-icon {
		color: var(--color-accent);
		font-size: 14px;
	}

	.branch-name {
		font-weight: 500;
	}

	.dropdown-arrow {
		font-size: 10px;
		color: var(--color-text-subtle);
	}

	.sync-status {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		font-weight: 500;
	}

	.ahead {
		color: var(--color-success);
	}

	.behind {
		color: var(--color-warning);
	}

	.changes-badge {
		display: flex;
		align-items: center;
		gap: 4px;
		background: none;
		border: none;
		color: var(--color-warning);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		padding: 2px 8px;
		border-radius: 4px;
	}

	.changes-badge:hover {
		background: var(--color-overlay);
	}

	.changes-dot {
		font-size: 8px;
	}

	.action-btn {
		background: none;
		border: 1px solid var(--color-border-muted);
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		padding: 2px 8px;
		border-radius: 4px;
	}

	.action-btn:hover:not(:disabled) {
		background: var(--color-overlay);
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.action-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.sync-btn {
		color: var(--color-accent);
		border-color: color-mix(in srgb, var(--color-accent) 25%, transparent);
	}

	.sync-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
	}
</style>
