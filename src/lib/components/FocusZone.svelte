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
	import InlineEditOverlay from './InlineEditOverlay.svelte';
	import AgentEditBar from './AgentEditBar.svelte';
	import { activeSessionId, activeSession, trackDevEdit } from '$lib/stores/agentTerminal';
	import { isAgentMode } from '$lib/stores/settings';
	import { sessionReview, activeReview, type MergeResult } from '$lib/stores/sessionReview';
	import { setMonacoInstance, activeTheme, generateMonacoTheme, applyTheme } from '$lib/themes';
	import { lspClientManager } from '$lib/lsp/LspClientManager';
	import type * as Monaco from 'monaco-editor';

	let editorContainer: HTMLDivElement;
	let diffContainer: HTMLDivElement;
	let mergeContainer: HTMLDivElement;
	let yoursContainer: HTMLDivElement;
	let theirsContainer: HTMLDivElement;
	let editor: Monaco.editor.IStandaloneCodeEditor | undefined;
	let settingContent = false;
	let diffEditor: Monaco.editor.IStandaloneDiffEditor | undefined;
	let mergeEditor: Monaco.editor.IStandaloneCodeEditor | undefined;
	let yoursDiffEditor: Monaco.editor.IStandaloneDiffEditor | undefined;
	let theirsDiffEditor: Monaco.editor.IStandaloneDiffEditor | undefined;
	let monaco: typeof Monaco;
	let diffInline = $state(false);
	let mergeView = $state<'result' | 'yours' | 'theirs'>('result');
	/** Key of the file currently loaded in the diff editor (sessionId:relPath). Prevents re-creation on every store update. */
	let currentDiffKey = $state<string | null>(null);
	/** Line change the user is currently hovering over in the diff editor. */
	let hoveredChange = $state<Monaco.editor.ILineChange | null>(null);
	/** Pixel top of the hovered chunk within the diff-wrapper (toolbar + editor offset). */
	let chunkMenuTop = $state<number | null>(null);
	let hideChunkTimer: ReturnType<typeof setTimeout> | null = null;
	/** Snapshot-based undo/redo for chunk accept/reject operations. */
	interface DiffSnapshot { orig: string; mod: string; }
	let chunkUndoStack: DiffSnapshot[] = [];
	let chunkRedoStack: DiffSnapshot[] = [];

	/** Relative path of the active file within the session (mainCwd or worktreePath prefix). */
	let activeFileRelPath = $derived.by(() => {
		const file = $activeFile;
		const review = $activeReview;
		if (!file || !review) return null;
		// Normalize paths to remove any trailing slash
		const mainCwd = review.mainCwd.replace(/\/+$/, '');
		const wtPath = review.worktreePath.replace(/\/+$/, '');
		const mainPrefix = mainCwd + '/';
		if (file.path.startsWith(mainPrefix)) return file.path.slice(mainPrefix.length);
		const wtPrefix = wtPath + '/';
		if (file.path.startsWith(wtPrefix)) return file.path.slice(wtPrefix.length);
		return null;
	});

	/** True when the active file has pending agent changes in the current session. */
	let inReview = $derived.by(() => {
		if (!$isAgentMode) return false;
		const rel = activeFileRelPath;
		const review = $activeReview;
		if (!rel || !review) return false;
		return review.diffs.some(d => d.path === rel);
	});

	let isConflict = $derived.by(() => {
		const rel = activeFileRelPath;
		const review = $activeReview;
		if (!rel || !review) return false;
		return review.fileDecisions.get(rel) === 'conflict';
	});

	function getLanguage(filename: string): string {
		const ext = filename.split('.').pop() ?? '';
		const map: Record<string, string> = {
			ts: 'typescript', js: 'javascript', svelte: 'svelte', rs: 'rust',
			json: 'json', css: 'css', html: 'html', md: 'markdown',
			toml: 'toml', py: 'python', go: 'go', yaml: 'yaml', yml: 'yaml',
			sh: 'shell', bash: 'shell', zsh: 'shell',
			rb: 'ruby', graphql: 'graphql', gql: 'graphql',
		};
		return map[ext] ?? 'plaintext';
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
			wordWrap: 'on',
			tabSize: 2,
			scrollbar: { horizontal: 'hidden' },
		});

		// Create diff editor eagerly so it's warm when the first diff appears (no creation latency)
		diffEditor = monaco.editor.createDiffEditor(diffContainer, {
			theme: 'tyck-theme',
			fontSize: 12,
			fontFamily: "'SF Mono', 'Fira Code', 'Cascadia Code', monospace",
			renderSideBySide: !diffInline,
			automaticLayout: true,
			scrollBeyondLastLine: false,
			padding: { top: 8 },
			minimap: { enabled: false },
			wordWrap: 'on',
			scrollbar: { horizontal: 'hidden' },
		});

		// Register undo/redo on the diff editor's modified pane.
		// We maintain our own snapshot stack for chunk accept/reject operations because
		// those edit the original model — Monaco's native undo only covers the modified model.
		{
			const modEd = diffEditor.getModifiedEditor();
			modEd.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyZ, () => chunkUndo());
			modEd.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyZ, () => chunkRedo());
			modEd.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyY, () => chunkRedo());
		}

		// Set up per-chunk hover listeners on the diff editor (once, reused across model swaps)
		{
			const TOOLBAR_H = 36; // approximate toolbar pixel height
			const modEd = diffEditor.getModifiedEditor();

			modEd.onMouseMove((e) => {
				const changes = diffEditor?.getLineChanges();
				if (!changes || !e.target.position) {
					scheduleHideChunk();
					return;
				}
				const line = e.target.position.lineNumber;
				const change = changes.find(c => {
					const end = c.modifiedEndLineNumber >= c.modifiedStartLineNumber
						? c.modifiedEndLineNumber
						: c.modifiedStartLineNumber;
					return line >= c.modifiedStartLineNumber && line <= end;
				}) ?? null;

				if (change) {
					cancelHideChunk();
					hoveredChange = change;
					chunkMenuTop = TOOLBAR_H + modEd.getTopForLineNumber(change.modifiedStartLineNumber) - modEd.getScrollTop();
				} else {
					scheduleHideChunk();
				}
			});

			modEd.onMouseLeave(() => scheduleHideChunk());
			modEd.onDidScrollChange(() => {
				if (hoveredChange) {
					chunkMenuTop = TOOLBAR_H + modEd.getTopForLineNumber(hoveredChange.modifiedStartLineNumber) - modEd.getScrollTop();
				}
			});
		}

		// Save on Ctrl+S / Cmd+S
		editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, async () => {
			const file = $activeFile;
			if (!file) return;
			const value = editor!.getValue();
			try {
				await invoke('write_file', { path: file.path, content: value });
				markFileSaved(file.path);
				const session = $activeSession;
				if (session) {
					// Exclude this file from the review tab — it's a developer edit, not an agent edit
					sessionReview.markDevSaved(session.id, file.path);
					// Track edits while agent is paused for context on resume
					if (session.status === 'paused') {
						trackDevEdit(session.id, file.path);
					}
				}
			} catch (e) {
				toast.error(`Failed to save ${file.name}: ${e}`);
			}
		});

		// Track content changes (skip programmatic setValue calls)
		editor.onDidChangeModelContent(() => {
			if (settingContent) return;
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

	// React to file changes — use per-file models with file:// URIs so LSP can
	// resolve imports, cross-file definitions, and workspace-wide references.
	$effect(() => {
		const file = $activeFile;
		if (!editor || !monaco) return;

		if (file) {
			const lang = getLanguage(file.name);
			const uri = monaco.Uri.file(file.path);

			// Get existing model for this file or create a new one with its URI
			let model = monaco.editor.getModel(uri);
			if (!model) {
				model = monaco.editor.createModel(file.content, lang, uri);
			} else {
				if (model.getLanguageId() !== lang) {
					monaco.editor.setModelLanguage(model, lang);
				}
				if (model.getValue() !== file.content) {
					settingContent = true;
					model.setValue(file.content);
					settingContent = false;
				}
			}

			// Swap model on the editor only when it actually changes
			// (avoids losing cursor position on unrelated store updates)
			if (editor.getModel() !== model) {
				editor.setModel(model);
			}

			// Start (or reuse) the language server for this file's language
			const root = $projectRoot;
			if (root) {
				lspClientManager.getOrStart(lang, root).catch(() => {});
			}
		} else {
			// No file open — show an empty plaintext model
			const emptyUri = monaco.Uri.parse('inmemory://tyck/empty');
			let model = monaco.editor.getModel(emptyUri);
			if (!model) {
				model = monaco.editor.createModel('', 'plaintext', emptyUri);
			}
			if (editor.getModel() !== model) {
				editor.setModel(model);
			}
		}
	});

	// Auto-trigger diff/merge editor when active file has agent changes
	$effect(() => {
		const review = $activeReview;
		const rel = activeFileRelPath;
		if (!monaco) return;

		if (review && rel && inReview) {
			const key = `${review.sessionId}:${rel}`;
			const fileIsConflict = review.fileDecisions.get(rel) === 'conflict';
			if (fileIsConflict) {
				// Always re-show merge editor on conflict (state may have changed)
				hideDiffEditor();
				showMergeEditor(review.sessionId, rel);
				currentDiffKey = key;
			} else if (key !== currentDiffKey) {
				// Only update models when file changes — not on every store update
				currentDiffKey = key;
				hideMergeEditor();
				showDiffEditor(review.sessionId, rel);
			}
		} else {
			if (currentDiffKey !== null) {
				currentDiffKey = null;
				hideDiffEditor();
				hideMergeEditor();
			}
		}
	});

	// ── Normal diff view (non-conflict) ──

	async function showDiffEditor(sessionId: string, filePath: string) {
		if (!monaco || !diffEditor) return;

		try {
			const [originalContent, modifiedContent] = await Promise.all([
				invoke<string>('get_file_at_base', { sessionId, filePath }).catch(() => ''),
				invoke<string>('get_file_from_worktree', { sessionId, filePath }).catch(() => ''),
			]);

			const lang = getLanguage(filePath.split('/').pop() ?? '');

			// Clear chunk undo/redo stacks — they belong to the previous file
			chunkUndoStack.length = 0;
			chunkRedoStack.length = 0;

			// Reuse the existing editor instance — just swap models (no dispose/recreate = zero latency)
			const oldModel = diffEditor.getModel();
			diffEditor.updateOptions({ renderSideBySide: !diffInline });
			diffEditor.setModel({
				original: monaco.editor.createModel(originalContent, lang),
				modified: monaco.editor.createModel(modifiedContent, lang),
			});
			// Dispose old models to avoid memory leaks
			oldModel?.original?.dispose();
			oldModel?.modified?.dispose();
		} catch (e) {
			console.warn('showDiffEditor failed:', e);
		}
	}

	function hideDiffEditor() {
		// Don't dispose — diffEditor is always-in-DOM and reused for instant rendering
		// Just clear the model so memory isn't held
		if (diffEditor) {
			const oldModel = diffEditor.getModel();
			if (oldModel?.original || oldModel?.modified) {
				diffEditor.setModel({
					original: monaco.editor.createModel('', 'plaintext'),
					modified: monaco.editor.createModel('', 'plaintext'),
				});
				oldModel?.original?.dispose();
				oldModel?.modified?.dispose();
			}
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

		const lang = getLanguage((activeFileRelPath ?? '').split('/').pop() ?? '');
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

		const lang = getLanguage((activeFileRelPath ?? '').split('/').pop() ?? '');
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
		const rel = activeFileRelPath;
		if (!mergeEditor || !rel || !sid) return;

		const content = mergeEditor.getValue();

		// Check for remaining conflict markers
		if (content.includes('<<<<<<<') || content.includes('>>>>>>>')) {
			if (!confirm('The file still contains conflict markers. Resolve anyway?')) return;
		}

		await sessionReview.resolveConflict(sid, rel, content);
		hideMergeEditor();
	}

	// ── Normal review actions ──

	function toggleDiffLayout() {
		diffInline = !diffInline;
		if (diffEditor) {
			diffEditor.updateOptions({ renderSideBySide: !diffInline });
		}
	}

	async function onAcceptReviewFile() {
		const rel = activeFileRelPath;
		const sid = $activeSessionId;
		const review = $activeReview;
		if (!rel || !sid || !review) return;

		// If the dev made edits in the diff editor, write them back to the WORKTREE
		// so those edits are part of the branch when Merge to Main / Push & PR is triggered.
		// Nothing is written to the main workspace here — that happens only on merge/push.
		const modifiedContent = diffEditor?.getModifiedEditor().getValue();
		if (modifiedContent !== undefined) {
			try {
				await invoke('write_file', { path: review.worktreePath + '/' + rel, content: modifiedContent });
			} catch (e) {
				toast.error(`Failed to save edits: ${e}`);
				return;
			}
		}

		// Remove from the review diff — file stays in the worktree branch
		sessionReview.removeFileFromReview(sid, rel);
	}

	function onRejectReviewFile() {
		const rel = activeFileRelPath;
		const sid = $activeSessionId;
		if (!rel || !sid) return;
		sessionReview.rejectFile(sid, rel);
	}

	function pushChunkUndo() {
		const origModel = diffEditor?.getOriginalEditor().getModel();
		const modModel = diffEditor?.getModifiedEditor().getModel();
		if (!origModel || !modModel) return;
		chunkUndoStack.push({ orig: origModel.getValue(), mod: modModel.getValue() });
		chunkRedoStack.length = 0;
	}

	function chunkUndo() {
		const modEd = diffEditor?.getModifiedEditor();
		const origModel = diffEditor?.getOriginalEditor().getModel();
		const modModel = modEd?.getModel();
		if (!origModel || !modModel || !modEd) return;
		if (chunkUndoStack.length === 0) {
			modEd.trigger('keyboard', 'undo', null);
			return;
		}
		const cursor = modEd.getPosition();
		chunkRedoStack.push({ orig: origModel.getValue(), mod: modModel.getValue() });
		const snap = chunkUndoStack.pop()!;
		origModel.setValue(snap.orig);
		modModel.setValue(snap.mod);
		if (cursor) modEd.setPosition(cursor);
		modEd.focus();
	}

	function chunkRedo() {
		const modEd = diffEditor?.getModifiedEditor();
		const origModel = diffEditor?.getOriginalEditor().getModel();
		const modModel = modEd?.getModel();
		if (!origModel || !modModel || !modEd) return;
		if (chunkRedoStack.length === 0) {
			modEd.trigger('keyboard', 'redo', null);
			return;
		}
		const cursor = modEd.getPosition();
		chunkUndoStack.push({ orig: origModel.getValue(), mod: modModel.getValue() });
		const snap = chunkRedoStack.pop()!;
		origModel.setValue(snap.orig);
		modModel.setValue(snap.mod);
		if (cursor) modEd.setPosition(cursor);
		modEd.focus();
	}

	function scheduleHideChunk() {
		if (hideChunkTimer) return;
		hideChunkTimer = setTimeout(() => {
			hoveredChange = null;
			chunkMenuTop = null;
			hideChunkTimer = null;
		}, 200);
	}

	function cancelHideChunk() {
		if (hideChunkTimer) {
			clearTimeout(hideChunkTimer);
			hideChunkTimer = null;
		}
	}

	function acceptChunk(change: Monaco.editor.ILineChange) {
		pushChunkUndo();
		// Apply this chunk to the original model so the diff for it disappears.
		// The modified content is unchanged — "Accept All" will still write it to disk.
		const modEd = diffEditor?.getModifiedEditor();
		const origEd = diffEditor?.getOriginalEditor();
		const modModel = modEd?.getModel();
		const origModel = origEd?.getModel();
		if (!modModel || !origModel || !monaco) return;

		const isPureInsertion = change.originalEndLineNumber < change.originalStartLineNumber;
		const isPureDeletion = change.modifiedEndLineNumber < change.modifiedStartLineNumber;

		if (isPureDeletion) {
			// Agent deleted lines — remove them from original too
			origModel.pushEditOperations([], [{
				range: {
					startLineNumber: change.originalStartLineNumber,
					startColumn: 1,
					endLineNumber: change.originalEndLineNumber,
					endColumn: origModel.getLineMaxColumn(change.originalEndLineNumber),
				},
				text: '',
			}], () => null);
		} else if (isPureInsertion) {
			// Agent added lines — insert them into original after originalStartLineNumber
			const modifiedText = modModel.getValueInRange({
				startLineNumber: change.modifiedStartLineNumber,
				startColumn: 1,
				endLineNumber: change.modifiedEndLineNumber,
				endColumn: modModel.getLineMaxColumn(change.modifiedEndLineNumber),
			});
			origModel.pushEditOperations([], [{
				range: {
					startLineNumber: change.originalStartLineNumber,
					startColumn: origModel.getLineMaxColumn(change.originalStartLineNumber),
					endLineNumber: change.originalStartLineNumber,
					endColumn: origModel.getLineMaxColumn(change.originalStartLineNumber),
				},
				text: '\n' + modifiedText,
			}], () => null);
		} else {
			// Agent modified lines — update original to match modified for this range
			const modifiedText = modModel.getValueInRange({
				startLineNumber: change.modifiedStartLineNumber,
				startColumn: 1,
				endLineNumber: change.modifiedEndLineNumber,
				endColumn: modModel.getLineMaxColumn(change.modifiedEndLineNumber),
			});
			origModel.pushEditOperations([], [{
				range: {
					startLineNumber: change.originalStartLineNumber,
					startColumn: 1,
					endLineNumber: change.originalEndLineNumber,
					endColumn: origModel.getLineMaxColumn(change.originalEndLineNumber),
				},
				text: modifiedText,
			}], () => null);
		}

		hoveredChange = null;
		chunkMenuTop = null;
		diffEditor?.getModifiedEditor().focus();
	}

	function rejectChunk(change: Monaco.editor.ILineChange) {
		pushChunkUndo();
		const modEd = diffEditor?.getModifiedEditor();
		const origEd = diffEditor?.getOriginalEditor();
		const modModel = modEd?.getModel();
		const origModel = origEd?.getModel();
		if (!modModel || !origModel || !monaco) return;

		const isPureDeletion = change.modifiedEndLineNumber < change.modifiedStartLineNumber;
		const isPureInsertion = change.originalEndLineNumber < change.originalStartLineNumber;

		let originalText = '';
		if (!isPureInsertion) {
			originalText = origModel.getValueInRange({
				startLineNumber: change.originalStartLineNumber,
				startColumn: 1,
				endLineNumber: change.originalEndLineNumber,
				endColumn: origModel.getLineMaxColumn(change.originalEndLineNumber),
			});
		}

		if (isPureDeletion) {
			// Lines were deleted by agent — restore them after modifiedStartLineNumber
			modModel.pushEditOperations([], [{
				range: {
					startLineNumber: change.modifiedStartLineNumber,
					startColumn: modModel.getLineMaxColumn(change.modifiedStartLineNumber),
					endLineNumber: change.modifiedStartLineNumber,
					endColumn: modModel.getLineMaxColumn(change.modifiedStartLineNumber),
				},
				text: '\n' + originalText,
			}], () => null);
		} else {
			modModel.pushEditOperations([], [{
				range: {
					startLineNumber: change.modifiedStartLineNumber,
					startColumn: 1,
					endLineNumber: change.modifiedEndLineNumber,
					endColumn: modModel.getLineMaxColumn(change.modifiedEndLineNumber),
				},
				text: originalText,
			}], () => null);
		}

		hoveredChange = null;
		chunkMenuTop = null;
		diffEditor?.getModifiedEditor().focus();
	}

	onDestroy(() => {
		if (hideChunkTimer) clearTimeout(hideChunkTimer);
		lspClientManager.stopAll().catch(() => {});
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
				<span class="diff-file-name conflict-label">CONFLICT: {activeFileRelPath}</span>
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
	{/if}

	<!-- Diff editor always in DOM (hidden when not in review) so Monaco stays alive and loads instantly -->
	<div class="review-diff-wrapper" class:hidden={!inReview || isConflict}>
		<div class="diff-toolbar">
			<span class="diff-file-name">{activeFileRelPath}</span>
			<div class="diff-toolbar-actions">
				<button class="diff-tool-btn" onclick={toggleDiffLayout}>
					{diffInline ? 'Side-by-Side' : 'Inline'}
				</button>
				<button class="diff-tool-btn accept" onclick={onAcceptReviewFile}>Accept All</button>
				<button class="diff-tool-btn reject" onclick={onRejectReviewFile}>Reject All</button>
			</div>
		</div>
		<div class="diff-editor-container" bind:this={diffContainer}></div>
		{#if hoveredChange !== null && chunkMenuTop !== null}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="chunk-action-bar"
				style="top: {chunkMenuTop}px"
				onmouseenter={cancelHideChunk}
				onmouseleave={scheduleHideChunk}
			>
				<button class="chunk-btn accept" onclick={() => acceptChunk(hoveredChange!)}>✓ Keep</button>
				<button class="chunk-btn reject" onclick={() => rejectChunk(hoveredChange!)}>✕ Revert</button>
			</div>
		{/if}
	</div>

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
		position: relative;
	}
	.review-diff-wrapper.hidden {
		display: none;
	}
	.chunk-action-bar {
		position: absolute;
		right: 16px;
		z-index: 20;
		display: flex;
		gap: 3px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 5px;
		padding: 2px 3px;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
		pointer-events: auto;
	}
	.chunk-btn {
		padding: 2px 8px;
		border: none;
		border-radius: 3px;
		font-size: 10px;
		font-weight: 700;
		cursor: pointer;
		transition: background 0.1s;
	}
	.chunk-btn.accept {
		background: color-mix(in srgb, var(--color-success) 15%, transparent);
		color: var(--color-success);
	}
	.chunk-btn.accept:hover {
		background: color-mix(in srgb, var(--color-success) 25%, transparent);
	}
	.chunk-btn.reject {
		background: color-mix(in srgb, var(--color-error) 15%, transparent);
		color: var(--color-error);
	}
	.chunk-btn.reject:hover {
		background: color-mix(in srgb, var(--color-error) 25%, transparent);
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
