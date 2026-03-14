import { writable } from 'svelte/store';

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
	terminalSessions.update(sessions => sessions.filter(s => s.id !== id));
	activeTerminalId.update(current => {
		if (current === id) {
			let remaining: TerminalSession[] = [];
			terminalSessions.subscribe(s => remaining = s)();
			return remaining.length > 0 ? remaining[remaining.length - 1].id : null;
		}
		return current;
	});
}

export function toggleTerminal() {
	terminalVisible.update(v => !v);
}
