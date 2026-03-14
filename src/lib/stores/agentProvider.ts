import { writable, derived } from 'svelte/store';
import { detectedProviders, settings } from './settings';

export interface AgentProvider {
	id: string;
	displayName: string;
	binary: string;
	args: string[];
	env: Record<string, string>;
}

const PROVIDER_DEFAULTS: Record<string, Omit<AgentProvider, 'id' | 'displayName' | 'binary'>> = {
	'claude-code': { args: [], env: {} },
	'codex': { args: [], env: {} },
	'cursor-agent': { args: [], env: {} },
	'copilot': { args: [], env: {} },
};

/** Only providers that are actually installed on this machine. */
export const agentProviders = derived(detectedProviders, ($detected) =>
	$detected
		.filter(p => p.installed)
		.map(p => ({
			id: p.id,
			displayName: p.displayName,
			binary: p.binary,
			...(PROVIDER_DEFAULTS[p.id] ?? { args: [], env: {} }),
		}))
);

export const activeProviderId = writable<string>('claude-code');

/** Sync activeProviderId from settings on load. */
export function applyDefaultProvider() {
	let unsub: (() => void) | undefined;
	unsub = settings.subscribe(s => {
		if (s.defaultProvider) {
			activeProviderId.set(s.defaultProvider);
		}
		// Only apply once
		if (unsub) unsub();
	});
}

export const activeProvider = derived(
	[agentProviders, activeProviderId],
	([$providers, $id]) => $providers.find(p => p.id === $id) ?? $providers[0]
);
