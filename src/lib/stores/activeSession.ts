import { writable } from 'svelte/store';

/** The currently selected agent session ID. Shared across stores to avoid circular imports. */
export const activeSessionId = writable<string | null>(null);
