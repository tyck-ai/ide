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
