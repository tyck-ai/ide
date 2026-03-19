<script lang="ts">
	import { onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { openFileInEditor, activeWorkingDirectory } from '$lib/stores/editor';
	import { git } from '$lib/stores/git';
	import { contextZoneTab } from '$lib/stores/layout';
	import { fileIndex, buildFileIndex } from '$lib/stores/fileIndex';
	import { toast } from '$lib/stores/toast';
	import SearchPanel from './SearchPanel.svelte';
	import FileTreeContextMenu from './FileTreeContextMenu.svelte';

	interface DirEntry {
		name: string;
		path: string;
		is_dir: boolean;
		children: DirEntry[] | null;
	}

	interface FsChangeEvent {
		path: string;
		parent: string;
		windowLabel: string;
	}

	let tree: DirEntry[] = $state([]);
	let expandedDirs = $state(new Set<string>());
	let rootPath = $state('');
	let unlisten: UnlistenFn | null = null;

	// Context menu state
	let ctxVisible = $state(false);
	let ctxX = $state(0);
	let ctxY = $state(0);
	let ctxEntry = $state<DirEntry | null>(null);

	// Inline create state
	let inlineCreateParent = $state<string | null>(null);
	let inlineCreateIsDir = $state(false);
	let inlineCreateName = $state('');
	let inlineCreateInput: HTMLInputElement | undefined;

	// Inline rename state
	let renamingEntry = $state<DirEntry | null>(null);
	let renameValue = $state('');
	let renameInput: HTMLInputElement | undefined;

	// Search overlay
	let searchPanelRef: SearchPanel;
	const showSearch = $derived($contextZoneTab === 'search');

	$effect(() => {
		if (showSearch) {
			setTimeout(() => searchPanelRef?.focusInput(), 50);
		}
	});

	// Git status map
	const gitStatusMap = $derived.by(() => {
		const map = new Map<string, { status: string; type: 'staged' | 'unstaged' | 'untracked' }>();
		for (const file of $git.staged) map.set(file.path, { status: file.status, type: 'staged' });
		for (const file of $git.unstaged) {
			if (!map.has(file.path)) map.set(file.path, { status: file.status, type: 'unstaged' });
		}
		for (const path of $git.untracked) {
			if (!map.has(path)) map.set(path, { status: '?', type: 'untracked' });
		}
		return map;
	});

	const unsubWorkDir = activeWorkingDirectory.subscribe(val => {
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
			await invoke('watch_directory', { path, windowLabel: getCurrentWindow().label });
		} catch (e) {
			console.warn('watch_directory failed:', e);
		}
		unlisten = await listen<FsChangeEvent>('fs-change', (event) => {
			if (event.payload.windowLabel !== getCurrentWindow().label) return;
			if (rootPath) loadTree(rootPath);
		});
	}

	async function stopWatching() {
		if (unlisten) { unlisten(); unlisten = null; }
		try { await invoke('stop_watching', { windowLabel: getCurrentWindow().label }); } catch { /* ignore */ }
	}

	onDestroy(() => { unsubWorkDir(); stopWatching(); });

	async function loadTree(path: string) {
		try {
			tree = await invoke('read_directory', { path });
			fileIndex.set(buildFileIndex(tree, path));
		} catch (e) {
			console.error('Failed to read directory:', e);
		}
	}

	function toggleDir(path: string) {
		expandedDirs = new Set(expandedDirs);
		if (expandedDirs.has(path)) expandedDirs.delete(path);
		else expandedDirs.add(path);
	}

	function collapseAll() { expandedDirs = new Set(); }

	async function openFile(entry: DirEntry) {
		if (entry.is_dir) { toggleDir(entry.path); return; }
		try {
			const content: string = await invoke('read_file', { path: entry.path });
			openFileInEditor(entry.path, entry.name, content);
		} catch (e) {
			console.error('Failed to open file:', e);
		}
	}

	function getIcon(entry: DirEntry): string {
		if (entry.is_dir) return expandedDirs.has(entry.path) ? '▾' : '▸';
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

	function getGitStatus(entry: DirEntry) {
		return gitStatusMap.get(getRelativePath(entry.path)) ?? null;
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

	// ── Context menu ──

	function onTreeContextMenu(e: MouseEvent, entry: DirEntry | null) {
		e.preventDefault();
		ctxEntry = entry; ctxX = e.clientX; ctxY = e.clientY; ctxVisible = true;
	}

	function onContainerContextMenu(e: MouseEvent) {
		if ((e.target as HTMLElement).closest('.tree-item')) return;
		e.preventDefault();
		ctxEntry = null; ctxX = e.clientX; ctxY = e.clientY; ctxVisible = true;
	}

	// ── Inline create ──

	function startInlineCreate(parentPath: string, isDir: boolean) {
		inlineCreateParent = parentPath;
		inlineCreateIsDir = isDir;
		inlineCreateName = '';
		if (parentPath !== rootPath) expandedDirs = new Set([...expandedDirs, parentPath]);
		setTimeout(() => inlineCreateInput?.focus(), 0);
	}

	async function confirmCreate() {
		if (!inlineCreateParent || !inlineCreateName.trim()) { cancelCreate(); return; }
		const newPath = `${inlineCreateParent}/${inlineCreateName.trim()}`;
		try {
			if (inlineCreateIsDir) await invoke('create_directory', { path: newPath });
			else await invoke('create_file', { path: newPath });
			if (rootPath) await loadTree(rootPath);
		} catch (e) {
			toast.error(`Failed to create: ${e}`);
		}
		cancelCreate();
	}

	function cancelCreate() { inlineCreateParent = null; inlineCreateName = ''; }

	// ── Inline rename ──

	function startRename(entry: DirEntry) {
		renamingEntry = entry;
		renameValue = entry.name;
		setTimeout(() => {
			if (renameInput) {
				renameInput.focus();
				const dotIdx = entry.name.lastIndexOf('.');
				if (!entry.is_dir && dotIdx > 0) renameInput.setSelectionRange(0, dotIdx);
				else renameInput.select();
			}
		}, 0);
	}

	async function confirmRename() {
		if (!renamingEntry || !renameValue.trim()) { cancelRename(); return; }
		const oldPath = renamingEntry.path;
		const parent = oldPath.split('/').slice(0, -1).join('/');
		const newPath = `${parent}/${renameValue.trim()}`;
		if (newPath === oldPath) { cancelRename(); return; }
		try {
			await invoke('rename_path', { from: oldPath, to: newPath });
			if (rootPath) await loadTree(rootPath);
		} catch (e) {
			toast.error(`Failed to rename: ${e}`);
		}
		cancelRename();
	}

	function cancelRename() { renamingEntry = null; renameValue = ''; }
</script>

<div class="explorer-panel">
	<!-- Header -->
	<div class="explorer-header">
		<span class="title">Explorer</span>
		<div class="header-actions">
			<button class="header-btn" onclick={() => startInlineCreate(rootPath, false)} title="New File">+f</button>
			<button class="header-btn" onclick={() => startInlineCreate(rootPath, true)} title="New Folder">+d</button>
			<button class="header-btn" onclick={collapseAll} title="Collapse All">⊟</button>
			<button
				class="header-btn"
				class:active={showSearch}
				onclick={() => contextZoneTab.set('search')}
				title="Search (Cmd+Shift+F)"
			>
				<svg width="11" height="11" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<circle cx="6.5" cy="6.5" r="4.5"/>
					<line x1="10.5" y1="10.5" x2="14" y2="14"/>
				</svg>
			</button>
		</div>
	</div>

	<!-- File tree (always rendered) -->
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="tree" oncontextmenu={onContainerContextMenu} role="tree">
		{#snippet renderTree(entries: DirEntry[], depth: number)}
			{#each entries as entry (entry.path)}
				{@const gitStatus = getGitStatus(entry)}
				{#if renamingEntry?.path === entry.path}
					<div class="tree-item rename-item" style="padding-left: {12 + depth * 16}px">
						<span class="icon">{getIcon(entry)}</span>
						<input
							bind:this={renameInput}
							bind:value={renameValue}
							class="rename-input"
							onkeydown={(e) => { if (e.key === 'Enter') confirmRename(); if (e.key === 'Escape') cancelRename(); }}
							onblur={confirmRename}
						/>
					</div>
				{:else}
					<button
						class="tree-item {getGitStatusClass(gitStatus)}"
						class:is-dir={entry.is_dir}
						style="padding-left: {12 + depth * 16}px"
						onclick={() => openFile(entry)}
						oncontextmenu={(e) => onTreeContextMenu(e, entry)}
					>
						<span class="icon">{getIcon(entry)}</span>
						<span class="name">{entry.name}</span>
						{#if gitStatus}
							<span class="git-badge">{gitStatus.status}</span>
						{/if}
					</button>
				{/if}
				{#if entry.is_dir && entry.children && expandedDirs.has(entry.path)}
					{@render renderTree(entry.children, depth + 1)}
					{#if inlineCreateParent === entry.path}
						<div class="tree-item inline-create" style="padding-left: {12 + (depth + 1) * 16}px">
							<span class="icon">{inlineCreateIsDir ? '▸' : '○'}</span>
							<input
								bind:this={inlineCreateInput}
								bind:value={inlineCreateName}
								class="create-input"
								placeholder={inlineCreateIsDir ? 'folder name' : 'filename.ext'}
								onkeydown={(e) => { if (e.key === 'Enter') confirmCreate(); if (e.key === 'Escape') cancelCreate(); }}
								onblur={confirmCreate}
							/>
						</div>
					{/if}
				{/if}
			{/each}
		{/snippet}
		{@render renderTree(tree, 0)}

		{#if inlineCreateParent === rootPath}
			<div class="tree-item inline-create" style="padding-left: 12px">
				<span class="icon">{inlineCreateIsDir ? '▸' : '○'}</span>
				<input
					bind:this={inlineCreateInput}
					bind:value={inlineCreateName}
					class="create-input"
					placeholder={inlineCreateIsDir ? 'folder name' : 'filename.ext'}
					onkeydown={(e) => { if (e.key === 'Enter') confirmCreate(); if (e.key === 'Escape') cancelCreate(); }}
					onblur={confirmCreate}
				/>
			</div>
		{/if}
	</div>

	<!-- Search overlay -->
	{#if showSearch}
		<div class="search-overlay">
			<div class="search-overlay-header">
				<span class="title">Search</span>
				<button class="close-btn" onclick={() => contextZoneTab.set('explorer')} title="Close search">✕</button>
			</div>
			<SearchPanel bind:this={searchPanelRef} />
		</div>
	{/if}
</div>

<FileTreeContextMenu
	visible={ctxVisible}
	x={ctxX}
	y={ctxY}
	entry={ctxEntry}
	parentPath={rootPath}
	onclose={() => ctxVisible = false}
	onrefresh={() => { if (rootPath) loadTree(rootPath); }}
	onrename={(entry) => startRename(entry)}
	oncreate={(parent, isDir) => startInlineCreate(parent, isDir)}
/>

<style>
	.explorer-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
		position: relative;
	}
	.explorer-header {
		padding: 6px 10px 6px 12px;
		border-bottom: 1px solid var(--color-border-muted);
		display: flex;
		align-items: center;
		flex-shrink: 0;
	}
	.title {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-subtle);
		flex: 1;
	}
	.header-actions {
		display: flex;
		gap: 2px;
		align-items: center;
	}
	.header-btn {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		padding: 2px 5px;
		border-radius: 3px;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.header-btn:hover {
		color: var(--color-text);
		background: var(--color-overlay);
	}
	.header-btn.active {
		color: var(--color-accent);
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
	.tree-item:hover { background: var(--color-overlay); }
	.rename-item { cursor: default; }
	.rename-item:hover { background: none; }
	.inline-create { cursor: default; }
	.inline-create:hover { background: none; }
	.rename-input,
	.create-input {
		flex: 1;
		background: var(--color-base);
		border: 1px solid var(--color-accent);
		border-radius: 3px;
		color: var(--color-text);
		font-size: 13px;
		padding: 1px 4px;
		outline: none;
		font-family: inherit;
		min-width: 0;
	}
	.icon {
		font-size: 11px;
		width: 14px;
		text-align: center;
		flex-shrink: 0;
		color: var(--color-text-subtle);
	}
	.is-dir .icon { color: var(--color-accent); }
	.name { overflow: hidden; text-overflow: ellipsis; }
	.git-badge {
		margin-left: auto;
		font-size: 10px;
		font-weight: 700;
		font-family: 'SF Mono', 'Fira Code', monospace;
		padding: 0 4px;
		border-radius: 3px;
		flex-shrink: 0;
	}
	.git-modified .name { color: var(--color-warning); }
	.git-modified .git-badge { color: var(--color-warning); }
	.git-added .name { color: var(--color-success); }
	.git-added .git-badge { color: var(--color-success); }
	.git-deleted .name { color: var(--color-error); text-decoration: line-through; }
	.git-deleted .git-badge { color: var(--color-error); }
	.git-untracked .name { color: var(--color-text-subtle); }
	.git-untracked .git-badge { color: var(--color-text-subtle); }

	/* Search overlay sits on top of the tree */
	.search-overlay {
		position: absolute;
		inset: 0;
		background: var(--color-base);
		display: flex;
		flex-direction: column;
		z-index: 10;
	}
	.search-overlay-header {
		padding: 6px 10px 6px 12px;
		border-bottom: 1px solid var(--color-border-muted);
		display: flex;
		align-items: center;
		flex-shrink: 0;
	}
	.close-btn {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 12px;
		cursor: pointer;
		padding: 2px 5px;
		border-radius: 3px;
		line-height: 1;
	}
	.close-btn:hover {
		color: var(--color-text);
		background: var(--color-overlay);
	}
</style>
