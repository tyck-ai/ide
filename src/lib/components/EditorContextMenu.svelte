<script lang="ts">
	import { contextMenuStore } from '$lib/stores/lsp';
	import { lspClientManager } from '$lib/lsp/LspClientManager';
	import type * as Monaco from 'monaco-editor';

	interface Props {
		editor: Monaco.editor.IStandaloneCodeEditor | undefined;
	}

	let { editor }: Props = $props();

	const state = $derived($contextMenuStore);

	function close() {
		contextMenuStore.update((s) => ({ ...s, visible: false }));
	}

	function getCapabilities() {
		if (!state.language || !state.visible) return null;
		const client = lspClientManager.getActiveClient(state.language);
		if (!client) return null;
		// initializeResult is a public property on BaseLanguageClient
		return (client as any).initializeResult?.capabilities ?? null;
	}

	const caps = $derived(getCapabilities());

	function runAction(actionId: string) {
		editor?.getAction(actionId)?.run();
		close();
	}

	function goToDefinition() {
		runAction('editor.action.revealDefinition');
	}

	function peekDefinition() {
		runAction('editor.action.peekDefinition');
	}

	function goToTypeDefinition() {
		runAction('editor.action.goToTypeDefinition');
	}

	function findReferences() {
		runAction('editor.action.referenceSearch.trigger');
	}

	function renameSymbol() {
		runAction('editor.action.rename');
	}

	function quickFix() {
		runAction('editor.action.quickFix');
	}

	function formatDocument() {
		runAction('editor.action.formatDocument');
	}

	function copyPath() {
		const model = editor?.getModel();
		if (model) {
			const fsPath = model.uri.fsPath || model.uri.path;
			navigator.clipboard.writeText(fsPath).catch(() => {});
		}
		close();
	}

	function copy() {
		runAction('editor.action.clipboardCopyAction');
	}

	function cut() {
		runAction('editor.action.clipboardCutAction');
	}

	const hasGoTo = $derived(!!(caps?.definitionProvider || caps?.typeDefinitionProvider));
	const hasRefOrRename = $derived(!!(caps?.referencesProvider || caps?.renameProvider));
	const hasCodeActions = $derived(!!(caps?.codeActionProvider || caps?.documentFormattingProvider));
</script>

{#if state.visible}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="ctx-backdrop"
		onclick={close}
		oncontextmenu={(e) => {
			e.preventDefault();
			close();
		}}
	></div>
	<div class="ctx-menu" style="left: {state.x}px; top: {state.y}px">
		<!-- Group 1: Go To (LSP-gated) -->
		{#if hasGoTo}
			<div class="ctx-group">
				{#if caps?.definitionProvider}
					<button class="ctx-item" onclick={goToDefinition}>
						<span class="ctx-label">Go to Definition</span>
						<span class="ctx-kbd">F12</span>
					</button>
					<button class="ctx-item" onclick={peekDefinition}>
						<span class="ctx-label">Peek Definition</span>
						<span class="ctx-kbd">⌥F12</span>
					</button>
				{/if}
				{#if caps?.typeDefinitionProvider}
					<button class="ctx-item" onclick={goToTypeDefinition}>
						<span class="ctx-label">Go to Type Definition</span>
					</button>
				{/if}
			</div>
			<div class="ctx-sep"></div>
		{/if}

		<!-- Group 2: References & Rename (LSP-gated) -->
		{#if hasRefOrRename}
			<div class="ctx-group">
				{#if caps?.referencesProvider}
					<button class="ctx-item" onclick={findReferences}>
						<span class="ctx-label">Find All References</span>
						<span class="ctx-kbd">⇧F12</span>
					</button>
				{/if}
				{#if caps?.renameProvider}
					<button class="ctx-item" onclick={renameSymbol}>
						<span class="ctx-label">Rename Symbol</span>
						<span class="ctx-kbd">F2</span>
					</button>
				{/if}
			</div>
			<div class="ctx-sep"></div>
		{/if}

		<!-- Group 3: Code Actions & Formatting (LSP-gated) -->
		{#if hasCodeActions}
			<div class="ctx-group">
				{#if caps?.codeActionProvider}
					<button class="ctx-item" onclick={quickFix}>
						<span class="ctx-label">Quick Fix...</span>
						<span class="ctx-kbd">⌘.</span>
					</button>
				{/if}
				{#if caps?.documentFormattingProvider}
					<button class="ctx-item" onclick={formatDocument}>
						<span class="ctx-label">Format Document</span>
						<span class="ctx-kbd">⇧⌥F</span>
					</button>
				{/if}
			</div>
			<div class="ctx-sep"></div>
		{/if}

		<!-- Group 4: Edit actions (always) -->
		<div class="ctx-group">
			<button class="ctx-item" onclick={copy}>
				<span class="ctx-label">Copy</span>
				<span class="ctx-kbd">⌘C</span>
			</button>
			<button class="ctx-item" onclick={cut}>
				<span class="ctx-label">Cut</span>
				<span class="ctx-kbd">⌘X</span>
			</button>
		</div>
		<div class="ctx-sep"></div>

		<!-- Group 5: File -->
		<div class="ctx-group">
			<button class="ctx-item" onclick={copyPath}>
				<span class="ctx-label">Copy File Path</span>
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
		min-width: 220px;
		max-width: 280px;
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
