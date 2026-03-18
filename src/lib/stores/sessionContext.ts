import { writable, get } from 'svelte/store';
import { openFiles, activeFilePath, type OpenFile } from './editor';

export type ContextTab = 'editor' | 'review';

/**
 * Per-session UI state that gets saved when switching away from a session
 * and restored when switching back.
 */
interface SessionState {
	openFiles: OpenFile[];
	activeFilePath: string | null;
	contextTab: ContextTab;
}

const sessionStates = new Map<string, SessionState>();
let currentSessionId: string | null = null;

/** The active context tab (Review / Editor) in agent mode — per session. */
export const contextTab = writable<ContextTab>('review');

/** Save the current session's UI state before switching away */
export function saveCurrentSessionState() {
	if (!currentSessionId) return;
	sessionStates.set(currentSessionId, {
		openFiles: get(openFiles),
		activeFilePath: get(activeFilePath),
		contextTab: get(contextTab),
	});
}

/** Restore a session's UI state when switching to it */
export function restoreSessionState(sessionId: string) {
	const state = sessionStates.get(sessionId);
	if (state) {
		openFiles.set(state.openFiles);
		activeFilePath.set(state.activeFilePath);
		contextTab.set(state.contextTab);
	} else {
		// New session — start fresh with Review tab open
		openFiles.set([]);
		activeFilePath.set(null);
		contextTab.set('review');
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
