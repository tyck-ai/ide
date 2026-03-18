// ─── Types ────────────────────────────────────────────────────────────────────

/** Callback invoked when LSP navigation (go to definition, etc.) opens a new file. */
export type OpenFileCallback = (filePath: string) => Promise<void>;

// ─── Initialisation ───────────────────────────────────────────────────────────

let initPromise: Promise<void> | null = null;
let openFileCallback: OpenFileCallback | null = null;

/**
 * Set the callback that handles cross-file navigation triggered by LSP
 * (go to definition, go to type definition, etc.).
 */
export function setOpenFileCallback(cb: OpenFileCallback) {
	openFileCallback = cb;
}

/**
 * Initialise @codingame/monaco-vscode-api services.
 *
 * MUST be called once, before any Monaco editor is created.
 * Safe to call multiple times — subsequent calls are no-ops.
 *
 * All @codingame imports are dynamic so they don't land in the static import
 * graph of Svelte components — that caused a TDZ cycle at module initialisation.
 */
export function initLspServices(): Promise<void> {
	if (!initPromise) initPromise = _doInit();
	return initPromise;
}

async function _doInit(): Promise<void> {

	// Dynamic imports keep @codingame out of the static module graph, preventing
	// the "Cannot access 'component' before initialization" TDZ error in Svelte 5.
	//
	// vscode/localExtensionHost MUST be imported before initialize() is called —
	// it registers the extension host factory that initialize() waits for.
	// All four are imported concurrently so they're all registered by the time
	// we call initialize().
	const [, { initialize }, { default: getEditorServiceOverride }, { default: getConfigurationServiceOverride }] =
		await Promise.all([
			import('vscode/localExtensionHost'),
			import('@codingame/monaco-vscode-api/services'),
			import('@codingame/monaco-vscode-editor-service-override'),
			import('@codingame/monaco-vscode-configuration-service-override'),
		]);

	const openEditor = async (modelRef: any, _options: any, _sideBySide: any) => {
		const resource = modelRef.object.textEditorModel?.uri;
		if (!resource) return undefined;
		if (openFileCallback) {
			await openFileCallback(resource.fsPath);
		}
		return undefined;
	};

	await initialize(
		{
			...getEditorServiceOverride(openEditor),
			...getConfigurationServiceOverride(),
		},
		document.body,
	);
}
