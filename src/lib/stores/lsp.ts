import { writable } from 'svelte/store';

export interface MissingServer {
	language: string;
	displayName: string;
	installHint: string;
}

export interface ContextMenuState {
	visible: boolean;
	x: number;
	y: number;
	language: string;
}

export const lspMissingServers = writable<MissingServer[]>([]);
export const dismissedLspNotifications = writable<Set<string>>(new Set());

export const contextMenuStore = writable<ContextMenuState>({
	visible: false,
	x: 0,
	y: 0,
	language: '',
});

export interface DiagnosticMarker {
	filePath: string;      // relative path
	absPath: string;
	line: number;
	column: number;
	endLine: number;
	endColumn: number;
	message: string;
	severity: 1 | 2 | 3 | 4; // monaco: Error=8, Warn=4, Info=2, Hint=1 — we map to 1-4
}

export const diagnostics = writable<DiagnosticMarker[]>([]);
