<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { showQuickOpen, quickOpenMode } from '$lib/stores/layout';
	import { fileIndex, fuzzyScore, type FileEntry } from '$lib/stores/fileIndex';
	import { openFileInEditor, recentFiles, pendingEditorAction, projectRoot } from '$lib/stores/editor';

	let inputEl: HTMLInputElement;
	let query = $state('');
	let selectedIndex = $state(0);

	const RECENT_LIMIT = 8;

	// Resolve mode from query prefix
	const mode = $derived.by(() => {
		if ($quickOpenMode === 'line' || query.startsWith(':')) return 'line';
		if ($quickOpenMode === 'symbol' || query.startsWith('@')) return 'symbol';
		return 'file';
	});

	const effectiveQuery = $derived.by(() => {
		if (mode === 'line') return query.replace(/^:/, '');
		if (mode === 'symbol') return query.replace(/^@/, '');
		return query;
	});

	const results = $derived.by(() => {
		if (mode !== 'file') return [];
		const q = effectiveQuery.trim();
		if (!q) return [];
		return $fileIndex
			.map(entry => ({ entry, score: fuzzyScore(q, entry) }))
			.filter(x => x.score > 0)
			.sort((a, b) => b.score - a.score)
			.slice(0, 20)
			.map(x => x.entry);
	});

	const showRecent = $derived(mode === 'file' && effectiveQuery.trim() === '');

	const displayItems = $derived.by((): FileEntry[] => {
		if (showRecent) {
			const root = $projectRoot ?? '';
			return $recentFiles
				.slice(0, RECENT_LIMIT)
				.map(r => {
					const relPath = root && r.path.startsWith(root)
						? r.path.slice(root.length).replace(/^\//, '')
						: r.path;
					return { name: r.name, path: relPath, absPath: r.path };
				});
		}
		return results;
	});

	$effect(() => {
		if ($showQuickOpen) {
			query = $quickOpenMode === 'line' ? ':' : $quickOpenMode === 'symbol' ? '@' : '';
			selectedIndex = 0;
			// Focus input after DOM update
			setTimeout(() => inputEl?.focus(), 0);
		}
	});

	$effect(() => {
		// Reset selection when results change
		selectedIndex = 0;
	});

	function close() {
		showQuickOpen.set(false);
		quickOpenMode.set('file');
		query = '';
	}

	function selectItem(item: FileEntry) {
		if (mode === 'file') {
			const root = $projectRoot;
			const absPath = root ? (item.absPath.startsWith('/') ? item.absPath : `${root}/${item.path}`) : item.absPath;
			invoke<string>('read_file', { path: absPath })
				.then(content => {
					openFileInEditor(absPath, item.name, content);
					close();
				})
				.catch(() => close());
		}
	}

	function confirmLine() {
		const lineNum = parseInt(effectiveQuery.trim(), 10);
		if (!isNaN(lineNum) && lineNum > 0) {
			pendingEditorAction.set({ type: 'goto-line', line: lineNum });
		}
		close();
	}

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			close();
			return;
		}
		if (mode === 'line') {
			if (e.key === 'Enter') {
				e.preventDefault();
				confirmLine();
			}
			return;
		}
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, displayItems.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			if (displayItems[selectedIndex]) {
				selectItem(displayItems[selectedIndex]);
			}
		}
	}

	function getIcon(name: string): string {
		const ext = name.split('.').pop() ?? '';
		const icons: Record<string, string> = {
			ts: '◇', js: '◇', svelte: '◆', rs: '⚙', json: '{}', css: '◈',
			html: '◁', md: '¶', toml: '≡',
		};
		return icons[ext] ?? '○';
	}

	function getDir(path: string): string {
		const parts = path.split('/');
		if (parts.length <= 1) return '';
		return parts.slice(0, -1).join('/');
	}
</script>

{#if $showQuickOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={close} role="dialog" aria-modal="true">
		<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
		<div class="palette" onclick={(e) => e.stopPropagation()}>
			<div class="input-row">
				<input
					bind:this={inputEl}
					bind:value={query}
					class="search-input"
					placeholder={mode === 'line' ? 'Go to line...' : mode === 'symbol' ? 'Go to symbol...' : 'Go to file...'}
					onkeydown={onKeydown}
					autocomplete="off"
					spellcheck={false}
				/>
				<button class="close-btn" onclick={close}>&times;</button>
			</div>

			{#if mode === 'line'}
				<div class="results">
					<div class="result-item hint">
						<span class="hint-text">Type a line number and press Enter</span>
					</div>
				</div>
			{:else if displayItems.length > 0}
				<div class="results">
					{#if showRecent}
						<div class="section-label">Recent</div>
					{/if}
					{#each displayItems as item, i (item.absPath || item.path)}
						<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
						<div
							class="result-item"
							class:selected={i === selectedIndex}
							onclick={() => selectItem(item)}
							onmouseenter={() => selectedIndex = i}
							role="option"
							aria-selected={i === selectedIndex}
						>
							<span class="file-icon">{getIcon(item.name)}</span>
							<span class="file-name">{item.name}</span>
							<span class="file-dir">{getDir(item.path)}</span>
						</div>
					{/each}
				</div>
			{:else if effectiveQuery.trim()}
				<div class="results">
					<div class="result-item hint">
						<span class="hint-text">No files found</span>
					</div>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 500;
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
	}
	.palette {
		width: 560px;
		max-width: 90vw;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
		overflow: hidden;
	}
	.input-row {
		display: flex;
		align-items: center;
		padding: 0 12px;
		border-bottom: 1px solid var(--color-border-muted);
	}
	.search-input {
		flex: 1;
		padding: 12px 0;
		background: none;
		border: none;
		outline: none;
		color: var(--color-text);
		font-size: 14px;
		font-family: inherit;
	}
	.search-input::placeholder {
		color: var(--color-text-subtle);
	}
	.close-btn {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 18px;
		cursor: pointer;
		padding: 4px 6px;
		line-height: 1;
	}
	.close-btn:hover {
		color: var(--color-text);
	}
	.results {
		max-height: 360px;
		overflow-y: auto;
		padding: 4px 0;
	}
	.section-label {
		padding: 4px 12px 2px;
		font-size: 10px;
		font-weight: 700;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}
	.result-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 5px 12px;
		cursor: pointer;
		transition: background 0.1s;
	}
	.result-item:hover,
	.result-item.selected {
		background: var(--color-overlay);
	}
	.result-item.hint {
		cursor: default;
	}
	.result-item.hint:hover {
		background: none;
	}
	.hint-text {
		font-size: 12px;
		color: var(--color-text-subtle);
		padding: 8px 0;
	}
	.file-icon {
		font-size: 11px;
		color: var(--color-text-subtle);
		width: 14px;
		flex-shrink: 0;
	}
	.file-name {
		font-size: 13px;
		color: var(--color-text);
		flex-shrink: 0;
	}
	.file-dir {
		font-size: 11px;
		color: var(--color-text-subtle);
		margin-left: auto;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		direction: rtl;
		text-align: left;
		max-width: 240px;
	}
</style>
