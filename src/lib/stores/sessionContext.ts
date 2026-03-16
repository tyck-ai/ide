import { get } from 'svelte/store';
import { openFiles, activeFilePath, type OpenFile } from './editor';

/**
 * Per-session UI state that gets saved when switching away from a session
 * and restored when switching back.
 */
interface SessionState {
	openFiles: OpenFile[];
	activeFilePath: string | null;
}

const sessionStates = new Map<string, SessionState>();
let currentSessionId: string | null = null;

/** Save the current session's UI state before switching away */
export function saveCurrentSessionState() {
	if (!currentSessionId) return;
	sessionStates.set(currentSessionId, {
		openFiles: get(openFiles),
		activeFilePath: get(activeFilePath),
	});
}

/** Restore a session's UI state when switching to it */
export function restoreSessionState(sessionId: string) {
	const state = sessionStates.get(sessionId);
	if (state) {
		openFiles.set(state.openFiles);
		activeFilePath.set(state.activeFilePath);
	} else {
		// New session — start with no files open
		// Don't clear globally — filter will handle visibility
	}
	currentSessionId = sessionId;
}

/** Clean up state for a closed session */
export function clearSessionState(sessionId: string) {
	sessionStates.delete(sessionId);
	if (currentSessionId === sessionId) {
		currentSessionId = null;
	}
}

/** Switch sessions: save old, restore new */
export function switchSessionContext(newSessionId: string) {
	saveCurrentSessionState();
	restoreSessionState(newSessionId);
}
