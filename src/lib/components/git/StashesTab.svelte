<script lang="ts">
	import { onMount } from 'svelte';
	import { git, type GitStash } from '$lib/stores/git';

	let selectedStash = $state<GitStash | null>(null);
	let stashMessage = $state('');
	let creating = $state(false);
	let applying = $state(false);
	let popping = $state(false);
	let dropping = $state(false);

	onMount(() => {
		git.refreshStashes();
	});

	function selectStash(stash: GitStash) {
		selectedStash = stash;
	}

	async function createStash() {
		creating = true;
		const message = stashMessage.trim() || undefined;
		await git.stashCreate(message);
		creating = false;
		stashMessage = '';
	}

	async function applyStash(stash: GitStash) {
		applying = true;
		await git.stashApply(stash.index);
		applying = false;
	}

	async function popStash(stash: GitStash) {
		popping = true;
		await git.stashPop(stash.index);
		popping = false;
		if (selectedStash?.index === stash.index) {
			selectedStash = null;
		}
	}

	async function dropStash(stash: GitStash) {
		if (!confirm(`Drop stash "${stash.message}"?`)) return;
		
		dropping = true;
		await git.stashDrop(stash.index);
		dropping = false;
		
		if (selectedStash?.index === stash.index) {
			selectedStash = null;
		}
	}
</script>

<div class="stashes-tab">
	<div class="stash-list">
		<div class="list-header">
			<span>Stashes</span>
		</div>

		<div class="list-content">
			{#if $git.stashes.length > 0}
				{#each $git.stashes as stash (stash.index)}
					<button 
						class="stash-item"
						class:selected={selectedStash?.index === stash.index}
						onclick={() => selectStash(stash)}
					>
						<div class="stash-indicator">○</div>
						<div class="stash-info">
							<div class="stash-ref">stash@{`{${stash.index}}`}</div>
							<div class="stash-message">{stash.message}</div>
							<div class="stash-meta">
								{#if stash.branch}
									<span class="stash-branch">{stash.branch}</span>
								{/if}
								<span class="stash-date">{stash.date}</span>
							</div>
						</div>
					</button>
				{/each}
			{:else}
				<div class="empty-state">
					<span class="empty-text">No stashes</span>
					<span class="empty-hint">Stash your changes to save them temporarily</span>
				</div>
			{/if}
		</div>

		<div class="create-section">
			<div class="section-header">Create Stash</div>
			<div class="create-row">
				<input
					bind:value={stashMessage}
					type="text"
					placeholder="Optional message..."
					class="create-input"
					onkeydown={(e) => e.key === 'Enter' && createStash()}
				/>
				<button 
					class="create-btn"
					onclick={createStash}
					disabled={creating}
				>
					{creating ? '...' : 'Stash'}
				</button>
			</div>
		</div>
	</div>

	<div class="stash-detail">
		{#if selectedStash}
			<div class="detail-header">
				<h2 class="detail-ref">stash@{`{${selectedStash.index}}`}</h2>
				<div class="detail-message">{selectedStash.message}</div>
			</div>

			<div class="detail-info">
				<div class="info-row">
					<span class="info-label">Created</span>
					<span class="info-value">{selectedStash.date}</span>
				</div>
				{#if selectedStash.branch}
					<div class="info-row">
						<span class="info-label">Branch</span>
						<span class="info-value">{selectedStash.branch}</span>
					</div>
				{/if}
			</div>

			<div class="detail-actions">
				<button 
					class="action-btn apply"
					onclick={() => applyStash(selectedStash!)}
					disabled={applying}
				>
					{applying ? 'Applying...' : 'Apply'}
				</button>
				<button 
					class="action-btn pop"
					onclick={() => popStash(selectedStash!)}
					disabled={popping}
				>
					{popping ? 'Popping...' : 'Pop'}
				</button>
				<button 
					class="action-btn drop"
					onclick={() => dropStash(selectedStash!)}
					disabled={dropping}
				>
					{dropping ? 'Dropping...' : 'Drop'}
				</button>
			</div>

			<div class="detail-help">
				<div class="help-item">
					<span class="help-term">Apply</span>
					<span class="help-desc">Apply stash changes but keep the stash</span>
				</div>
				<div class="help-item">
					<span class="help-term">Pop</span>
					<span class="help-desc">Apply stash changes and remove the stash</span>
				</div>
				<div class="help-item">
					<span class="help-term">Drop</span>
					<span class="help-desc">Remove the stash without applying</span>
				</div>
			</div>
		{:else}
			<div class="detail-placeholder">
				<span>Select a stash to view details</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.stashes-tab {
		display: flex;
		height: 100%;
		overflow: hidden;
	}

	.stash-list {
		width: 320px;
		flex-shrink: 0;
		border-right: 1px solid var(--color-border-muted);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.list-header {
		padding: 12px 16px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
	}

	.list-content {
		flex: 1;
		overflow-y: auto;
	}

	.stash-item {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		width: 100%;
		padding: 12px 16px;
		background: none;
		border: none;
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-muted) 25%, transparent);
		cursor: pointer;
		text-align: left;
	}

	.stash-item:hover {
		background: var(--color-border-muted);
	}

	.stash-item.selected {
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
	}

	.stash-indicator {
		color: var(--color-warning);
		font-size: 10px;
		margin-top: 2px;
	}

	.stash-info {
		flex: 1;
		min-width: 0;
	}

	.stash-ref {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 11px;
		color: var(--color-accent);
		margin-bottom: 4px;
	}

	.stash-message {
		font-size: 13px;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-bottom: 4px;
	}

	.stash-meta {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.stash-branch {
		color: var(--color-success);
	}

	.empty-state {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 40px;
		text-align: center;
	}

	.empty-text {
		font-size: 14px;
		color: var(--color-text);
	}

	.empty-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.create-section {
		border-top: 1px solid var(--color-border-muted);
		padding: 12px 16px;
		background: var(--color-surface);
	}

	.section-header {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 8px;
	}

	.create-row {
		display: flex;
		gap: 8px;
	}

	.create-input {
		flex: 1;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		padding: 8px 12px;
		color: var(--color-text);
		font-size: 12px;
		outline: none;
	}

	.create-input:focus {
		border-color: var(--color-accent);
	}

	.create-input::placeholder {
		color: var(--color-text-subtle);
	}

	.create-btn {
		background: var(--color-warning);
		border: none;
		border-radius: 6px;
		color: var(--color-base);
		font-size: 12px;
		font-weight: 600;
		padding: 8px 16px;
		cursor: pointer;
	}

	.create-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.create-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.stash-detail {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.detail-placeholder {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-subtle);
		font-size: 14px;
	}

	.detail-header {
		padding: 24px;
		border-bottom: 1px solid var(--color-border-muted);
		background: var(--color-surface);
	}

	.detail-ref {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 14px;
		font-weight: 600;
		color: var(--color-accent);
		margin: 0 0 8px;
	}

	.detail-message {
		font-size: 16px;
		color: var(--color-text);
	}

	.detail-info {
		padding: 24px;
		border-bottom: 1px solid var(--color-border-muted);
	}

	.info-row {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 6px 0;
	}

	.info-label {
		width: 80px;
		font-size: 12px;
		color: var(--color-text-subtle);
		font-weight: 500;
	}

	.info-value {
		font-size: 13px;
		color: var(--color-text);
	}

	.detail-actions {
		display: flex;
		gap: 8px;
		padding: 24px;
		border-bottom: 1px solid var(--color-border-muted);
	}

	.action-btn {
		flex: 1;
		padding: 10px 16px;
		border: none;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.action-btn.apply {
		background: var(--color-success);
		color: var(--color-base);
	}

	.action-btn.apply:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.action-btn.pop {
		background: var(--color-accent);
		color: var(--color-base);
	}

	.action-btn.pop:hover:not(:disabled) {
		filter: brightness(1.1);
	}

	.action-btn.drop {
		background: color-mix(in srgb, var(--color-error) 12%, transparent);
		color: var(--color-error);
		border: 1px solid color-mix(in srgb, var(--color-error) 25%, transparent);
	}

	.action-btn.drop:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-error) 18%, transparent);
	}

	.detail-help {
		padding: 24px;
	}

	.help-item {
		display: flex;
		align-items: baseline;
		gap: 8px;
		padding: 6px 0;
	}

	.help-term {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
		width: 50px;
	}

	.help-desc {
		font-size: 12px;
		color: var(--color-text-subtle);
	}
</style>
