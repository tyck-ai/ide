import { derived } from 'svelte/store';
import { createGitStore } from './git';
import { activeSession } from './agentTerminal';
import { isAgentMode } from './settings';

/**
 * Git store that auto-tracks the active agent session's worktree.
 * Used by the git status bar in agent mode.
 */
export const agentGit = createGitStore();

// Auto-start / auto-switch when the active agent session changes
let trackedSessionId: string | null = null;

const autoTrack = derived([activeSession, isAgentMode], ([$session, $agentMode]) =>
	$agentMode && $session ? $session : null
);

autoTrack.subscribe(session => {
	if (session?.id === trackedSessionId) return;
	trackedSessionId = session?.id ?? null;
	if (session?.worktreePath) {
		agentGit.startWatching(session.worktreePath);
	} else {
		agentGit.stopWatching();
	}
});
