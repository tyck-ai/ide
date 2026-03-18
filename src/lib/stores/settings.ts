import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export type WorkspaceMode = 'dev' | 'agent';

export interface TyckSettings {
	defaultProvider: string;
	lastOpenedFolder?: string;
	workspaceMode: WorkspaceMode;
	activeTheme: string;
	lspFormatOnSave?: boolean;
	lspDismissed?: string[];
}

export interface ProviderInfo {
	id: string;
	displayName: string;
	binary: string;
	installed: boolean;
	resolvedPath: string;
}

export const settings = writable<TyckSettings>({
	defaultProvider: 'claude-code',
	workspaceMode: 'dev',
	activeTheme: 'catppuccin-mocha'
});
export const detectedProviders = writable<ProviderInfo[]>([]);
export const settingsLoaded = writable(false);

/** Derived stores for easy mode checking */
export const isAgentMode = derived(settings, $s => $s.workspaceMode === 'agent');
export const isDevMode = derived(settings, $s => $s.workspaceMode === 'dev');

export async function initSettings(): Promise<void> {
	try {
		const [s, providers] = await Promise.all([
			invoke<TyckSettings>('load_settings').catch(() => ({
				defaultProvider: 'claude-code',
				workspaceMode: 'dev',
				activeTheme: 'catppuccin-mocha',
			} as TyckSettings)),
			invoke<ProviderInfo[]>('detect_providers').catch(() => []),
		]);

		settings.set(s);
		detectedProviders.set(providers);

		// Restore dismissed LSP notifications so banners don't re-appear after restart
		if (s.lspDismissed?.length) {
			const { dismissedLspNotifications } = await import('$lib/stores/lsp');
			dismissedLspNotifications.set(new Set(s.lspDismissed));
		}
	} catch (e) {
		console.error('Failed to initialize settings:', e);
	} finally {
		settingsLoaded.set(true);
	}
}

export async function refreshProviders(): Promise<void> {
	const providers = await invoke<ProviderInfo[]>('detect_providers');
	detectedProviders.set(providers);
}

export async function updateSettings(partial: Partial<TyckSettings>): Promise<void> {
	const current = get(settings);
	const updated = { ...current, ...partial };
	settings.set(updated);
	await invoke('save_settings', { settings: updated });
}
