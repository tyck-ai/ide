import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { log } from '$lib/log';

export interface TerminalSession {
	id: string;
	title: string;
}

export const terminalSessions = writable<TerminalSession[]>([]);
export const activeTerminalId = writable<string | null>(null);
export const terminalVisible = writable(false);

export function addTerminal(id: string, title: string = 'zsh') {
	terminalSessions.update(sessions => [...sessions, { id, title }]);
	activeTerminalId.set(id);
}

export function removeTerminal(id: string) {
	let sessions: TerminalSession[] = [];
	terminalSessions.subscribe(s => sessions = s)();
	const remaining = sessions.filter(s => s.id !== id);
	terminalSessions.set(remaining);
	activeTerminalId.update(current => {
		if (current === id) {
			return remaining.length > 0 ? remaining[remaining.length - 1].id : null;
		}
		return current;
	});
}

export function toggleTerminal() {
	terminalVisible.update(v => !v);
}

/**
 * Open the terminal panel and send a command to the active (or newly created) PTY.
 * Handles the case where no terminal session exists yet — waits for TerminalPanel's
 * auto-create $effect to fire, then gives the PTY a moment to spawn before writing.
 */
export async function sendCommandToTerminal(cmd: string): Promise<void> {
	const hadSessions = get(terminalSessions).length > 0;

	// Show the panel (TerminalPanel auto-creates a session if there are none)
	terminalVisible.set(true);

	// Wait for an active terminal id if none exists yet
	let id = get(activeTerminalId);
	if (!id) {
		await new Promise<void>((resolve) => {
			const unsub = activeTerminalId.subscribe((val) => {
				if (val) {
					unsub();
					resolve();
				}
			});
			// Safety fallback — don't wait forever
			setTimeout(() => {
				unsub();
				resolve();
			}, 2000);
		});
		id = get(activeTerminalId);
	}

	if (!id) return;

	// Give the PTY time to spawn if this is a brand-new terminal session
	if (!hadSessions) {
		await new Promise((r) => setTimeout(r, 500));
	}

	await invoke('write_terminal', { id, data: cmd + '\r' }).catch((e) => log.warn('[terminal] sendCommandToTerminal write_terminal', e));
}
