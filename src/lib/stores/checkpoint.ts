import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

export interface FileDiff {
	path: string;
	status: string; // A, M, D
	additions: number;
	deletions: number;
}

export type FileDecision = 'pending' | 'accepted' | 'rejected';

export interface ReviewRecord {
	checkpointId: string;
	cwd: string;
	createdAt: string;
	completedAt: string;
	files: { path: string; status: string; decision: string }[];
}

export interface CheckpointState {
	active: boolean;
	checkpointId: string;
	cwd: string;
	diffs: FileDiff[];
	fileDecisions: Map<string, FileDecision>;
	reviewMode: boolean;
	selectedFile: string | null;
	loading: boolean;
	error: string | null;
	stashed: boolean;
}

const EMPTY_STATE: CheckpointState = {
	active: false,
	checkpointId: '',
	cwd: '',
	diffs: [],
	fileDecisions: new Map(),
	reviewMode: false,
	selectedFile: null,
	loading: false,
	error: null,
	stashed: false,
};

function createCheckpointStore() {
	const store = writable<CheckpointState>({ ...EMPTY_STATE, fileDecisions: new Map() });
	const { subscribe, set, update } = store;

	let pollInterval: ReturnType<typeof setInterval> | null = null;
	let fsUnlisten: (() => void) | null = null;
	let refreshInFlight = false;

	function cleanup() {
		if (pollInterval) { clearInterval(pollInterval); pollInterval = null; }
		if (fsUnlisten) { fsUnlisten(); fsUnlisten = null; }
	}

	function current(): CheckpointState {
		return get(store);
	}

	async function createCheckpoint(cwd: string, sessionId: string) {
		cleanup();

		set({
			active: true,
			checkpointId: sessionId,
			cwd,
			diffs: [],
			fileDecisions: new Map(),
			reviewMode: false,
			selectedFile: null,
			loading: false,
			error: null,
			stashed: false,
		});

		try {
			await invoke('create_checkpoint', { cwd, sessionId });
			console.log('[checkpoint] created checkpoint', sessionId);
		} catch (e) {
			console.warn('[checkpoint] create_checkpoint failed (will use HEAD fallback):', e);
		}

		// Start fs-change listener for live polling.
		// Filter by window label: Tauri v2 listen() is global across all windows.
		try {
			const myLabel = getCurrentWindow().label;
			const unlisten = await listen<{ windowLabel: string }>('fs-change', (event) => {
				if (event.payload.windowLabel !== myLabel) return;
				refreshDiffs();
			});
			fsUnlisten = unlisten;
		} catch { /* non-critical */ }

		// Poll every 3s while agent is running
		pollInterval = setInterval(() => refreshDiffs(), 3000);

		await refreshDiffs();
	}

	/** Live scan of working tree vs checkpoint (while agent is running) */
	async function refreshDiffs() {
		if (refreshInFlight) return;
		const state = current();
		if (!state.active || !state.cwd || state.stashed) return;

		refreshInFlight = true;
		try {
			const diffs = await invoke<FileDiff[]>('scan_changes', {
				cwd: state.cwd,
				checkpointId: state.checkpointId || null,
			});

			update(s => {
				const decisions = new Map(s.fileDecisions);
				for (const diff of diffs) {
					if (!decisions.has(diff.path)) {
						decisions.set(diff.path, 'pending');
					}
				}
				for (const key of decisions.keys()) {
					if (!diffs.find(d => d.path === key)) {
						decisions.delete(key);
					}
				}
				return { ...s, diffs, fileDecisions: decisions, error: null };
			});
		} catch (e) {
			const msg = String(e);
			console.warn('[checkpoint] scan_changes failed:', msg);
			update(s => ({ ...s, error: msg }));
		} finally {
			refreshInFlight = false;
		}
	}

	/** After agent finishes: capture changes, restore disk to checkpoint state */
	async function stashAgentChanges() {
		const state = current();
		if (!state.active || !state.cwd) return;

		// Stop live polling — no longer needed
		cleanup();

		try {
			const hasChanges = await invoke<boolean>('stash_agent_changes', {
				cwd: state.cwd,
				checkpointId: state.checkpointId,
			});

			if (!hasChanges) {
				console.log('[checkpoint] agent made no changes');
				set({ ...EMPTY_STATE, fileDecisions: new Map() });
				return;
			}

			// Load stashed diffs
			const diffs = await invoke<FileDiff[]>('scan_stashed_changes', {
				cwd: state.cwd,
				checkpointId: state.checkpointId,
			});

			const decisions = new Map<string, FileDecision>();
			for (const diff of diffs) {
				decisions.set(diff.path, 'pending');
			}

			update(s => ({
				...s,
				stashed: true,
				diffs,
				fileDecisions: decisions,
				error: null,
			}));

			console.log('[checkpoint] stashed agent changes,', diffs.length, 'files changed');
		} catch (e) {
			console.warn('[checkpoint] stash failed:', e);
			update(s => ({ ...s, error: String(e) }));
		}
	}

	function acceptFile(path: string) {
		update(s => {
			const d = new Map(s.fileDecisions);
			d.set(path, 'accepted');
			return { ...s, fileDecisions: d };
		});
	}

	function rejectFile(path: string) {
		update(s => {
			const d = new Map(s.fileDecisions);
			d.set(path, 'rejected');
			return { ...s, fileDecisions: d };
		});
	}

	function acceptAll() {
		update(s => {
			const d = new Map(s.fileDecisions);
			for (const key of d.keys()) d.set(key, 'accepted');
			return { ...s, fileDecisions: d };
		});
	}

	function rejectAll() {
		update(s => {
			const d = new Map(s.fileDecisions);
			for (const key of d.keys()) d.set(key, 'rejected');
			return { ...s, fileDecisions: d };
		});
	}

	async function enterReviewMode() {
		const state = current();

		// If not yet stashed, stash now (pulls agent changes off disk)
		if (!state.stashed && state.active && state.cwd) {
			try {
				const hasChanges = await invoke<boolean>('stash_agent_changes', {
					cwd: state.cwd,
					checkpointId: state.checkpointId,
				});

				if (!hasChanges) {
					// No changes to review
					return;
				}

				// Stop live polling — diffs are now static in git object store
				cleanup();

				const diffs = await invoke<FileDiff[]>('scan_stashed_changes', {
					cwd: state.cwd,
					checkpointId: state.checkpointId,
				});

				const decisions = new Map<string, FileDecision>();
				for (const diff of diffs) {
					decisions.set(diff.path, 'pending');
				}

				update(s => ({
					...s,
					stashed: true,
					diffs,
					fileDecisions: decisions,
					error: null,
					reviewMode: true,
					selectedFile: diffs.length > 0 ? diffs[0].path : null,
				}));
				console.log('[checkpoint] stashed and entering review,', diffs.length, 'files');
				return;
			} catch (e) {
				console.warn('[checkpoint] stash on review entry failed:', e);
				// Fall through to live review
			}
		}

		// Already stashed or stash failed — just enter review with current diffs
		if (!state.stashed) {
			await refreshDiffs();
		}
		update(s => ({
			...s,
			reviewMode: true,
			selectedFile: s.diffs.length > 0 ? s.diffs[0].path : null,
		}));
	}

	function exitReviewMode() {
		update(s => ({ ...s, reviewMode: false, selectedFile: null }));
	}

	function selectReviewFile(path: string) {
		update(s => ({ ...s, selectedFile: path }));
	}

	/** Finalize: apply accepted files to disk, save review record, clean up */
	async function finalizeReview() {
		const state = current();

		// Build decisions map — treat remaining pending as rejected
		const decisions: Record<string, string> = {};
		for (const [path, decision] of state.fileDecisions) {
			decisions[path] = decision === 'pending' ? 'rejected' : decision;
		}

		try {
			await invoke('finalize_review', {
				cwd: state.cwd,
				checkpointId: state.checkpointId,
				decisions,
			});
			console.log('[checkpoint] review finalized');
		} catch (e) {
			console.warn('[checkpoint] finalize_review failed:', e);
		}

		cleanup();
		set({ ...EMPTY_STATE, fileDecisions: new Map() });
	}

	// Keep for backward compat / non-stash flows
	async function finalizeCheckpoint(action: 'accept') {
		const state = current();
		try {
			await invoke('finalize_checkpoint', {
				cwd: state.cwd,
				checkpointId: state.checkpointId,
				action,
			});
		} catch (e) {
			console.warn('[checkpoint] finalize_checkpoint failed:', e);
		}
		cleanup();
		set({ ...EMPTY_STATE, fileDecisions: new Map() });
	}

	return {
		subscribe,
		createCheckpoint,
		refreshDiffs,
		acceptFile,
		rejectFile,
		acceptAll,
		rejectAll,
		enterReviewMode,
		exitReviewMode,
		selectReviewFile,
		finalizeReview,
		finalizeCheckpoint,
	};
}

export const checkpoint = createCheckpointStore();

export const hasActiveCheckpoint = derived(checkpoint, $cp => $cp.active);
export const pendingFiles = derived(checkpoint, $cp => {
	let count = 0;
	for (const decision of $cp.fileDecisions.values()) {
		if (decision === 'pending') count++;
	}
	return count;
});
export const reviewProgress = derived(checkpoint, $cp => {
	const total = $cp.fileDecisions.size;
	let resolved = 0;
	for (const decision of $cp.fileDecisions.values()) {
		if (decision !== 'pending') resolved++;
	}
	return { resolved, total };
});
