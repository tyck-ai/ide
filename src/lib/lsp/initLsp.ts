import { initialize } from '@codingame/monaco-vscode-api/services';
import getEditorServiceOverride, { type OpenEditor } from '@codingame/monaco-vscode-editor-service-override';
import getConfigurationServiceOverride from '@codingame/monaco-vscode-configuration-service-override';

// ─── Types ────────────────────────────────────────────────────────────────────

/** Callback invoked when LSP navigation (go to definition, etc.) opens a new file. */
export type OpenFileCallback = (filePath: string) => Promise<void>;

// ─── Initialisation ───────────────────────────────────────────────────────────

let initialized = false;
let openFileCallback: OpenFileCallback | null = null;

/**
 * Set the callback that handles cross-file navigation triggered by LSP
 * (go to definition, go to type definition, etc.).
 *
 * Must be called before initLspServices() or the editor won't open files.
 */
export function setOpenFileCallback(cb: OpenFileCallback) {
	openFileCallback = cb;
}

/**
 * Initialise @codingame/monaco-vscode-api services.
 *
 * MUST be called once, before any Monaco editor is created.
 * Safe to call multiple times — subsequent calls are no-ops.
 */
export async function initLspServices(): Promise<void> {
	if (initialized) return;
	initialized = true;

	const openEditor: OpenEditor = async (modelRef, _options, _sideBySide) => {
		const resource = modelRef.object.textEditorModel?.uri;
		if (!resource) return undefined;

		// Hand cross-file navigation back to FocusZone via the registered callback
		if (openFileCallback) {
			await openFileCallback(resource.fsPath);
		}

		// Return undefined to let Monaco handle rendering the model it already has
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
