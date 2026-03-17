<script lang="ts">
	import { agentProviders, activeProviderId } from '$lib/stores/agentProvider';
	import { spawnAgentSession } from '$lib/stores/agentTerminal';
	import { invoke } from '@tauri-apps/api/core';

	interface Props {
		onClose: () => void;
	}

	let { onClose }: Props = $props();

	let branchName = $state(`feat/${crypto.randomUUID().slice(0, 8)}`);
	let selectedProvider = $state($activeProviderId ?? 'claude-code');
	let instructions = $state('');
	let starting = $state(false);
	let error = $state<string | null>(null);

	async function startSession() {
		if (starting) return;
		starting = true;
		error = null;

		try {
			const sessionId = await spawnAgentSession(undefined, selectedProvider);

			// If instructions provided, send them as the first message
			// Retry a few times since the agent takes a moment to initialize
			if (instructions.trim()) {
				const msg = instructions.trim() + '\n';
				const sendWithRetry = async (attempts: number, delay: number) => {
					for (let i = 0; i < attempts; i++) {
						await new Promise(r => setTimeout(r, delay));
						try {
							await invoke('write_terminal', { id: sessionId, data: msg });
							return;
						} catch {
							// Agent not ready yet, retry
						}
					}
				};
				sendWithRetry(5, 2000);
			}

			onClose();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			starting = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
		if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) startSession();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={onClose}>
	<div class="modal" onclick={(e) => e.stopPropagation()}>
		<div class="modal-title">New Agent Session</div>

		<label class="field">
			<span class="field-label">Branch name</span>
			<input
				type="text"
				class="field-input"
				bind:value={branchName}
				placeholder="feat/my-feature"
			/>
		</label>

		<label class="field">
			<span class="field-label">Agent</span>
			<select class="field-input" bind:value={selectedProvider}>
				{#each $agentProviders as p (p.id)}
					<option value={p.id}>{p.displayName}</option>
				{/each}
				{#if $agentProviders.length === 0}
					<option value="" disabled>No agents installed</option>
				{/if}
			</select>
		</label>

		<label class="field">
			<span class="field-label">Instructions <span class="optional">(optional)</span></span>
			<textarea
				class="field-input field-textarea"
				bind:value={instructions}
				placeholder="Describe what the agent should work on..."
				rows="3"
			></textarea>
		</label>

		{#if error}
			<div class="error">{error}</div>
		{/if}

		<div class="modal-actions">
			<button class="btn cancel" onclick={onClose}>Cancel</button>
			<button
				class="btn primary"
				onclick={startSession}
				disabled={starting || $agentProviders.length === 0}
			>
				{starting ? 'Starting...' : 'Start Session'}
			</button>
		</div>
	</div>
</div>

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}
	.modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		padding: 24px;
		width: 420px;
		max-width: 90vw;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.modal-title {
		font-size: 16px;
		font-weight: 600;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.field-label {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-secondary);
	}
	.optional {
		font-weight: 400;
		color: var(--color-text-subtle);
	}
	.field-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: inherit;
	}
	.field-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}
	.field-textarea {
		resize: vertical;
		min-height: 60px;
	}
	select.field-input {
		cursor: pointer;
	}
	.error {
		padding: 8px 12px;
		background: rgba(248, 81, 73, 0.1);
		border: 1px solid rgba(248, 81, 73, 0.3);
		border-radius: 6px;
		color: #f85149;
		font-size: 12px;
	}
	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 4px;
	}
	.btn {
		padding: 8px 18px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
	}
	.btn.cancel {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		color: var(--color-text);
	}
	.btn.primary {
		background: var(--color-accent);
		border: none;
		color: white;
	}
	.btn.primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
