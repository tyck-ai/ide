<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { openFileInEditor, projectRoot } from '$lib/stores/editor';
	import { activeReview } from '$lib/stores/sessionReview';
	import { git } from '$lib/stores/git';
	import ReviewPanel from './ReviewPanel.svelte';

	interface DirEntry {
		name: string;
		path: string;
		is_dir: boolean;
		children: DirEntry[] | null;
	}

	interface FsChangeEvent {
		path: string;
		parent: string;
	}

	let tree: DirEntry[] = $state([]);
	let expandedDirs = $state(new Set<string>());
	let rootPath = $state('');
	let unlisten: UnlistenFn | null = null;

	// Create a map of file paths to their git status
	const gitStatusMap = $derived.by(() => {
		const map = new Map<string, { status: string; type: 'staged' | 'unstaged' | 'untracked' }>();
		
		for (const file of $git.staged) {
			map.set(file.path, { status: file.status, type: 'staged' });
		}
		for (const file of $git.unstaged) {
			if (!map.has(file.path)) {
				map.set(file.path, { status: file.status, type: 'unstaged' });
			}
		}
		for (const path of $git.untracked) {
			if (!map.has(path)) {
				map.set(path, { status: '?', type: 'untracked' });
			}
		}
		
		return map;
	});

	projectRoot.subscribe(val => {
		if (val) {
			rootPath = val;
			loadTree(val);
			startWatching(val);
		} else {
			stopWatching();
		}
	});

	async function startWatching(path: string) {
		await stopWatching();

		try {
			await invoke('watch_directory', { path });
		} catch (e) {
			console.warn('watch_directory failed:', e);
		}

		unlisten = await listen<FsChangeEvent>('fs-change', () => {
			// Reload the full tree on any filesystem change
			if (rootPath) loadTree(rootPath);
		});
	}

	async function stopWatching() {
		if (unlisten) {
			unlisten();
			unlisten = null;
		}
		try {
			await invoke('stop_watching');
		} catch { /* ignore */ }
	}

	onDestroy(() => {
		stopWatching();
	});

	async function loadTree(path: string) {
		try {
			tree = await invoke('read_directory', { path });
		} catch (e) {
			console.error('Failed to read directory:', e);
		}
	}

	function toggleDir(path: string) {
		expandedDirs = new Set(expandedDirs);
		if (expandedDirs.has(path)) {
			expandedDirs.delete(path);
		} else {
			expandedDirs.add(path);
		}
	}

	async function openFile(entry: DirEntry) {
		if (entry.is_dir) {
			toggleDir(entry.path);
			return;
		}
		try {
			const content: string = await invoke('read_file', { path: entry.path });
			openFileInEditor(entry.path, entry.name, content);
		} catch (e) {
			console.error('Failed to open file:', e);
		}
	}

	function getIcon(entry: DirEntry): string {
		if (entry.is_dir) {
			return expandedDirs.has(entry.path) ? '▾' : '▸';
		}
		const ext = entry.name.split('.').pop() ?? '';
		const icons: Record<string, string> = {
			ts: '◇', js: '◇', svelte: '◆', rs: '⚙', json: '{}', css: '◈',
			html: '◁', md: '¶', toml: '≡', lock: '🔒',
		};
		return icons[ext] ?? '○';
	}

	function getRelativePath(fullPath: string): string {
		if (!rootPath) return fullPath;
		return fullPath.startsWith(rootPath) ? fullPath.slice(rootPath.length + 1) : fullPath;
	}

	function getGitStatus(entry: DirEntry): { status: string; type: string } | null {
		const relativePath = getRelativePath(entry.path);
		return gitStatusMap.get(relativePath) ?? null;
	}

	function getGitStatusClass(gitStatus: { status: string; type: string } | null): string {
		if (!gitStatus) return '';
		switch (gitStatus.status) {
			case 'M': return 'git-modified';
			case 'A': return 'git-added';
			case 'D': return 'git-deleted';
			case '?': return 'git-untracked';
			default: return 'git-modified';
		}
	}
</script>

<div class="context-zone">
	{#if $activeReview?.reviewMode}
		<ReviewPanel />
	{:else}
		<div class="header">
			<span class="title">Explorer</span>
		</div>
		<div class="tree">
			{#snippet renderTree(entries: DirEntry[], depth: number)}
				{#each entries as entry (entry.path)}
					{@const gitStatus = getGitStatus(entry)}
					<button
						class="tree-item {getGitStatusClass(gitStatus)}"
						class:is-dir={entry.is_dir}
						style="padding-left: {12 + depth * 16}px"
						onclick={() => openFile(entry)}
					>
						<span class="icon">{getIcon(entry)}</span>
						<span class="name">{entry.name}</span>
						{#if gitStatus}
							<span class="git-badge">{gitStatus.status}</span>
						{/if}
					</button>
					{#if entry.is_dir && entry.children && expandedDirs.has(entry.path)}
						{@render renderTree(entry.children, depth + 1)}
					{/if}
				{/each}
			{/snippet}
			{@render renderTree(tree, 0)}
		</div>
	{/if}
</div>

<style>
	.context-zone {
		display: flex;
		flex-direction: column;
		background: var(--color-base);
		height: 100%;
		overflow: hidden;
	}
	.header {
		padding: 10px 12px;
		border-bottom: 1px solid var(--color-border-muted);
	}
	.title {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-subtle);
	}
	.tree {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
	}
	.tree-item {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 3px 12px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 13px;
		cursor: pointer;
		text-align: left;
		white-space: nowrap;
	}
	.tree-item:hover {
		background: var(--color-overlay);
	}
	.icon {
		font-size: 11px;
		width: 14px;
		text-align: center;
		flex-shrink: 0;
		color: var(--color-text-subtle);
	}
	.is-dir .icon {
		color: var(--color-accent);
	}
	.name {
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.git-badge {
		margin-left: auto;
		font-size: 10px;
		font-weight: 700;
		font-family: 'SF Mono', 'Fira Code', monospace;
		padding: 0 4px;
		border-radius: 3px;
		flex-shrink: 0;
	}
	.git-modified .name {
		color: var(--color-warning);
	}
	.git-modified .git-badge {
		color: var(--color-warning);
	}
	.git-added .name {
		color: var(--color-success);
	}
	.git-added .git-badge {
		color: var(--color-success);
	}
	.git-deleted .name {
		color: var(--color-error);
		text-decoration: line-through;
	}
	.git-deleted .git-badge {
		color: var(--color-error);
	}
	.git-untracked .name {
		color: var(--color-text-subtle);
	}
	.git-untracked .git-badge {
		color: var(--color-text-subtle);
	}
</style>
