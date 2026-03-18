import { get } from 'svelte/store';
import { updateSettings, settings } from './settings';
import { openFiles, activeFilePath, type OpenFile } from './editor';
import { saveCurrentSessionState, restoreSessionState } from './sessionContext';
import { activeSessionId } from './activeSession';

interface DevModeState {
	openFiles: OpenFile[];
	activeFilePath: string | null;
}

let devModeState: DevModeState = { openFiles: [], activeFilePath: null };

function saveDevModeState() {
	devModeState = {
		openFiles: get(openFiles),
		activeFilePath: get(activeFilePath),
	};
}

function restoreDevModeState() {
	openFiles.set(devModeState.openFiles);
	activeFilePath.set(devModeState.activeFilePath);
}

/**
 * Switch workspace mode, saving and restoring per-mode editor state.
 * Use this instead of calling updateSettings({ workspaceMode }) directly.
 */
export async function switchToMode(mode: 'dev' | 'agent'): Promise<void> {
	const current = get(settings).workspaceMode;
	if (current === mode) return;

	if (mode === 'dev') {
		// Leaving agent mode: save the active session's state, restore dev state
		saveCurrentSessionState();
		restoreDevModeState();
	} else {
		// Leaving dev mode: save dev state, restore the active agent session's state
		saveDevModeState();
		const sessionId = get(activeSessionId);
		if (sessionId) {
			restoreSessionState(sessionId);
		} else {
			// No active agent session yet — clear editor for a clean slate
			openFiles.set([]);
			activeFilePath.set(null);
		}
	}

	await updateSettings({ workspaceMode: mode });
}
