<script lang="ts">
	import { agentProviders, activeProviderId } from '$lib/stores/agentProvider';
	import { agentSessions, spawnAgentSession } from '$lib/stores/agentTerminal';
	import { pickSessionName } from '$lib/utils/sessionNames';
	import { invoke } from '@tauri-apps/api/core';
	import { get } from 'svelte/store';

	interface Props {
		onClose: () => void;
	}

	let { onClose }: Props = $props();

	const usedNames = new Set(get(agentSessions).map(s => s.label));
	let sessionName = $state(pickSessionName(usedNames));
	let selectedProvider = $state($activeProviderId ?? $agentProviders[0]?.id ?? '');
	let instructions = $state('');
	let starting = $state(false);
	let error = $state<string | null>(null);

	async function startSession() {
		if (starting || !selectedProvider) return;
		starting = true;
		error = null;
		activeProviderId.set(selectedProvider);

		try {
			const sessionId = await spawnAgentSession(undefined, selectedProvider, undefined, sessionName.trim() || undefined);

			if (instructions.trim()) {
				const msg = instructions.trim() + '\r';
				const sendWithRetry = async () => {
					for (let i = 0; i < 5; i++) {
						await new Promise(r => setTimeout(r, 2000));
						try {
							await invoke('write_terminal', { id: sessionId, data: msg });
							return;
						} catch { /* agent not ready yet */ }
					}
				};
				sendWithRetry().catch(() => console.warn('Failed to send initial instructions to agent'));
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
<div class="backdrop" onclick={onClose}>
	<div class="modal" onclick={(e) => e.stopPropagation()}>
		<div class="modal-header">
			<span class="modal-title">Start an Agent</span>
			<button class="close-btn" onclick={onClose}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
			</button>
		</div>

		{#if $agentProviders.length === 0}
			<div class="no-providers">
				<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
					<circle cx="12" cy="12" r="10"/><path d="M12 8v4m0 4h.01"/>
				</svg>
				<p>No AI provider installed.</p>
				<p class="hint">Install an AI agent to get started.</p>
			</div>
		{:else}
			<label class="field">
			<span class="field-label">Name</span>
			<input
				type="text"
				class="field-input"
				bind:value={sessionName}
				placeholder="Session name"
				maxlength="32"
				disabled={starting}
			/>
		</label>

		<div class="section-label">Choose agent</div>
			<div class="provider-list">
				{#each $agentProviders as provider (provider.id)}
					<button
						class="provider-card"
						class:selected={selectedProvider === provider.id}
						onclick={() => selectedProvider = provider.id}
					>
						<div class="provider-radio">
							{#if selectedProvider === provider.id}
								<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="10" opacity="0.15"/><circle cx="12" cy="12" r="5"/></svg>
							{:else}
								<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/></svg>
							{/if}
						</div>
						<div class="provider-info">
							<span class="provider-name">{provider.displayName}</span>
							<span class="provider-binary">{provider.binary}</span>
						</div>
					</button>
				{/each}
			</div>

			<label class="field">
				<span class="field-label">First message <span class="optional">(optional)</span></span>
				<textarea
					class="field-textarea"
					bind:value={instructions}
					placeholder="Describe what you want the agent to do..."
					rows="3"
					disabled={starting}
				></textarea>
			</label>

			{#if error}
				<div class="error">{error}</div>
			{/if}

			<div class="modal-actions">
				<button class="btn cancel" onclick={onClose} disabled={starting}>Cancel</button>
				<button
					class="btn primary"
					onclick={startSession}
					disabled={starting || !selectedProvider}
				>
					{starting ? 'Starting...' : 'Start Agent'}
				</button>
			</div>
		{/if}
	</div>
</div>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 300;
	}
	.modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		padding: 20px;
		width: 380px;
		max-width: 90vw;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
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
		cursor: pointer;
		padding: 2px;
		display: flex;
		border-radius: 4px;
	}
	.close-btn:hover { background: var(--color-overlay); color: var(--color-text); }

	.section-label {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-subtle);
	}
	.provider-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.provider-card {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		cursor: pointer;
		text-align: left;
		transition: border-color 0.1s, background 0.1s;
	}
	.provider-card:hover {
		border-color: var(--color-border);
		background: var(--color-overlay);
	}
	.provider-card.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}
	.provider-radio {
		color: var(--color-accent);
		flex-shrink: 0;
		display: flex;
	}
	.provider-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.provider-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
	}
	.provider-binary {
		font-size: 10px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		color: var(--color-text-subtle);
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.field-label {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-subtle);
	}
	.optional {
		font-weight: 400;
		text-transform: none;
		letter-spacing: 0;
	}
	.field-input {
		padding: 8px 10px;
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
	.field-input:disabled { opacity: 0.5; }
	.field-textarea {
		padding: 8px 10px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 12px;
		font-family: inherit;
		resize: vertical;
		min-height: 60px;
	}
	.field-textarea:focus {
		outline: none;
		border-color: var(--color-accent);
	}
	.field-textarea:disabled { opacity: 0.5; }

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
	}
	.btn {
		padding: 7px 16px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
	}
	.btn.cancel {
		background: none;
		border: 1px solid var(--color-border);
		color: var(--color-text);
	}
	.btn.cancel:hover { background: var(--color-overlay); }
	.btn.primary {
		background: var(--color-accent);
		border: none;
		color: white;
	}
	.btn.primary:hover:not(:disabled) { filter: brightness(1.1); }
	.btn:disabled { opacity: 0.5; cursor: not-allowed; }

	.no-providers {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 24px 0;
		color: var(--color-text-subtle);
		text-align: center;
	}
	.no-providers p { font-size: 13px; margin: 0; }
	.no-providers .hint { font-size: 11px; }
</style>
