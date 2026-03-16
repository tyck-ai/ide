<script lang="ts">
	import { agentActiveFile } from '$lib/stores/agent';
	import { activeSession, pauseAgent, resumeAgent } from '$lib/stores/agentTerminal';
	import { activeFilePath } from '$lib/stores/editor';
	import { isAgentMode } from '$lib/stores/settings';

	const agentIsEditingThisFile = $derived(
		$isAgentMode &&
		$activeSession &&
		$agentActiveFile &&
		$activeFilePath &&
		$agentActiveFile === $activeFilePath
	);

	const agentIsPaused = $derived(
		$isAgentMode &&
		$activeSession?.status === 'paused'
	);

	const showBar = $derived(agentIsEditingThisFile || agentIsPaused);
	let resuming = $state(false);

	async function handleTakeOver() {
		if ($activeSession) {
			await pauseAgent($activeSession.id);
		}
	}

	async function handleResume() {
		if ($activeSession) {
			resuming = true;
			await resumeAgent($activeSession.id);
			resuming = false;
		}
	}
</script>

{#if showBar}
	<div class="agent-edit-bar" class:paused={agentIsPaused} class:active={agentIsEditingThisFile && !agentIsPaused}>
		{#if resuming}
			<span class="bar-text">
				<span class="bar-icon">🤖</span>
				Agent resuming...
			</span>
		{:else if agentIsPaused}
			<span class="bar-text">
				<span class="bar-icon">✏️</span>
				You're editing — agent paused
			</span>
			<button class="bar-btn resume" onclick={handleResume}>
				Resume Agent
			</button>
		{:else if agentIsEditingThisFile}
			<span class="bar-text">
				<span class="bar-icon">🤖</span>
				Agent is editing this file
			</span>
			<button class="bar-btn takeover" onclick={handleTakeOver}>
				Take Over
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
	.agent-edit-bar.active {
		background: color-mix(in srgb, var(--color-accent) 12%, var(--color-surface));
		border-bottom: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
		color: var(--color-accent);
	}
	.agent-edit-bar.paused {
		background: color-mix(in srgb, var(--color-warning, #d29922) 12%, var(--color-surface));
		border-bottom: 1px solid color-mix(in srgb, var(--color-warning, #d29922) 30%, transparent);
		color: var(--color-warning, #d29922);
	}
	.bar-text {
		display: flex;
		align-items: center;
		gap: 6px;
		font-weight: 500;
	}
	.bar-icon {
		font-size: 14px;
	}
	.bar-btn {
		padding: 4px 14px;
		border-radius: 4px;
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		border: none;
	}
	.bar-btn.takeover {
		background: var(--color-accent);
		color: white;
	}
	.bar-btn.takeover:hover {
		filter: brightness(1.1);
	}
	.bar-btn.resume {
		background: var(--color-success, #238636);
		color: white;
	}
	.bar-btn.resume:hover {
		filter: brightness(1.1);
	}
</style>
