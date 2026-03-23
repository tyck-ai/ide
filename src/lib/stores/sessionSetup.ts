import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';

export type SetupStep = 'workspace' | 'files' | 'agent' | 'started';

export interface SessionSetup {
	step: SetupStep;
	progressText?: string;
}

/** Map of session_id → current setup step. Entries are removed once agent starts. */
export const sessionSetups = writable<Map<string, SessionSetup>>(new Map());

/**
 * Marks a session as "in setup" and begins listening for step events from the
 * backend.  Should be called before `spawn_agent_with_worktree_setup` is invoked
 * so the listener is registered before any events are emitted.
 *
 * Returns a cleanup function that stops listening and removes the entry.
 */
export async function trackSessionSetup(sessionId: string): Promise<() => void> {
	// Synchronous update — happens before any await so the store is populated
	// before we register the listener or invoke the Rust command.
	sessionSetups.update((m) => {
		const next = new Map(m);
		next.set(sessionId, { step: 'workspace' });
		return next;
	});

	let unlisten: (() => void) | null = null;
	let unlistenProgress: (() => void) | null = null;

	unlisten = await listen<SetupStep>(`session-setup-step-${sessionId}`, (event) => {
		const step = event.payload;
		sessionSetups.update((m) => {
			const next = new Map(m);
			const existing = next.get(sessionId);
			next.set(sessionId, { ...existing, step });
			return next;
		});

		if (step === 'started') {
			// Give the fade-out transition time to complete before removing the entry.
			setTimeout(() => {
				clearSessionSetup(sessionId);
				unlisten?.();
				unlistenProgress?.();
			}, 400);
		}
	});

	unlistenProgress = await listen<string>(`session-setup-progress-${sessionId}`, (event) => {
		const progressText = event.payload;
		sessionSetups.update((m) => {
			const next = new Map(m);
			const existing = next.get(sessionId);
			if (existing) next.set(sessionId, { ...existing, progressText });
			return next;
		});
	});

	return () => {
		unlisten?.();
		unlistenProgress?.();
		clearSessionSetup(sessionId);
	};
}

export function clearSessionSetup(sessionId: string): void {
	sessionSetups.update((m) => {
		const next = new Map(m);
		next.delete(sessionId);
		return next;
	});
}
