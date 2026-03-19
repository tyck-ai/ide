<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { projectRoot } from '$lib/stores/editor';
	import { toast } from '$lib/stores/toast';

	type Props = {
		onAdded: () => void;
		onClose: () => void;
	};

	let { onAdded, onClose }: Props = $props();

	let remoteUrl = $state('');
	let adding = $state(false);

	async function addRemote() {
		if (!$projectRoot || adding || !remoteUrl.trim()) return;
		adding = true;
		try {
			await invoke('git_add_remote', { path: $projectRoot, name: 'origin', url: remoteUrl.trim() });
			onAdded();
		} catch (e) {
			toast.error(`Failed to add remote: ${e}`);
		}
		adding = false;
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
	<div class="modal" onclick={(e) => e.stopPropagation()}>
		<div class="title">Add Git Remote</div>
		<p class="desc">No remote is configured for this repository. Add one to push your commits.</p>
		<label class="field">
			<span class="label">Remote URL</span>
			<input
				class="input"
				bind:value={remoteUrl}
				placeholder="https://github.com/user/repo.git"
				onkeydown={(e) => e.key === 'Enter' && addRemote()}
			/>
		</label>
		<p class="hint">Will be added as <code>origin</code>.</p>
		<div class="actions">
			<button class="btn cancel" onclick={onClose}>Cancel</button>
			<button class="btn confirm" onclick={addRemote} disabled={adding || !remoteUrl.trim()}>
				{adding ? 'Adding…' : 'Add & Push'}
			</button>
		</div>
	</div>
</div>

<style>
	.backdrop {
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
		width: 440px;
		max-width: 90vw;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.title { font-size: 16px; font-weight: 600; }
	.desc { font-size: 13px; color: var(--color-text-secondary); line-height: 1.5; margin: 0; }
	.field { display: flex; flex-direction: column; gap: 6px; }
	.label { font-size: 12px; font-weight: 600; color: var(--color-text-secondary); }
	.input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: inherit;
		outline: none;
	}
	.input:focus { border-color: var(--color-accent); }
	.hint { font-size: 12px; color: var(--color-text-subtle); margin: 0; }
	.hint code {
		font-family: 'SF Mono', 'Fira Code', monospace;
		background: var(--color-overlay);
		padding: 1px 5px;
		border-radius: 3px;
	}
	.actions { display: flex; justify-content: flex-end; gap: 8px; }
	.btn { padding: 8px 18px; border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; }
	.btn.cancel { background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text); }
	.btn.confirm { background: var(--color-accent); border: none; color: white; }
	.btn.confirm:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
