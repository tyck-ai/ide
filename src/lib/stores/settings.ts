import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface TyckSettings {
	defaultProvider: string;
	lastOpenedFolder?: string;
	reviewEnabled: boolean;
	activeTheme: string;
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
	reviewEnabled: true,
	activeTheme: 'catppuccin-mocha'
});
export const detectedProviders = writable<ProviderInfo[]>([]);
export const settingsLoaded = writable(false);

export async function initSettings(): Promise<void> {
	try {
		const [s, providers] = await Promise.all([
			invoke<TyckSettings>('load_settings').catch(() => ({
				defaultProvider: 'claude-code',
				reviewEnabled: true,
				activeTheme: 'catppuccin-mocha',
			} as TyckSettings)),
			invoke<ProviderInfo[]>('detect_providers').catch(() => []),
		]);

		settings.set(s);
		detectedProviders.set(providers);
	} catch (e) {
		console.error('Failed to initialize settings:', e);
	} finally {
		// Always mark as loaded so the app doesn't hang
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
