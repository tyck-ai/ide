import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { projectRoot } from './editor';

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

export interface FileStatus {
	path: string;
	status: string; // M, A, D, R, C
}

export interface GitFullStatus {
	isRepo: boolean;
	branch: string;
	ahead: number;
	behind: number;
	staged: FileStatus[];
	unstaged: FileStatus[];
	untracked: string[];
	conflicts: string[];
}

export interface GitBranch {
	name: string;
	isCurrent: boolean;
	isRemote: boolean;
	ahead: number;
	behind: number;
	lastCommit: string;
}

export interface GitCommit {
	sha: string;
	shortSha: string;
	message: string;
	author: string;
	authorEmail: string;
	date: string;
	relativeDate: string;
	parents: string[];
}

export interface GitStash {
	index: number;
	message: string;
	branch: string;
	date: string;
}

export interface CommitFiles {
	added: string[];
	modified: string[];
	deleted: string[];
}

export interface CommitDetail {
	commit: GitCommit;
	files: CommitFiles;
	diff: string;
}

export interface BlameLine {
	sha: string;
	author: string;
	date: string;
	lineNumber: number;
	content: string;
}

interface GitState {
	isRepo: boolean;
	branch: string;
	ahead: number;
	behind: number;
	staged: FileStatus[];
	unstaged: FileStatus[];
	untracked: string[];
	conflicts: string[];
	branches: GitBranch[];
	commits: GitCommit[];
	stashes: GitStash[];
	loading: boolean;
	error: string | null;
}

// ─────────────────────────────────────────────────────────────────────────────
// Store Creation
// ─────────────────────────────────────────────────────────────────────────────

const emptyState: GitState = {
	isRepo: false,
	branch: '',
	ahead: 0,
	behind: 0,
	staged: [],
	unstaged: [],
	untracked: [],
	conflicts: [],
	branches: [],
	commits: [],
	stashes: [],
	loading: false,
	error: null,
};

function createGitStore() {
	const { subscribe, set, update } = writable<GitState>({ ...emptyState });

	let pollInterval: ReturnType<typeof setInterval> | null = null;
	let currentPath: string | null = null;

	function getPath(): string | null {
		return currentPath;
	}

	async function refresh() {
		const path = getPath();
		if (!path) return;

		update(s => ({ ...s, loading: true, error: null }));

		try {
			const result = await invoke<{
				is_repo: boolean;
				branch: string;
				ahead: number;
				behind: number;
				staged: { path: string; status: string }[];
				unstaged: { path: string; status: string }[];
				untracked: string[];
				conflicts: string[];
			}>('git_full_status', { path });

			update(s => ({
				...s,
				isRepo: result.is_repo,
				branch: result.branch,
				ahead: result.ahead,
				behind: result.behind,
				staged: result.staged.map(f => ({ path: f.path, status: f.status })),
				unstaged: result.unstaged.map(f => ({ path: f.path, status: f.status })),
				untracked: result.untracked,
				conflicts: result.conflicts,
				loading: false,
			}));
		} catch (e) {
			update(s => ({ ...s, loading: false, error: String(e) }));
		}
	}

	async function refreshBranches() {
		const path = getPath();
		if (!path) return;

		try {
			const result = await invoke<{
				name: string;
				is_current: boolean;
				is_remote: boolean;
				ahead: number;
				behind: number;
				last_commit: string;
			}[]>('git_branches', { path });

			update(s => ({
				...s,
				branches: result.map(b => ({
					name: b.name,
					isCurrent: b.is_current,
					isRemote: b.is_remote,
					ahead: b.ahead,
					behind: b.behind,
					lastCommit: b.last_commit,
				})),
			}));
		} catch (e) {
			console.error('Failed to refresh branches:', e);
		}
	}

	async function refreshCommits(limit = 50, skip = 0) {
		const path = getPath();
		if (!path) return;

		try {
			const result = await invoke<{
				sha: string;
				short_sha: string;
				message: string;
				author: string;
				author_email: string;
				date: string;
				relative_date: string;
				parents: string[];
			}[]>('git_log', { path, limit, skip });

			const commits = result.map(c => ({
				sha: c.sha,
				shortSha: c.short_sha,
				message: c.message,
				author: c.author,
				authorEmail: c.author_email,
				date: c.date,
				relativeDate: c.relative_date,
				parents: c.parents,
			}));

			update(s => ({
				...s,
				commits: skip === 0 ? commits : [...s.commits, ...commits],
			}));
		} catch (e) {
			console.error('Failed to refresh commits:', e);
		}
	}

	async function refreshStashes() {
		const path = getPath();
		if (!path) return;

		try {
			const result = await invoke<{
				index: number;
				message: string;
				branch: string;
				date: string;
			}[]>('git_stash_list', { path });

			update(s => ({
				...s,
				stashes: result.map(st => ({
					index: st.index,
					message: st.message,
					branch: st.branch,
					date: st.date,
				})),
			}));
		} catch (e) {
			console.error('Failed to refresh stashes:', e);
		}
	}

	// ─────────────────────────────────────────────────────────────────────────
	// Staging Actions
	// ─────────────────────────────────────────────────────────────────────────

	async function stage(files: string[]) {
		const path = getPath();
		if (!path) return;

		try {
			await invoke('git_stage', { path, files });
			await refresh();
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
		}
	}

	async function unstage(files: string[]) {
		const path = getPath();
		if (!path) return;

		try {
			await invoke('git_unstage', { path, files });
			await refresh();
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
		}
	}

	async function stageAll() {
		const path = getPath();
		if (!path) return;

		try {
			await invoke('git_stage_all', { path });
			await refresh();
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
		}
	}

	async function discardFile(file: string) {
		const path = getPath();
		if (!path) return;

		try {
			await invoke('git_discard_file', { path, file });
			await refresh();
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
		}
	}

	// ─────────────────────────────────────────────────────────────────────────
	// Commit Actions
	// ─────────────────────────────────────────────────────────────────────────

	async function commit(message: string): Promise<string | null> {
		const path = getPath();
		if (!path) return null;

		try {
			const sha = await invoke<string>('git_commit', { path, message });
			await refresh();
			await refreshCommits();
			return sha;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return null;
		}
	}

	async function commitAndPush(message: string): Promise<boolean> {
		const sha = await commit(message);
		if (!sha) return false;
		return await push();
	}

	// ─────────────────────────────────────────────────────────────────────────
	// Remote Actions
	// ─────────────────────────────────────────────────────────────────────────

	async function push(setUpstream = false): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_push', { path, setUpstream });
			await refresh();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	async function pull(): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_pull', { path });
			await refresh();
			await refreshCommits();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	async function fetch(): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_fetch', { path });
			await refresh();
			await refreshBranches();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	// ─────────────────────────────────────────────────────────────────────────
	// Branch Actions
	// ─────────────────────────────────────────────────────────────────────────

	async function checkout(branch: string, create = false): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_checkout', { path, branch, create });
			await refresh();
			await refreshBranches();
			await refreshCommits();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	async function createBranch(name: string, fromRef?: string): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_create_branch', { path, name, fromRef });
			await refresh();
			await refreshBranches();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	async function deleteBranch(name: string, force = false): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_delete_branch', { path, name, force });
			await refreshBranches();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	// ─────────────────────────────────────────────────────────────────────────
	// Stash Actions
	// ─────────────────────────────────────────────────────────────────────────

	async function stashCreate(message?: string): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_stash_create', { path, message });
			await refresh();
			await refreshStashes();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	async function stashApply(index: number): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_stash_apply', { path, index });
			await refresh();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	async function stashPop(index: number): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_stash_pop', { path, index });
			await refresh();
			await refreshStashes();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	async function stashDrop(index: number): Promise<boolean> {
		const path = getPath();
		if (!path) return false;

		try {
			await invoke('git_stash_drop', { path, index });
			await refreshStashes();
			return true;
		} catch (e) {
			update(s => ({ ...s, error: String(e) }));
			return false;
		}
	}

	// ─────────────────────────────────────────────────────────────────────────
	// Diff and Blame
	// ─────────────────────────────────────────────────────────────────────────

	async function getDiff(file: string, staged: boolean): Promise<string | null> {
		const path = getPath();
		if (!path) return null;

		try {
			return await invoke<string>('git_diff_file', { path, file, staged });
		} catch (e) {
			console.error('Failed to get diff:', e);
			return null;
		}
	}

	async function getFileAtHead(file: string): Promise<string | null> {
		const path = getPath();
		if (!path) return null;

		try {
			return await invoke<string>('git_file_content_at_head', { path, file });
		} catch (e) {
			console.error('Failed to get file at HEAD:', e);
			return null;
		}
	}

	async function getCommitDetail(sha: string): Promise<CommitDetail | null> {
		const path = getPath();
		if (!path) return null;

		try {
			const result = await invoke<{
				commit: {
					sha: string;
					short_sha: string;
					message: string;
					author: string;
					author_email: string;
					date: string;
					relative_date: string;
					parents: string[];
				};
				files: {
					added: string[];
					modified: string[];
					deleted: string[];
				};
				diff: string;
			}>('git_show_commit', { path, sha });

			return {
				commit: {
					sha: result.commit.sha,
					shortSha: result.commit.short_sha,
					message: result.commit.message,
					author: result.commit.author,
					authorEmail: result.commit.author_email,
					date: result.commit.date,
					relativeDate: result.commit.relative_date,
					parents: result.commit.parents,
				},
				files: result.files,
				diff: result.diff,
			};
		} catch (e) {
			console.error('Failed to get commit detail:', e);
			return null;
		}
	}

	async function getBlame(file: string): Promise<BlameLine[] | null> {
		const path = getPath();
		if (!path) return null;

		try {
			const result = await invoke<{
				sha: string;
				author: string;
				date: string;
				line_number: number;
				content: string;
			}[]>('git_blame_file', { path, file });

			return result.map(l => ({
				sha: l.sha,
				author: l.author,
				date: l.date,
				lineNumber: l.line_number,
				content: l.content,
			}));
		} catch (e) {
			console.error('Failed to get blame:', e);
			return null;
		}
	}

	// ─────────────────────────────────────────────────────────────────────────
	// Hybrid Watching (filesystem + fallback poll + window focus)
	// ─────────────────────────────────────────────────────────────────────────

	let unlistenGitChange: UnlistenFn | null = null;
	let unlistenFsChange: UnlistenFn | null = null;

	async function startWatching(path: string) {
		await stopWatching();
		currentPath = path;
		set({ ...emptyState });

		// Initial refresh
		await refresh();

		// 1. Watch .git directory for changes (instant updates)
		try {
			await invoke('watch_git_directory', { path });
			unlistenGitChange = await listen<{ reason: string }>('git-change', (event) => {
				console.log('[git] Change detected:', event.payload.reason);
				refresh();
				// Also refresh related data based on what changed
				if (event.payload.reason === 'refs' || event.payload.reason === 'head') {
					refreshBranches();
				}
				if (event.payload.reason === 'stash') {
					refreshStashes();
				}
			});
		} catch (e) {
			console.warn('[git] Failed to start git watcher:', e);
		}

		// 2. Listen to general filesystem changes (catches file saves)
		unlistenFsChange = await listen('fs-change', () => {
			// Debounce: only refresh if not already loading
			const state = get({ subscribe });
			if (!state.loading) {
				refresh();
			}
		});

		// 3. Fallback poll every 30 seconds (catches external git operations)
		pollInterval = setInterval(() => refresh(), 30000);

		// 4. Refresh on window focus (catches terminal git commands, external IDE changes)
		if (typeof window !== 'undefined') {
			window.addEventListener('focus', handleWindowFocus);
		}
	}

	function handleWindowFocus() {
		// Small delay to let any pending file operations complete
		setTimeout(() => refresh(), 100);
	}

	async function stopWatching() {
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
		if (unlistenGitChange) {
			unlistenGitChange();
			unlistenGitChange = null;
		}
		if (unlistenFsChange) {
			unlistenFsChange();
			unlistenFsChange = null;
		}
		if (typeof window !== 'undefined') {
			window.removeEventListener('focus', handleWindowFocus);
		}
		try {
			await invoke('stop_git_watching');
		} catch { /* ignore */ }
	}

	// Legacy alias for backward compatibility
	function startPolling(path: string) {
		startWatching(path);
	}

	function stopPolling() {
		stopWatching();
	}

	function clearError() {
		update(s => ({ ...s, error: null }));
	}

	return {
		subscribe,
		refresh,
		refreshBranches,
		refreshCommits,
		refreshStashes,
		stage,
		unstage,
		stageAll,
		discardFile,
		commit,
		commitAndPush,
		push,
		pull,
		fetch,
		checkout,
		createBranch,
		deleteBranch,
		stashCreate,
		stashApply,
		stashPop,
		stashDrop,
		getDiff,
		getFileAtHead,
		getCommitDetail,
		getBlame,
		startWatching,
		stopWatching,
		startPolling,
		stopPolling,
		clearError,
	};
}

export const git = createGitStore();

// ─────────────────────────────────────────────────────────────────────────────
// Derived Stores
// ─────────────────────────────────────────────────────────────────────────────

export const gitInfo = derived(git, $git => ({
	isRepo: $git.isRepo,
	branch: $git.branch,
	changedFiles: [
		...$git.staged.map(f => f.path),
		...$git.unstaged.map(f => f.path),
		...$git.untracked,
	],
}));

export const hasChanges = derived(git, $git => 
	$git.staged.length > 0 || $git.unstaged.length > 0 || $git.untracked.length > 0
);

export const totalChanges = derived(git, $git => 
	$git.staged.length + $git.unstaged.length + $git.untracked.length
);

export const localBranches = derived(git, $git => 
	$git.branches.filter(b => !b.isRemote)
);

export const remoteBranches = derived(git, $git => 
	$git.branches.filter(b => b.isRemote)
);

// ─────────────────────────────────────────────────────────────────────────────
// Helper for backward compatibility
// ─────────────────────────────────────────────────────────────────────────────

export function startGitPoller(cwd: string) {
	git.startPolling(cwd);
}
