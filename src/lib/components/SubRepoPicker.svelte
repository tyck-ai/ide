<script lang="ts">
	export interface SubRepo { name: string; path: string; }

	interface Props {
		repos: SubRepo[];
		selected: string | null;
		onSelect: (path: string) => void;
	}

	let { repos, selected, onSelect }: Props = $props();
</script>

<div class="repo-list">
	{#each repos as repo (repo.path)}
		<button
			class="repo-item"
			class:selected={selected === repo.path}
			onclick={() => onSelect(repo.path)}
		>
			<svg class="repo-icon" width="13" height="13" viewBox="0 0 16 16" fill="none">
				<path d="M2 2.5A2.5 2.5 0 014.5 0h8.75a.75.75 0 01.75.75v12.5a.75.75 0 01-.75.75h-2.5a.75.75 0 010-1.5h1.75v-2h-8a1 1 0 00-.714 1.7.75.75 0 01-1.072 1.05A2.495 2.495 0 012 11.5v-9zm10.5-1V9h-8c-.356 0-.694.074-1 .208V2.5a1 1 0 011-1h8z" fill="currentColor"/>
			</svg>
			<span class="repo-name">{repo.name}</span>
		</button>
	{/each}
</div>

<style>
	.repo-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.repo-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 7px 10px;
		border-radius: 6px;
		border: 1px solid transparent;
		background: none;
		cursor: pointer;
		text-align: left;
		width: 100%;
		transition: background 0.12s, border-color 0.12s;
	}

	.repo-item:hover {
		background: var(--color-overlay);
	}

	.repo-item.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
	}

	.repo-icon {
		color: var(--color-text-subtle);
		flex-shrink: 0;
	}

	.repo-item.selected .repo-icon {
		color: var(--color-accent);
	}

	.repo-name {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
