<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { searchState, type SearchResultFile } from '$lib/stores/search';
	import { openFileInEditor, projectRoot } from '$lib/stores/editor';
	import { pendingEditorAction } from '$lib/stores/editor';

	let searchInput: HTMLInputElement;

	let query = $state('');
	let replacement = $state('');
	let isRegex = $state(false);
	let caseSensitive = $state(false);
	let includeGlob = $state('');
	let replaceMode = $state(false);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let results = $state<SearchResultFile[]>([]);
	let truncated = $state(false);
	let showOptions = $state(false);

	let searchTimer: ReturnType<typeof setTimeout> | null = null;

	export function focusInput() {
		searchInput?.focus();
	}

	export function setReplaceMode(enabled: boolean) {
		replaceMode = enabled;
		if (enabled) {
			setTimeout(() => searchInput?.focus(), 0);
		}
	}

	async function runSearch() {
		// Clear any pending debounce to avoid a redundant second run
		if (searchTimer) { clearTimeout(searchTimer); searchTimer = null; }
		const root = $projectRoot;
		if (!root || !query.trim()) {
			results = [];
			truncated = false;
			return;
		}
		loading = true;
		error = null;
		try {
			const response = await invoke<{
				matches: Array<{
					filePath: string;
					lineNumber: number;
					column: number;
					lineText: string;
					matchStart: number;
					matchEnd: number;
				}>;
				truncated: boolean;
			}>('search_in_project', {
				root,
				query,
				isRegex,
				caseSensitive,
				includeGlob,
				excludeGlob: '',
			});

			truncated = response.truncated;
			// Group by file
			const byFile = new Map<string, SearchResultFile>();
			for (const m of response.matches) {
				if (!byFile.has(m.filePath)) {
					byFile.set(m.filePath, {
						filePath: m.filePath,
						matches: [],
						expanded: true,
						selectedForReplace: true,
					});
				}
				byFile.get(m.filePath)!.matches.push(m);
			}
			results = Array.from(byFile.values());
		} catch (e) {
			error = String(e);
			results = [];
			truncated = false;
		} finally {
			loading = false;
		}
	}

	function onQueryInput() {
		if (searchTimer) clearTimeout(searchTimer);
		searchTimer = setTimeout(runSearch, 300);
	}

	function toggleExpand(filePath: string) {
		results = results.map(r =>
			r.filePath === filePath ? { ...r, expanded: !r.expanded } : r
		);
	}

	function toggleSelect(filePath: string) {
		results = results.map(r =>
			r.filePath === filePath ? { ...r, selectedForReplace: !r.selectedForReplace } : r
		);
	}

	async function openResult(filePath: string, lineNumber: number) {
		const root = $projectRoot;
		if (!root) return;
		const absPath = filePath.startsWith('/') ? filePath : `${root}/${filePath}`;
		const name = filePath.split('/').pop() ?? filePath;
		try {
			const content = await invoke<string>('read_file', { path: absPath });
			openFileInEditor(absPath, name, content);
			pendingEditorAction.set({ type: 'goto-line', line: lineNumber });
		} catch {
			// ignore
		}
	}

	async function replaceAll() {
		const root = $projectRoot;
		if (!root || !query.trim()) return;
		const selectedPaths = results
			.filter(r => r.selectedForReplace)
			.map(r => r.filePath);
		if (!selectedPaths.length) return;
		try {
			const result = await invoke<{ filesChanged: number; replacementsMade: number }>('replace_in_project', {
				root,
				query,
				replacement,
				isRegex,
				caseSensitive,
				filePaths: selectedPaths,
			});
			// Re-run search to show updated results
			await runSearch();
			error = null;
		} catch (e) {
			error = String(e);
		}
	}

	function getHighlightedLine(lineText: string, matchStart: number, matchEnd: number): [string, string, string] {
		return [
			lineText.slice(0, matchStart),
			lineText.slice(matchStart, matchEnd),
			lineText.slice(matchEnd),
		];
	}

	const totalMatchCount = $derived(results.reduce((sum, r) => sum + r.matches.length, 0));
</script>

<div class="search-panel">
	<div class="search-inputs">
		<div class="query-row">
			<input
				bind:this={searchInput}
				bind:value={query}
				class="search-input"
				placeholder="Search"
				oninput={onQueryInput}
				onkeydown={(e) => { if (e.key === 'Enter') runSearch(); }}
			/>
			<button
				class="toggle-btn"
				class:active={caseSensitive}
				onclick={() => { caseSensitive = !caseSensitive; runSearch(); }}
				title="Match Case (Alt+C)"
			>Aa</button>
			<button
				class="toggle-btn"
				class:active={isRegex}
				onclick={() => { isRegex = !isRegex; runSearch(); }}
				title="Use Regex (Alt+R)"
			>.*</button>
			<button
				class="toggle-btn"
				class:active={replaceMode}
				onclick={() => replaceMode = !replaceMode}
				title="Replace mode"
			>↔</button>
		</div>

		{#if replaceMode}
			<div class="replace-row">
				<input
					bind:value={replacement}
					class="search-input"
					placeholder="Replace"
				/>
				<button class="replace-btn" onclick={replaceAll}>Replace All</button>
			</div>
		{/if}

		<div class="options-row">
			<button class="options-toggle" onclick={() => showOptions = !showOptions}>
				{showOptions ? '▾' : '▸'} Filter
			</button>
			{#if showOptions}
				<input
					bind:value={includeGlob}
					class="glob-input"
					placeholder="files to include (e.g. *.ts)"
					oninput={onQueryInput}
				/>
			{/if}
		</div>
	</div>

	<div class="results-area">
		{#if loading}
			<div class="status-msg">Searching...</div>
		{:else if error}
			<div class="status-msg error">{error}</div>
		{:else if results.length === 0 && query.trim()}
			<div class="status-msg">No results</div>
		{:else if results.length > 0}
			<div class="results-header">
				{totalMatchCount} result{totalMatchCount !== 1 ? 's' : ''} in {results.length} file{results.length !== 1 ? 's' : ''}
				{#if truncated}<span class="truncated-notice"> — showing first 500</span>{/if}
			</div>
			{#each results as file (file.filePath)}
				<div class="file-group">
					<button
						class="file-header"
						onclick={() => toggleExpand(file.filePath)}
					>
						{#if replaceMode}
							<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
							<span
								class="select-check"
								onclick={(e) => { e.stopPropagation(); toggleSelect(file.filePath); }}
							>{file.selectedForReplace ? '☑' : '☐'}</span>
						{/if}
						<span class="expand-icon">{file.expanded ? '▾' : '▸'}</span>
						<span class="file-path">{file.filePath}</span>
						<span class="match-count">{file.matches.length}</span>
					</button>
					{#if file.expanded}
						{#each file.matches as match}
							{@const trimOffset = match.lineText.length - match.lineText.trimStart().length}
						{@const [pre, hit, post] = getHighlightedLine(match.lineText.trimStart(), Math.max(0, match.matchStart - trimOffset), Math.max(0, match.matchEnd - trimOffset))}
							<button
								class="match-line"
								onclick={() => openResult(file.filePath, match.lineNumber)}
							>
								<span class="line-num">{match.lineNumber}</span>
								<span class="line-content">{pre}<mark class="match-highlight">{hit}</mark>{post}</span>
							</button>
						{/each}
					{/if}
				</div>
			{/each}
		{/if}
	</div>
</div>

<style>
	.search-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}
	.search-inputs {
		padding: 8px 8px 4px;
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.query-row,
	.replace-row {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.search-input {
		flex: 1;
		padding: 4px 8px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		color: var(--color-text);
		font-size: 12px;
		outline: none;
		font-family: inherit;
		min-width: 0;
	}
	.search-input:focus {
		border-color: var(--color-accent);
	}
	.toggle-btn {
		padding: 3px 6px;
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		flex-shrink: 0;
	}
	.toggle-btn:hover {
		color: var(--color-text);
	}
	.toggle-btn.active {
		color: var(--color-accent);
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}
	.replace-btn {
		padding: 3px 8px;
		background: var(--color-accent);
		border: none;
		border-radius: 4px;
		color: white;
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		white-space: nowrap;
		flex-shrink: 0;
	}
	.replace-btn:hover {
		filter: brightness(1.1);
	}
	.options-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.options-toggle {
		background: none;
		border: none;
		color: var(--color-text-subtle);
		font-size: 10px;
		cursor: pointer;
		padding: 2px 0;
	}
	.options-toggle:hover {
		color: var(--color-text);
	}
	.glob-input {
		flex: 1;
		padding: 3px 6px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		color: var(--color-text);
		font-size: 11px;
		outline: none;
		font-family: inherit;
	}
	.results-area {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
	}
	.status-msg {
		padding: 12px;
		font-size: 12px;
		color: var(--color-text-subtle);
	}
	.status-msg.error {
		color: var(--color-error);
		font-size: 11px;
	}
	.results-header {
		padding: 4px 10px 2px;
		font-size: 10px;
		color: var(--color-text-subtle);
		font-weight: 600;
	}
	.truncated-notice {
		color: var(--color-warning);
		font-weight: 400;
	}
	.file-group {
		border-bottom: 1px solid var(--color-border-muted);
	}
	.file-header {
		display: flex;
		align-items: center;
		gap: 4px;
		width: 100%;
		padding: 4px 8px;
		background: var(--color-surface);
		border: none;
		color: var(--color-text);
		font-size: 11px;
		cursor: pointer;
		text-align: left;
		font-weight: 600;
	}
	.file-header:hover {
		background: var(--color-overlay);
	}
	.select-check {
		font-size: 12px;
		color: var(--color-accent);
		cursor: pointer;
		padding: 0 2px;
	}
	.expand-icon {
		color: var(--color-text-subtle);
		font-size: 10px;
		flex-shrink: 0;
	}
	.file-path {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 11px;
		color: var(--color-text);
		direction: rtl;
		text-align: left;
	}
	.match-count {
		font-size: 10px;
		color: var(--color-text-subtle);
		background: var(--color-overlay);
		padding: 1px 6px;
		border-radius: 8px;
		flex-shrink: 0;
	}
	.match-line {
		display: flex;
		align-items: baseline;
		gap: 8px;
		padding: 2px 8px 2px 20px;
		width: 100%;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 11px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		cursor: pointer;
		text-align: left;
		overflow: hidden;
	}
	.match-line:hover {
		background: var(--color-overlay);
	}
	.line-num {
		color: var(--color-text-subtle);
		font-size: 10px;
		flex-shrink: 0;
		min-width: 28px;
		text-align: right;
	}
	.line-content {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	:global(.match-highlight) {
		background: color-mix(in srgb, var(--color-warning) 30%, transparent);
		color: var(--color-text);
		border-radius: 2px;
	}
</style>
