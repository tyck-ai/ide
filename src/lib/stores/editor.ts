import { writable, derived } from 'svelte/store';
import { isAgentMode } from './settings';

export interface OpenFile {
	path: string;
	name: string;
	content: string;
	modified: boolean;
}

export const openFiles = writable<OpenFile[]>([]);
export const activeFilePath = writable<string | null>(null);
export const projectRoot = writable<string | null>(null);
export const selection = writable<string>('');
export const cursorLine = writable<number>(1);

/** Set by agentTerminal when the active session changes. Used to root the file tree. */
export const activeWorktreePath = writable<string | null>(null);

/** When true, ContextZone shows main workspace (read-only) in agent mode */
export const peekingMain = writable<boolean>(false);

/** The directory the file tree should show — worktree in agent mode, project root in dev mode */
export const activeWorkingDirectory = derived(
	[isAgentMode, activeWorktreePath, projectRoot, peekingMain],
	([$isAgent, $wt, $root, $peeking]) => {
		if ($peeking) return $root;
		if ($isAgent && $wt) return $wt;
		return $root;
	}
);

export const activeFile = derived(
	[openFiles, activeFilePath],
	([$openFiles, $activeFilePath]) => {
		return $openFiles.find(f => f.path === $activeFilePath) ?? null;
	}
);

export function openFileInEditor(path: string, name: string, content: string) {
	openFiles.update(files => {
		const existing = files.find(f => f.path === path);
		if (!existing) {
			files.push({ path, name, content, modified: false });
		}
		return files;
	});
	activeFilePath.set(path);
}

export function updateFileContent(path: string, content: string) {
	openFiles.update(files =>
		files.map(f => f.path === path ? { ...f, content, modified: true } : f)
	);
}

export function markFileSaved(path: string) {
	openFiles.update(files =>
		files.map(f => f.path === path ? { ...f, modified: false } : f)
	);
}

export function closeFile(path: string) {
	// Read current files, compute next active, then update both atomically
	let files: OpenFile[] = [];
	openFiles.subscribe(f => files = f)();
	const remaining = files.filter(f => f.path !== path);
	openFiles.set(remaining);
	activeFilePath.update(current => {
		if (current === path) {
			return remaining.length > 0 ? remaining[remaining.length - 1].path : null;
		}
		return current;
	});
}

/** Files visible in the current context (filtered by worktree in agent mode) */
export const visibleFiles = derived(
	[openFiles, isAgentMode, activeWorktreePath],
	([$files, $isAgent, $wt]) => {
		if ($isAgent && $wt) {
			return $files.filter(f => f.path.startsWith($wt));
		}
		return $files;
	}
);

/** Close all files and reset workspace state. */
export function resetWorkspace() {
	openFiles.set([]);
	activeFilePath.set(null);
}
