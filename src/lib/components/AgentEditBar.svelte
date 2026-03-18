<script lang="ts">
	import { activeSession, pauseAgent, resumeAgent } from '$lib/stores/agentTerminal';
	import { isAgentMode } from '$lib/stores/settings';

	const agentIsPaused = $derived(
		$isAgentMode &&
		$activeSession?.status === 'paused'
	);

	let resuming = $state(false);

	async function handleResume() {
		if ($activeSession) {
			resuming = true;
			await resumeAgent($activeSession.id);
			resuming = false;
		}
	}
</script>

{#if agentIsPaused}
	<div class="agent-edit-bar paused">
		{#if resuming}
			<span class="bar-text">Agent resuming...</span>
		{:else}
			<span class="bar-text">
				You're editing — agent paused
			</span>
			<button class="bar-btn resume" onclick={handleResume}>
				Resume Agent
			</button>
		{/if}
	</div>
{/if}

<style>
	.agent-edit-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 14px;
		font-size: 12px;
		z-index: 10;
		flex-shrink: 0;
	}
	.agent-edit-bar.paused {
		background: color-mix(in srgb, var(--color-warning, #d29922) 12%, var(--color-surface));
		border-bottom: 1px solid color-mix(in srgb, var(--color-warning, #d29922) 30%, transparent);
		color: var(--color-warning, #d29922);
	}
	.bar-text {
		font-weight: 500;
	}
	.bar-btn {
		padding: 4px 14px;
		border-radius: 4px;
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		border: none;
	}
	.bar-btn.resume {
		background: var(--color-success, #238636);
		color: white;
	}
	.bar-btn.resume:hover {
		filter: brightness(1.1);
	}
</style>
