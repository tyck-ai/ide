import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeFilePath } from './editor';
import { startGitPoller } from './git';

// The git root currently tracked by the git panel.
// - Auto-follows the active file (walk up to nearest .git)
// - Can be pinned/overridden manually via the picker in GitView
export const activeGitRoot = writable<string | null>(null);
export const gitRootPinned = writable(false);

let lastFilePath: string | null = null;

/** Called when the workspace changes (new folder opened). Resets pin and sets initial root. */
export function resetGitRootForWorkspace(cwd: string) {
	gitRootPinned.set(false);
	lastFilePath = null;
	setActiveGitRoot(cwd);
}

/** Called when the active file changes. Ignored if pinned. */
export async function followActiveFile(filePath: string | null): Promise<void> {
	if (get(gitRootPinned)) return;
	if (filePath === lastFilePath) return;
	lastFilePath = filePath;

	if (!filePath) return; // keep current root when no file is open

	try {
		const gitRoot = await invoke<string | null>('find_git_root_for_file', { filePath });
		if (gitRoot) setActiveGitRoot(gitRoot);
	} catch {
		// non-critical — keep current root
	}
}

/** Manually pin a git root from the picker. */
export function pinGitRoot(path: string) {
	gitRootPinned.set(true);
	setActiveGitRoot(path);
}

/** Unpin and revert to following the active file. */
export function unpinGitRoot() {
	gitRootPinned.set(false);
	lastFilePath = null;
	followActiveFile(get(activeFilePath));
}

function setActiveGitRoot(path: string | null) {
	const current = get(activeGitRoot);
	if (!path || path === current) return;
	activeGitRoot.set(path);
	startGitPoller(path);
}
