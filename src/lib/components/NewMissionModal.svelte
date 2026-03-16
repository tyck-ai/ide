<script lang="ts">
	import { agentProviders, activeProviderId } from '$lib/stores/agentProvider';
	import { createMission, startMission, type MissionTask } from '$lib/stores/missions';

	interface Props {
		onClose: () => void;
	}

	let { onClose }: Props = $props();

	let description = $state('');
	let selectedAgent = $state($activeProviderId ?? 'claude-code');
	let maxParallel = $state(3);
	let phase = $state<'describe' | 'plan'>('describe');
	let tasks = $state<{ description: string; branchName: string; enabled: boolean }[]>([]);
	let starting = $state(false);

	function addTask() {
		const idx = tasks.length + 1;
		tasks = [...tasks, {
			description: '',
			branchName: `feat/task-${idx}`,
			enabled: true,
		}];
	}

	function removeTask(index: number) {
		tasks = tasks.filter((_, i) => i !== index);
	}

	function goToPlan() {
		if (!description.trim()) return;
		// Start with one empty task for the user to fill in
		if (tasks.length === 0) {
			addTask();
		}
		phase = 'plan';
	}

	async function start() {
		if (starting) return;
		starting = true;

		const enabledTasks = tasks.filter(t => t.enabled && t.description.trim());
		const mission = createMission(
			description.trim(),
			enabledTasks.map(t => ({
				description: t.description.trim(),
				branchName: t.branchName.trim(),
				agent: selectedAgent,
			})),
			selectedAgent,
			maxParallel,
		);

		await startMission(mission.id);
		onClose();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={onClose}>
	<div class="modal" onclick={(e) => e.stopPropagation()}>
		{#if phase === 'describe'}
			<div class="modal-title">New Mission</div>
			<p class="modal-subtitle">Describe what you want to accomplish. You'll define tasks next.</p>

			<label class="field">
				<span class="field-label">What do you want to accomplish?</span>
				<textarea
					class="field-input field-textarea"
					bind:value={description}
					placeholder="Add authentication to the app. Need JWT middleware, login/signup pages, and integration tests."
					rows="4"
				></textarea>
			</label>

			<div class="row">
				<label class="field flex-1">
					<span class="field-label">Agent</span>
					<select class="field-input" bind:value={selectedAgent}>
						{#each $agentProviders as p (p.id)}
							<option value={p.id}>{p.displayName}</option>
						{/each}
					</select>
				</label>
				<label class="field">
					<span class="field-label">Max parallel</span>
					<select class="field-input" bind:value={maxParallel}>
						<option value={1}>1</option>
						<option value={2}>2</option>
						<option value={3}>3</option>
						<option value={4}>4</option>
						<option value={6}>6</option>
					</select>
				</label>
			</div>

			<div class="modal-actions">
				<button class="btn cancel" onclick={onClose}>Cancel</button>
				<button class="btn primary" onclick={goToPlan} disabled={!description.trim()}>
					Define Tasks →
				</button>
			</div>
		{:else}
			<div class="modal-title">Mission Tasks</div>
			<p class="modal-subtitle">{description}</p>

			<div class="tasks-list">
				{#each tasks as task, i}
					<div class="task-row">
						<label class="task-check">
							<input type="checkbox" bind:checked={task.enabled} />
						</label>
						<div class="task-fields">
							<input
								class="task-input"
								bind:value={task.description}
								placeholder="Describe this task..."
							/>
							<input
								class="task-branch"
								bind:value={task.branchName}
								placeholder="feat/branch-name"
							/>
						</div>
						<button class="task-remove" onclick={() => removeTask(i)}>×</button>
					</div>
				{/each}
			</div>

			<button class="add-task-btn" onclick={addTask}>+ Add Task</button>

			<div class="modal-actions">
				<button class="btn cancel" onclick={() => phase = 'describe'}>← Back</button>
				<button class="btn cancel" onclick={onClose}>Cancel</button>
				<button
					class="btn primary"
					onclick={start}
					disabled={starting || tasks.filter(t => t.enabled && t.description.trim()).length === 0}
				>
					{starting ? 'Starting...' : `Start ${tasks.filter(t => t.enabled && t.description.trim()).length} Tasks`}
				</button>
			</div>
		{/if}
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
		width: 520px;
		max-width: 90vw;
		max-height: 80vh;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.modal-title {
		font-size: 16px;
		font-weight: 600;
	}
	.modal-subtitle {
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.4;
		margin-top: -8px;
	}
	.row {
		display: flex;
		gap: 12px;
	}
	.flex-1 { flex: 1; }
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
	.field-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: inherit;
	}
	.field-input:focus { outline: none; border-color: var(--color-accent); }
	.field-textarea { resize: vertical; min-height: 80px; }
	select.field-input { cursor: pointer; }

	.tasks-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.task-row {
		display: flex;
		align-items: flex-start;
		gap: 8px;
	}
	.task-check {
		padding-top: 8px;
	}
	.task-fields {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.task-input, .task-branch {
		padding: 6px 10px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		color: var(--color-text);
		font-size: 12px;
	}
	.task-input:focus, .task-branch:focus {
		outline: none;
		border-color: var(--color-accent);
	}
	.task-branch {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 11px;
		color: var(--color-text-subtle);
	}
	.task-remove {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 16px;
		cursor: pointer;
		padding: 4px;
	}
	.task-remove:hover { color: var(--color-error); }

	.add-task-btn {
		background: none;
		border: 1px dashed var(--color-border-muted);
		border-radius: 6px;
		padding: 8px;
		color: var(--color-text-subtle);
		font-size: 12px;
		cursor: pointer;
	}
	.add-task-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
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
	.btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
