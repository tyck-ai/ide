<script lang="ts">
	import { git, totalChanges } from '$lib/stores/git';
	import { agentGit } from '$lib/stores/agentGit';
	import { invoke } from '@tauri-apps/api/core';
	import { showBranchSwitcher, showGitView, gitViewTab, gitAgentSessionId } from '$lib/stores/layout';
	import { isAgentMode } from '$lib/stores/settings';
	import { activeSession } from '$lib/stores/agentTerminal';
	import { activeSessionId } from '$lib/stores/activeSession';
	import { sessionReview } from '$lib/stores/sessionReview';
	import { projectRoot } from '$lib/stores/editor';
	import AddRemoteModal from './AddRemoteModal.svelte';

	let syncing = $state(false);
	let pushing = $state(false);
	let pulling = $state(false);
	let hasRemote = $state(false);
	let showAddRemoteModal = $state(false);

	$effect(() => {
		if ($projectRoot) {
			invoke<boolean>('git_has_remote', { path: $projectRoot }).then(v => hasRemote = v).catch(() => hasRemote = false);
		}
	});

	// Agent mode: change count from sessionReview (most accurate — tracks worktree diffs)
	const activeReviewState = $derived($activeSessionId ? $sessionReview.get($activeSessionId) : null);
	const agentFileCount = $derived(activeReviewState?.diffs.length ?? 0);

	// Agent branch display: live from agentGit if available, fallback to session metadata
	const agentBranch = $derived(
		$agentGit.branch && !$agentGit.branch.startsWith('(HEAD detached')
			? $agentGit.branch
			: $activeSession?.branchName ?? ''
	);

	function openBranchSwitcher() {
		showBranchSwitcher.set(true);
	}

	function openGitView() {
		if ($isAgentMode && $activeSessionId) {
			gitViewTab.set('agent');
			gitAgentSessionId.set($activeSessionId);
		} else {
			gitViewTab.set('changes');
		}
		showGitView.set(true);
	}

	async function onPull() {
		pulling = true;
		await git.pull();
		pulling = false;
	}

	async function onPush() {
		if (!hasRemote) {
			showAddRemoteModal = true;
			return;
		}
		pushing = true;
		await git.push();
		pushing = false;
	}

	async function onRemoteAdded() {
		hasRemote = true;
		showAddRemoteModal = false;
		pushing = true;
		await git.push(true);
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
	{#if $isAgentMode && $activeSession}
		<!-- Agent mode: show active session context -->
		<div class="left-section">
			<button class="agent-session-btn" onclick={openGitView} title="Open agent git view">
				<span class="session-dot">●</span>
				<span class="session-name">{$activeSession.label}</span>
			</button>

			{#if agentBranch}
				<div class="agent-branch">
					<span class="branch-icon">⎇</span>
					<span class="branch-name">{agentBranch}</span>
				</div>
			{/if}

			{#if $agentGit.ahead > 0 || $agentGit.behind > 0}
				<div class="sync-status">
					{#if $agentGit.ahead > 0}
						<span class="ahead">↑{$agentGit.ahead}</span>
					{/if}
					{#if $agentGit.behind > 0}
						<span class="behind">↓{$agentGit.behind}</span>
					{/if}
				</div>
			{/if}

			{#if agentFileCount > 0}
				<button class="changes-badge" onclick={openGitView} title="View agent changes">
					<span class="changes-dot">●</span>
					<span class="changes-count">{agentFileCount} file{agentFileCount !== 1 ? 's' : ''}</span>
				</button>
			{/if}
		</div>

		<div class="right-section">
			<div class="right-divider"></div>
			<!-- Main workspace branch (smaller, as context) -->
			<button class="branch-btn secondary" onclick={openBranchSwitcher} title="Switch branch (main workspace)">
				<span class="branch-icon">⎇</span>
				<span class="branch-name">{$git.branch || 'No branch'}</span>
			</button>
		</div>
	{:else}
		<!-- Dev mode: standard git bar -->
		<div class="left-section">
			<button class="git-panel-btn" onclick={openGitView} title="Open git panel">
				<svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
					<path d="M15.698 7.287 8.712.302a1.03 1.03 0 0 0-1.457 0l-1.45 1.45 1.84 1.84a1.223 1.223 0 0 1 1.55 1.56l1.773 1.774a1.224 1.224 0 0 1 1.267 2.025 1.226 1.226 0 0 1-2.002-1.334L8.58 5.963v4.353a1.226 1.226 0 1 1-1.008-.036V5.887a1.226 1.226 0 0 1-.666-1.608L5.093 2.445 .302 7.237a1.03 1.03 0 0 0 0 1.457l6.986 6.986a1.03 1.03 0 0 0 1.457 0l6.953-6.953a1.03 1.03 0 0 0 0-1.44z"/>
				</svg>
				Git
			</button>
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
				<button class="sync-status" onclick={openGitView} title="{$git.ahead} ahead, {$git.behind} behind — open git">
					{#if $git.ahead > 0}
						<span class="ahead">↑{$git.ahead}</span>
					{/if}
					{#if $git.behind > 0}
						<span class="behind">↓{$git.behind}</span>
					{/if}
				</button>
			{/if}

			{#if $totalChanges > 0}
				<button class="changes-badge" onclick={openGitView} title="View changes">
					<span class="changes-dot">●</span>
					<span class="changes-count">{$totalChanges} change{$totalChanges !== 1 ? 's' : ''}</span>
				</button>
			{/if}
		</div>

		<div class="right-section">
			<div class="right-divider"></div>
			<button
				class="action-btn"
				onclick={onFetch}
				title="Fetch remote updates"
			>
				Fetch
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
				disabled={pushing || ($git.ahead === 0 && hasRemote)}
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
	{/if}
</div>

{#if showAddRemoteModal}
	<AddRemoteModal onAdded={onRemoteAdded} onClose={() => showAddRemoteModal = false} />
{/if}

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

	/* Agent mode styles */
	.agent-session-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 12px;
		font-weight: 600;
		cursor: pointer;
		padding: 2px 6px;
		border-radius: 4px;
	}

	.agent-session-btn:hover {
		background: var(--color-overlay);
	}

	.session-dot {
		font-size: 7px;
		color: var(--color-success);
	}

	.session-name {
		font-weight: 600;
	}

	.agent-branch {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: var(--color-text-muted);
	}

	.git-panel-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		background: none;
		border: 1px solid var(--color-border-muted);
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		padding: 2px 8px;
		border-radius: 4px;
	}
	.git-panel-btn:hover {
		background: var(--color-overlay);
		color: var(--color-text);
		border-color: var(--color-border);
	}

	/* Dev mode styles */
	.branch-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		background: none;
		border: 1px solid var(--color-border-muted);
		color: var(--color-text);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		padding: 2px 8px;
		border-radius: 4px;
	}

	.branch-btn:hover {
		background: var(--color-overlay);
		border-color: var(--color-border);
	}

	.branch-btn.secondary {
		color: var(--color-text-muted);
		font-size: 11px;
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
		background: none;
		border: 1px solid var(--color-border-muted);
		padding: 2px 8px;
		border-radius: 4px;
		cursor: pointer;
	}
	.sync-status:hover {
		background: var(--color-overlay);
		border-color: var(--color-border);
	}

	.ahead { color: var(--color-success); }
	.behind { color: var(--color-warning); }

	.changes-badge {
		display: flex;
		align-items: center;
		gap: 4px;
		background: none;
		border: 1px solid color-mix(in srgb, var(--color-warning) 40%, transparent);
		color: var(--color-warning);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		padding: 2px 8px;
		border-radius: 4px;
	}

	.changes-badge:hover {
		background: color-mix(in srgb, var(--color-warning) 8%, transparent);
		border-color: var(--color-warning);
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
