<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from '$lib/stores/toast';
	import {
		activeFile,
		updateFileContent,
		markFileSaved,
		activeFilePath,
		projectRoot,
		selection as selectionStore,
		cursorLine as cursorLineStore,
	} from '$lib/stores/editor';
	import { pendingEdits, updateEditStatus } from '$lib/stores/agent';
	import InlineEditOverlay from './InlineEditOverlay.svelte';
	import AgentEditBar from './AgentEditBar.svelte';
	import { activeSessionId, activeSession, trackDevEdit } from '$lib/stores/agentTerminal';
	import { isAgentMode } from '$lib/stores/settings';
	import { sessionReview, activeReview, type MergeResult } from '$lib/stores/sessionReview';
	import { setMonacoInstance, activeTheme, generateMonacoTheme, applyTheme } from '$lib/themes';
	import type * as Monaco from 'monaco-editor';

	let editorContainer: HTMLDivElement;
	let diffContainer: HTMLDivElement;
	let mergeContainer: HTMLDivElement;
	let yoursContainer: HTMLDivElement;
	let theirsContainer: HTMLDivElement;
	let editor: Monaco.editor.IStandaloneCodeEditor | undefined;
	let diffEditor: Monaco.editor.IStandaloneDiffEditor | undefined;
	let mergeEditor: Monaco.editor.IStandaloneCodeEditor | undefined;
	let yoursDiffEditor: Monaco.editor.IStandaloneDiffEditor | undefined;
	let theirsDiffEditor: Monaco.editor.IStandaloneDiffEditor | undefined;
	let monaco: typeof Monaco;
	let decorationCollection: Monaco.editor.IEditorDecorationsCollection | undefined;
	let diffInline = $state(false);
	let inReview = $derived(!!$activeReview?.reviewMode && !!$activeReview?.selectedFile);
	let isConflict = $derived(
		!!$activeReview?.selectedFile &&
		$activeReview?.fileDecisions.get($activeReview.selectedFile) === 'conflict'
	);
	let mergeView = $state<'result' | 'yours' | 'theirs'>('result');

	function getLanguage(filename: string): string {
		const ext = filename.split('.').pop() ?? '';
		const map: Record<string, string> = {
			ts: 'typescript', js: 'javascript', svelte: 'html', rs: 'rust',
			json: 'json', css: 'css', html: 'html', md: 'markdown',
			toml: 'toml', py: 'python', go: 'go', yaml: 'yaml', yml: 'yaml',
			sh: 'shell', bash: 'shell', zsh: 'shell',
		};
		return map[ext] ?? 'plaintext';
	}

	function updateDecorations() {
		const file = $activeFile;
		if (!editor || !monaco || !file) {
			decorationCollection?.clear();
			return;
		}

		const fileEdits = $pendingEdits.filter(
			e => e.filePath === file.path && e.status === 'pending'
		);

		if (fileEdits.length === 0) {
			decorationCollection?.clear();
			return;
		}

		const content = editor.getValue();
		const decorations: Monaco.editor.IModelDeltaDecoration[] = [];

		for (const edit of fileEdits) {
			if (edit.oldContent) {
				const startIdx = content.indexOf(edit.oldContent);
				if (startIdx !== -1) {
					const model = editor.getModel();
					if (!model) continue;
					const startPos = model.getPositionAt(startIdx);
					const endPos = model.getPositionAt(startIdx + edit.oldContent.length);
					decorations.push({
						range: new monaco.Range(startPos.lineNumber, 1, endPos.lineNumber, 1000),
						options: {
							isWholeLine: true,
							className: 'diff-delete-line',
							overviewRuler: {
								color: '#f38ba860',
								position: monaco.editor.OverviewRulerLane.Full,
							},
							minimap: {
								color: '#f38ba860',
								position: monaco.editor.MinimapPosition.Inline,
							},
						},
					});
				}
			}
		}

		if (decorationCollection) {
			decorationCollection.set(decorations);
		} else {
			decorationCollection = editor.createDecorationsCollection(decorations);
		}
	}

	onMount(async () => {
		monaco = await import('monaco-editor');

		// Register Monaco instance for theme system
		setMonacoInstance(monaco);

		// Define initial theme from theme store
		const currentTheme = $activeTheme;
		const monacoTheme = generateMonacoTheme(currentTheme);
		monaco.editor.defineTheme('tyck-theme', monacoTheme);

		self.MonacoEnvironment = {
			getWorker(_: unknown, label: string) {
				switch (label) {
					case 'json':
						return new Worker(new URL('monaco-editor/esm/vs/language/json/json.worker.js', import.meta.url), { type: 'module' });
					case 'css':
					case 'scss':
					case 'less':
						return new Worker(new URL('monaco-editor/esm/vs/language/css/css.worker.js', import.meta.url), { type: 'module' });
					case 'html':
					case 'handlebars':
					case 'razor':
						return new Worker(new URL('monaco-editor/esm/vs/language/html/html.worker.js', import.meta.url), { type: 'module' });
					case 'typescript':
					case 'javascript':
						return new Worker(new URL('monaco-editor/esm/vs/language/typescript/ts.worker.js', import.meta.url), { type: 'module' });
					default:
						return new Worker(new URL('monaco-editor/esm/vs/editor/editor.worker.js', import.meta.url), { type: 'module' });
				}
			}
		};

		editor = monaco.editor.create(editorContainer, {
			value: '',
			language: 'plaintext',
			theme: 'tyck-theme',
			fontSize: 12,
			fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
			minimap: { enabled: true, maxColumn: 80 },
			lineNumbers: 'on',
			renderWhitespace: 'selection',
			bracketPairColorization: { enabled: true },
			smoothScrolling: true,
			cursorBlinking: 'smooth',
			cursorSmoothCaretAnimation: 'on',
			padding: { top: 8 },
			scrollBeyondLastLine: false,
			automaticLayout: true,
			wordWrap: 'off',
			tabSize: 2,
		});

		// Save on Ctrl+S / Cmd+S
		editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, async () => {
			const file = $activeFile;
			if (!file) return;
			const value = editor!.getValue();
			try {
				await invoke('write_file', { path: file.path, content: value });
				markFileSaved(file.path);
				// Track edits while agent is paused for context on resume
				const session = $activeSession;
				if (session?.status === 'paused') {
					trackDevEdit(session.id, file.path);
				}
			} catch (e) {
				toast.error(`Failed to save ${file.name}: ${e}`);
			}
		});

		// Accept diffs: Cmd+Shift+Enter
		editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.Enter, () => {
			const file = $activeFile;
			const fileEdits = $pendingEdits.filter(
				e => file && e.filePath === file.path && e.status === 'pending'
			);
			for (const edit of fileEdits) {
				updateEditStatus(edit.toolId, 'accepted');
				if (edit.oldContent && edit.newContent) {
					const model = editor!.getModel();
					if (model) {
						const content = model.getValue();
						const newContent = content.replace(edit.oldContent, edit.newContent);
						model.setValue(newContent);
					}
				}
			}
		});

		// Reject diffs: Escape
		editor.addCommand(monaco.KeyCode.Escape, () => {
			const file = $activeFile;
			const fileEdits = $pendingEdits.filter(
				e => file && e.filePath === file.path && e.status === 'pending'
			);
			if (fileEdits.length > 0) {
				for (const edit of fileEdits) {
					updateEditStatus(edit.toolId, 'rejected');
				}
			}
		});

		// Track content changes
		editor.onDidChangeModelContent(() => {
			const file = $activeFile;
			if (file) {
				updateFileContent(file.path, editor!.getValue());
			}
		});

		// Track selection changes
		editor.onDidChangeCursorSelection((e) => {
			const sel = editor!.getModel()?.getValueInRange(e.selection) || '';
			selectionStore.set(sel);
		});

		// Track cursor position
		editor.onDidChangeCursorPosition((e) => {
			cursorLineStore.set(e.position.lineNumber);
		});
	});

	// React to file changes
	$effect(() => {
		const file = $activeFile;
		if (!editor || !monaco) return;
		if (file) {
			const model = editor.getModel();
			const lang = getLanguage(file.name);
			if (model) {
				monaco.editor.setModelLanguage(model, lang);
				if (model.getValue() !== file.content) {
					model.setValue(file.content);
				}
			}
		} else {
			const model = editor.getModel();
			if (model && model.getValue() !== '') {
				model.setValue('');
			}
		}
	});

	// React to pending edits
	$effect(() => {
		$pendingEdits;
		$activeFile;
		updateDecorations();
	});

	// React to review mode + selected file → show diff or merge editor
	$effect(() => {
		const review = $activeReview;
		if (!monaco) return;

		if (review?.reviewMode && review?.selectedFile) {
			const sid = review.sessionId;
			const file = review.selectedFile;
			const fileIsConflict = review.fileDecisions.get(file) === 'conflict';

			if (fileIsConflict) {
				tick().then(() => showMergeEditor(sid, file));
			} else {
				hideMergeEditor();
				tick().then(() => showDiffEditor(sid, file));
			}
		} else {
			hideDiffEditor();
			hideMergeEditor();
		}
	});

	// ── Normal diff view (non-conflict) ──

	async function showDiffEditor(sessionId: string, filePath: string) {
		if (!monaco || !diffContainer) return;

		try {
			const [originalContent, modifiedContent] = await Promise.all([
				invoke<string>('get_file_at_base', { sessionId, filePath }).catch(() => ''),
				invoke<string>('get_file_from_worktree', { sessionId, filePath }).catch(() => ''),
			]);

			if (diffEditor) {
				diffEditor.dispose();
				diffEditor = undefined;
			}

			const lang = getLanguage(filePath.split('/').pop() ?? '');

			diffEditor = monaco.editor.createDiffEditor(diffContainer, {
				theme: 'tyck-theme',
				fontSize: 12,
				fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
				readOnly: true,
				renderSideBySide: !diffInline,
				automaticLayout: true,
				scrollBeyondLastLine: false,
				padding: { top: 8 },
				minimap: { enabled: false },
			});

			diffEditor.setModel({
				original: monaco.editor.createModel(originalContent, lang),
				modified: monaco.editor.createModel(modifiedContent, lang),
			});
		} catch (e) {
			console.warn('showDiffEditor failed:', e);
		}
	}

	function hideDiffEditor() {
		if (diffEditor) {
			diffEditor.dispose();
			diffEditor = undefined;
		}
	}

	// ── Three-way merge editor (conflict) ──

	let currentMerge: MergeResult | null = null;

	async function showMergeEditor(sessionId: string, filePath: string) {
		if (!monaco) return;

		hideDiffEditor();
		hideMergeEditor();

		try {
			currentMerge = await invoke<MergeResult>('three_way_merge', { sessionId, filePath });
			mergeView = 'result';

			await tick();
			if (!mergeContainer) return;

			const lang = getLanguage(filePath.split('/').pop() ?? '');

			// Main merge editor — editable, shows merged content with conflict markers
			mergeEditor = monaco.editor.create(mergeContainer, {
				value: currentMerge.content,
				language: lang,
				theme: 'tyck-theme',
				fontSize: 12,
				fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
				automaticLayout: true,
				scrollBeyondLastLine: false,
				padding: { top: 8 },
				minimap: { enabled: false },
			});

			// Highlight conflict markers
			highlightConflictMarkers();
		} catch (e) {
			console.warn('showMergeEditor failed:', e);
		}
	}

	function highlightConflictMarkers() {
		if (!mergeEditor || !monaco) return;
		const model = mergeEditor.getModel();
		if (!model) return;

		const decorations: Monaco.editor.IModelDeltaDecoration[] = [];
		const lineCount = model.getLineCount();

		for (let i = 1; i <= lineCount; i++) {
			const line = model.getLineContent(i);
			if (line.startsWith('<<<<<<<')) {
				decorations.push({
					range: new monaco.Range(i, 1, i, 1000),
					options: {
						isWholeLine: true,
						className: 'merge-marker-yours',
						glyphMarginClassName: 'merge-glyph-yours',
					},
				});
			} else if (line.startsWith('|||||||')) {
				decorations.push({
					range: new monaco.Range(i, 1, i, 1000),
					options: {
						isWholeLine: true,
						className: 'merge-marker-base',
					},
				});
			} else if (line.startsWith('=======')) {
				decorations.push({
					range: new monaco.Range(i, 1, i, 1000),
					options: {
						isWholeLine: true,
						className: 'merge-marker-separator',
					},
				});
			} else if (line.startsWith('>>>>>>>')) {
				decorations.push({
					range: new monaco.Range(i, 1, i, 1000),
					options: {
						isWholeLine: true,
						className: 'merge-marker-theirs',
						glyphMarginClassName: 'merge-glyph-theirs',
					},
				});
			}
		}

		mergeEditor.createDecorationsCollection(decorations);
	}

	function showYoursDiff() {
		mergeView = 'yours';
		disposeSideDiffs();
		if (!monaco || !currentMerge || !yoursContainer) return;

		const lang = getLanguage(($activeReview?.selectedFile ?? '').split('/').pop() ?? '');
		yoursDiffEditor = monaco.editor.createDiffEditor(yoursContainer, {
			theme: 'tyck-theme',
			fontSize: 12,
			fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
			readOnly: true,
			renderSideBySide: true,
			automaticLayout: true,
			scrollBeyondLastLine: false,
			padding: { top: 8 },
			minimap: { enabled: false },
		});
		yoursDiffEditor.setModel({
			original: monaco.editor.createModel(currentMerge.baseContent, lang),
			modified: monaco.editor.createModel(currentMerge.yoursContent, lang),
		});
	}

	function showTheirsDiff() {
		mergeView = 'theirs';
		disposeSideDiffs();
		if (!monaco || !currentMerge || !theirsContainer) return;

		const lang = getLanguage(($activeReview?.selectedFile ?? '').split('/').pop() ?? '');
		theirsDiffEditor = monaco.editor.createDiffEditor(theirsContainer, {
			theme: 'tyck-theme',
			fontSize: 12,
			fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
			readOnly: true,
			renderSideBySide: true,
			automaticLayout: true,
			scrollBeyondLastLine: false,
			padding: { top: 8 },
			minimap: { enabled: false },
		});
		theirsDiffEditor.setModel({
			original: monaco.editor.createModel(currentMerge.baseContent, lang),
			modified: monaco.editor.createModel(currentMerge.theirsContent, lang),
		});
	}

	function showResultView() {
		mergeView = 'result';
		disposeSideDiffs();
	}

	function disposeSideDiffs() {
		if (yoursDiffEditor) { yoursDiffEditor.dispose(); yoursDiffEditor = undefined; }
		if (theirsDiffEditor) { theirsDiffEditor.dispose(); theirsDiffEditor = undefined; }
	}

	function hideMergeEditor() {
		if (mergeEditor) { mergeEditor.dispose(); mergeEditor = undefined; }
		disposeSideDiffs();
		currentMerge = null;
	}

	/** Accept "yours" for all conflicts (keep workspace version) */
	function acceptYours() {
		if (!mergeEditor || !currentMerge) return;
		mergeEditor.setValue(currentMerge.yoursContent);
	}

	/** Accept "theirs" for all conflicts (use agent version) */
	function acceptTheirs() {
		if (!mergeEditor || !currentMerge) return;
		mergeEditor.setValue(currentMerge.theirsContent);
	}

	/** Save the resolved merge result */
	async function onResolve() {
		const review = $activeReview;
		const sid = $activeSessionId;
		if (!mergeEditor || !review?.selectedFile || !sid) return;

		const content = mergeEditor.getValue();

		// Check for remaining conflict markers
		if (content.includes('<<<<<<<') || content.includes('>>>>>>>')) {
			if (!confirm('The file still contains conflict markers. Resolve anyway?')) return;
		}

		await sessionReview.resolveConflict(sid, review.selectedFile, content);
		hideMergeEditor();
	}

	// ── Normal review actions ──

	function toggleDiffLayout() {
		diffInline = !diffInline;
		if (diffEditor) {
			diffEditor.updateOptions({ renderSideBySide: !diffInline });
		}
	}

	function onAcceptReviewFile() {
		const review = $activeReview;
		const sid = $activeSessionId;
		if (review?.selectedFile && sid) {
			sessionReview.acceptFile(sid, review.selectedFile);
		}
	}

	function onRejectReviewFile() {
		const review = $activeReview;
		const sid = $activeSessionId;
		if (review?.selectedFile && sid) {
			sessionReview.rejectFile(sid, review.selectedFile);
		}
	}

	onDestroy(() => {
		decorationCollection?.clear();
		editor?.dispose();
		diffEditor?.dispose();
		mergeEditor?.dispose();
		disposeSideDiffs();
	});
</script>

<div class="focus-zone">
	{#if inReview && isConflict}
		<!-- Merge conflict editor -->
		<div class="review-diff-wrapper">
			<div class="diff-toolbar merge-toolbar">
				<span class="diff-file-name conflict-label">CONFLICT: {$activeReview?.selectedFile}</span>
				<div class="diff-toolbar-actions">
					<div class="merge-tabs">
						<button class="merge-tab" class:active={mergeView === 'result'} onclick={showResultView}>Result</button>
						<button class="merge-tab" class:active={mergeView === 'yours'} onclick={() => { tick().then(showYoursDiff); }}>Yours</button>
						<button class="merge-tab" class:active={mergeView === 'theirs'} onclick={() => { tick().then(showTheirsDiff); }}>Theirs</button>
					</div>
					<button class="diff-tool-btn" onclick={acceptYours}>Use Yours</button>
					<button class="diff-tool-btn" onclick={acceptTheirs}>Use Theirs</button>
					<button class="diff-tool-btn resolve" onclick={onResolve}>Resolve</button>
				</div>
			</div>
			<div class="merge-editor-area">
				<div class="merge-editor-container" class:hidden={mergeView !== 'result'} bind:this={mergeContainer}></div>
				<div class="merge-editor-container" class:hidden={mergeView !== 'yours'} bind:this={yoursContainer}></div>
				<div class="merge-editor-container" class:hidden={mergeView !== 'theirs'} bind:this={theirsContainer}></div>
			</div>
		</div>
	{:else if inReview}
		<!-- Review mode: diff editor -->
		<div class="review-diff-wrapper">
			<div class="diff-toolbar">
				<span class="diff-file-name">{$activeReview?.selectedFile}</span>
				<div class="diff-toolbar-actions">
					<button class="diff-tool-btn" onclick={toggleDiffLayout}>
						{diffInline ? 'Side-by-Side' : 'Inline'}
					</button>
					<button class="diff-tool-btn accept" onclick={onAcceptReviewFile}>Accept</button>
					<button class="diff-tool-btn reject" onclick={onRejectReviewFile}>Reject</button>
				</div>
			</div>
			<div class="diff-editor-container" bind:this={diffContainer}></div>
		</div>
	{/if}

	<!-- Editor always in DOM (hidden during review) so Monaco stays alive -->
	<div class="editor-layer" class:hidden={inReview}>
		{#if !$activeFile && !inReview}
			<div class="empty-state">
				<div class="logo">tyck</div>
				<p>Open a file from the explorer to start editing</p>
			</div>
		{/if}
		<AgentEditBar />
		<InlineEditOverlay />
		<div class="editor-container" class:hidden={!$activeFile} bind:this={editorContainer}></div>
	</div>
</div>

<style>
	.focus-zone {
		height: 100%;
		position: relative;
		background: var(--color-base);
	}
	.editor-layer {
		width: 100%;
		height: 100%;
		position: relative;
	}
	.editor-layer.hidden {
		display: none;
	}
	.editor-container {
		width: 100%;
		height: 100%;
	}
	.editor-container.hidden {
		display: none;
	}
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--color-text-subtle);
		gap: 12px;
	}
	.logo {
		font-size: 48px;
		font-weight: 800;
		color: var(--color-overlay);
		letter-spacing: 2px;
	}
	.empty-state p {
		font-size: 14px;
	}
	.diff-bar {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		background: color-mix(in srgb, var(--color-warning) 10%, transparent);
		border-top: 1px solid color-mix(in srgb, var(--color-warning) 25%, transparent);
		padding: 6px 12px;
		display: flex;
		justify-content: space-between;
		align-items: center;
		z-index: 10;
	}
	.diff-bar-text {
		font-size: 12px;
		color: var(--color-warning);
		font-weight: 600;
	}
	.diff-bar-hint {
		font-size: 11px;
		color: var(--color-text-subtle);
	}
	:global(.diff-delete-line) {
		background: color-mix(in srgb, var(--color-error) 10%, transparent) !important;
	}
	.diff-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 12px;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
	}
	.merge-toolbar {
		border-bottom-color: color-mix(in srgb, var(--color-warning) 25%, transparent);
	}
	.diff-file-name {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.conflict-label {
		color: var(--color-warning);
	}
	.diff-toolbar-actions {
		display: flex;
		gap: 4px;
		flex-shrink: 0;
		align-items: center;
	}
	.diff-tool-btn {
		padding: 3px 10px;
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		background: none;
		color: var(--color-text-muted);
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s;
	}
	.diff-tool-btn:hover {
		border-color: var(--color-border);
		color: var(--color-text);
	}
	.diff-tool-btn.accept {
		color: var(--color-success);
	}
	.diff-tool-btn.accept:hover {
		border-color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 10%, transparent);
	}
	.diff-tool-btn.reject {
		color: var(--color-error);
	}
	.diff-tool-btn.reject:hover {
		border-color: var(--color-error);
		background: color-mix(in srgb, var(--color-error) 10%, transparent);
	}
	.diff-tool-btn.resolve {
		color: var(--color-success);
		border-color: color-mix(in srgb, var(--color-success) 25%, transparent);
	}
	.diff-tool-btn.resolve:hover {
		border-color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 10%, transparent);
	}
	.review-diff-wrapper {
		display: flex;
		flex-direction: column;
		height: 100%;
		width: 100%;
	}
	.diff-editor-container {
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	/* Merge editor */
	.merge-tabs {
		display: flex;
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		overflow: hidden;
		margin-right: 8px;
	}
	.merge-tab {
		padding: 3px 10px;
		border: none;
		background: none;
		color: var(--color-text-subtle);
		font-size: 10px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s;
	}
	.merge-tab:hover {
		color: var(--color-text-muted);
		background: var(--color-base);
	}
	.merge-tab.active {
		color: var(--color-text);
		background: var(--color-overlay);
	}
	.merge-editor-area {
		flex: 1;
		min-height: 0;
		overflow: hidden;
		position: relative;
	}
	.merge-editor-container {
		position: absolute;
		inset: 0;
	}
	.merge-editor-container.hidden {
		display: none;
	}

	/* Conflict marker highlights */
	:global(.merge-marker-yours) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent) !important;
	}
	:global(.merge-marker-base) {
		background: color-mix(in srgb, var(--color-text-subtle) 10%, transparent) !important;
	}
	:global(.merge-marker-separator) {
		background: color-mix(in srgb, var(--color-border) 20%, transparent) !important;
	}
	:global(.merge-marker-theirs) {
		background: color-mix(in srgb, var(--color-success) 10%, transparent) !important;
	}
	:global(.merge-glyph-yours) {
		background: var(--color-accent);
		width: 4px !important;
		margin-left: 3px;
		border-radius: 2px;
	}
	:global(.merge-glyph-theirs) {
		background: var(--color-success);
		width: 4px !important;
		margin-left: 3px;
		border-radius: 2px;
	}
</style>
