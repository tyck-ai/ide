import { writable, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';

export interface AgentStatus {
	modelId: string;
	modelName: string;
	contextUsedPercent: number;
	contextWindowSize: number;
	totalInputTokens: number;
	totalOutputTokens: number;
	totalCostUsd: number;
	linesAdded: number;
	linesRemoved: number;
	sessionId: string;
}

const empty: AgentStatus = {
	modelId: '',
	modelName: '',
	contextUsedPercent: 0,
	contextWindowSize: 0,
	totalInputTokens: 0,
	totalOutputTokens: 0,
	totalCostUsd: 0,
	linesAdded: 0,
	linesRemoved: 0,
	sessionId: '',
};

// The main status store — always reflects the active session
export const agentStatus = writable<AgentStatus>({ ...empty });
export const agentStatusConnected = writable<boolean>(false);

// Per-session status cache so we can switch without losing data
const statusCache = new Map<string, AgentStatus>();

// All sessions' statuses — reactive, keyed by session ID
export const allSessionStatuses = writable<Record<string, AgentStatus>>({});

// Which session ID the AwarenessBar is currently showing
let currentActiveId: string | null = null;

/**
 * Called when the user switches active session tab.
 * Updates the main store from the cache for the new session.
 */
export function switchActiveStatus(sessionId: string) {
	currentActiveId = sessionId;
	const cached = statusCache.get(sessionId);
	if (cached) {
		agentStatus.set(cached);
		agentStatusConnected.set(true);
	} else {
		agentStatus.set({ ...empty });
		agentStatusConnected.set(false);
	}
}

/**
 * Record a status update into the cache.
 * Only updates the main store if it's for the currently active session.
 */
export function recordStatus(watcherId: string, status: AgentStatus) {
	statusCache.set(watcherId, status);
	allSessionStatuses.update(map => ({ ...map, [watcherId]: status }));
	if (watcherId === currentActiveId) {
		agentStatus.set(status);
		agentStatusConnected.set(true);
	}
}

let listening = false;

export function startAgentStatusListener() {
	if (listening) return;
	listening = true;

	listen<{ watcherId: string; status: AgentStatus }>('agent-status', (event) => {
		const { watcherId, status } = event.payload;
		recordStatus(watcherId, status);
	});
}
