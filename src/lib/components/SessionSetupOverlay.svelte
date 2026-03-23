<script lang="ts">
	import { fade } from 'svelte/transition';
	import { sessionSetups, type SetupStep } from '$lib/stores/sessionSetup';

	let { sessionId }: { sessionId: string } = $props();

	const setup = $derived($sessionSetups.get(sessionId));
	const step = $derived(setup?.step ?? 'workspace');
	const progressText = $derived(setup?.progressText ?? '');

	const STEPS: { id: SetupStep; label: string }[] = [
		{ id: 'workspace', label: 'Creating workspace' },
		{ id: 'files', label: 'Copying project files' },
		{ id: 'agent', label: 'Starting agent' }
	];

	const STEP_ORDER: SetupStep[] = ['workspace', 'files', 'agent', 'started'];

	function stepState(id: SetupStep): 'pending' | 'active' | 'done' {
		const current = STEP_ORDER.indexOf(step);
		const target = STEP_ORDER.indexOf(id);
		if (target < current) return 'done';
		if (target === current && step !== 'started') return 'active';
		return 'pending';
	}

	// Elapsed time hint — show after 4s if still on 'workspace' or 'files' step
	let elapsedSeconds = $state(0);
	let startTime = $state(Date.now());

	$effect(() => {
		// Reset timer when step changes
		// eslint-disable-next-line @typescript-eslint/no-unused-expressions
		step;
		startTime = Date.now();
		elapsedSeconds = 0;
	});

	$effect(() => {
		const interval = setInterval(() => {
			elapsedSeconds = Math.floor((Date.now() - startTime) / 1000);
		}, 500);
		return () => clearInterval(interval);
	});

	const showHint = $derived(
		(step === 'workspace' || step === 'files') && elapsedSeconds >= 4 && !progressText
	);
</script>

<div class="overlay" transition:fade={{ duration: 250 }}>
	<div class="card">
		<div class="logo">tyck</div>
		<div class="steps">
			{#each STEPS as s}
				{@const state = stepState(s.id)}
				<div class="step" class:active={state === 'active'} class:done={state === 'done'}>
					<div class="step-icon">
						{#if state === 'done'}
							<svg width="14" height="14" viewBox="0 0 14 14" fill="none">
								<path
									d="M2.5 7L5.5 10L11.5 4"
									stroke="currentColor"
									stroke-width="1.75"
									stroke-linecap="round"
									stroke-linejoin="round"
								/>
							</svg>
						{:else if state === 'active'}
							<div class="spinner"></div>
						{:else}
							<div class="dot"></div>
						{/if}
					</div>
					<span class="step-label">{s.label}</span>
				</div>
			{/each}
		</div>
		{#if progressText}
			<p class="progress-text" transition:fade={{ duration: 150 }}>{progressText}</p>
		{:else if showHint}
			<p class="hint-text" transition:fade={{ duration: 300 }}>Large repositories may take 20–30 seconds</p>
		{/if}
	</div>
</div>

<style>
	.overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--color-base);
		z-index: 50;
	}

	.card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 32px;
	}

	.logo {
		font-size: 48px;
		font-weight: 800;
		color: var(--color-accent);
		letter-spacing: 2px;
		text-shadow: 0 0 40px color-mix(in srgb, var(--color-accent) 30%, transparent);
		line-height: 1;
	}

	.steps {
		display: flex;
		flex-direction: column;
		gap: 16px;
		min-width: 190px;
	}

	.step {
		display: flex;
		align-items: center;
		gap: 10px;
		opacity: 0.2;
		transition:
			opacity 0.35s ease,
			color 0.35s ease;
	}

	.step.active {
		opacity: 1;
	}

	.step.done {
		opacity: 0.45;
	}

	.step-icon {
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		color: var(--color-accent);
	}

	.step-label {
		font-size: 13px;
		color: var(--color-text);
		font-weight: 400;
		letter-spacing: 0.01em;
	}

	.step.active .step-label {
		font-weight: 500;
	}

	.dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: currentColor;
		color: var(--color-text-subtle);
		opacity: 0.5;
	}

	.spinner {
		width: 13px;
		height: 13px;
		border: 1.5px solid var(--color-accent);
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.65s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.progress-text {
		font-size: 11px;
		color: var(--color-text-subtle);
		opacity: 0.6;
		font-variant-numeric: tabular-nums;
		letter-spacing: 0.02em;
		max-width: 260px;
		text-align: center;
	}

	.hint-text {
		font-size: 11px;
		color: var(--color-text-subtle);
		opacity: 0.45;
		letter-spacing: 0.02em;
		max-width: 260px;
		text-align: center;
	}
</style>
