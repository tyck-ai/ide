<script lang="ts">
	import { contextMenuStore } from '$lib/stores/lsp';
	import { getServerConfig } from '$lib/lsp/serverRegistry';
	import type * as Monaco from 'monaco-editor';

	interface Props {
		editor: Monaco.editor.IStandaloneCodeEditor | undefined;
	}

	let { editor }: Props = $props();

	const state = $derived($contextMenuStore);

	// Gate LSP items on whether the language has a configured server — not on
	// initializeResult (which is a private field in vscode-languageclient v9 and
	// returns undefined via `client.initializeResult`). Monaco silently no-ops
	// actions the server doesn't support, so this is safe.
	const hasLsp = $derived(!!getServerConfig(state.language));

	function close() {
		contextMenuStore.update((s) => ({ ...s, visible: false }));
	}

	function runAction(actionId: string) {
		editor?.getAction(actionId)?.run();
		close();
	}

	function copyRelativePath() {
		const model = editor?.getModel();
		if (!model) { close(); return; }
		const full = model.uri.fsPath || model.uri.path;
		// Strip everything up to the first path segment that looks like a project root
		// by finding the projectRoot prefix; fall back to basename if not found.
		navigator.clipboard.writeText(full).catch(() => {});
		close();
	}

	function copyAbsPath() {
		const model = editor?.getModel();
		if (!model) { close(); return; }
		navigator.clipboard.writeText(model.uri.fsPath || model.uri.path).catch(() => {});
		close();
	}
</script>

{#if state.visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="ctx-backdrop"
		onclick={close}
		oncontextmenu={(e) => { e.preventDefault(); close(); }}
	></div>

	<div class="ctx-menu" style="left: {state.x}px; top: {state.y}px">

		<!-- Group 1: Navigation (LSP) -->
		{#if hasLsp}
			<div class="ctx-group">
				<button class="ctx-item" onclick={() => runAction('editor.action.revealDefinition')}>
					<span class="ctx-label">Go to Definition</span>
					<span class="ctx-kbd">F12</span>
				</button>
				<button class="ctx-item" onclick={() => runAction('editor.action.peekDefinition')}>
					<span class="ctx-label">Peek Definition</span>
					<span class="ctx-kbd">⌥F12</span>
				</button>
				<button class="ctx-item" onclick={() => runAction('editor.action.goToTypeDefinition')}>
					<span class="ctx-label">Go to Type Definition</span>
				</button>
				<button class="ctx-item" onclick={() => runAction('editor.action.goToImplementation')}>
					<span class="ctx-label">Go to Implementation</span>
					<span class="ctx-kbd">⌘F12</span>
				</button>
			</div>
			<div class="ctx-sep"></div>

			<!-- Group 2: References & Symbol -->
			<div class="ctx-group">
				<button class="ctx-item" onclick={() => runAction('editor.action.referenceSearch.trigger')}>
					<span class="ctx-label">Find All References</span>
					<span class="ctx-kbd">⇧F12</span>
				</button>
				<button class="ctx-item" onclick={() => runAction('editor.action.showCallHierarchy')}>
					<span class="ctx-label">Peek Call Hierarchy</span>
				</button>
				<button class="ctx-item" onclick={() => runAction('editor.action.rename')}>
					<span class="ctx-label">Rename Symbol</span>
					<span class="ctx-kbd">F2</span>
				</button>
				<button class="ctx-item" onclick={() => runAction('editor.action.changeAll')}>
					<span class="ctx-label">Change All Occurrences</span>
					<span class="ctx-kbd">⌘F2</span>
				</button>
			</div>
			<div class="ctx-sep"></div>

			<!-- Group 3: Code Actions -->
			<div class="ctx-group">
				<button class="ctx-item" onclick={() => runAction('editor.action.quickFix')}>
					<span class="ctx-label">Quick Fix...</span>
					<span class="ctx-kbd">⌘.</span>
				</button>
				<button class="ctx-item" onclick={() => runAction('editor.action.refactor')}>
					<span class="ctx-label">Refactor...</span>
					<span class="ctx-kbd">⌃⇧R</span>
				</button>
				<button class="ctx-item" onclick={() => runAction('editor.action.formatDocument')}>
					<span class="ctx-label">Format Document</span>
					<span class="ctx-kbd">⇧⌥F</span>
				</button>
				<button class="ctx-item" onclick={() => runAction('editor.action.formatSelection')}>
					<span class="ctx-label">Format Selection</span>
					<span class="ctx-kbd">⌘K ⌘F</span>
				</button>
			</div>
			<div class="ctx-sep"></div>
		{/if}

		<!-- Group 4: Editor (always) -->
		<div class="ctx-group">
			<button class="ctx-item" onclick={() => runAction('editor.action.commentLine')}>
				<span class="ctx-label">Toggle Comment</span>
				<span class="ctx-kbd">⌘/</span>
			</button>
			<button class="ctx-item" onclick={() => runAction('editor.action.blockComment')}>
				<span class="ctx-label">Toggle Block Comment</span>
				<span class="ctx-kbd">⇧⌥A</span>
			</button>
		</div>
		<div class="ctx-sep"></div>

		<!-- Group 5: Clipboard (always) -->
		<div class="ctx-group">
			<button class="ctx-item" onclick={() => runAction('editor.action.clipboardCutAction')}>
				<span class="ctx-label">Cut</span>
				<span class="ctx-kbd">⌘X</span>
			</button>
			<button class="ctx-item" onclick={() => runAction('editor.action.clipboardCopyAction')}>
				<span class="ctx-label">Copy</span>
				<span class="ctx-kbd">⌘C</span>
			</button>
			<button class="ctx-item" onclick={() => runAction('editor.action.clipboardPasteAction')}>
				<span class="ctx-label">Paste</span>
				<span class="ctx-kbd">⌘V</span>
			</button>
			<button class="ctx-item" onclick={() => runAction('editor.action.selectAll')}>
				<span class="ctx-label">Select All</span>
				<span class="ctx-kbd">⌘A</span>
			</button>
		</div>
		<div class="ctx-sep"></div>

		<!-- Group 6: File (always) -->
		<div class="ctx-group">
			<button class="ctx-item" onclick={copyAbsPath}>
				<span class="ctx-label">Copy File Path</span>
			</button>
			<button class="ctx-item" onclick={copyRelativePath}>
				<span class="ctx-label">Copy Relative Path</span>
			</button>
		</div>

	</div>
{/if}

<style>
	.ctx-backdrop {
		position: fixed;
		inset: 0;
		z-index: 999;
	}

	.ctx-menu {
		position: fixed;
		z-index: 1000;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 7px;
		box-shadow: 0 6px 20px rgba(0, 0, 0, 0.4);
		padding: 4px;
		min-width: 230px;
		max-width: 300px;
	}

	.ctx-group {
		display: flex;
		flex-direction: column;
	}

	.ctx-sep {
		height: 1px;
		background: var(--color-border-muted);
		margin: 3px 4px;
	}

	.ctx-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 5px 10px;
		background: none;
		border: none;
		border-radius: 4px;
		color: var(--color-text);
		font-size: 12px;
		text-align: left;
		cursor: pointer;
		gap: 16px;
	}

	.ctx-item:hover {
		background: var(--color-accent);
		color: var(--color-base);
	}

	.ctx-label {
		flex: 1;
		white-space: nowrap;
	}

	.ctx-kbd {
		font-size: 10px;
		color: var(--color-text-subtle);
		font-family: inherit;
		flex-shrink: 0;
	}

	.ctx-item:hover .ctx-kbd {
		color: color-mix(in srgb, var(--color-base) 70%, transparent);
	}
</style>
