import { derived } from 'svelte/store';
import { settings } from './settings';
import { DEFAULT_KEYBINDINGS } from '$lib/keybindings';

/** Resolved bindings: defaults merged with user overrides from settings. */
export const activeKeybindings = derived(settings, ($s) => {
	const overrides = $s.keybindings ?? {};
	return DEFAULT_KEYBINDINGS.map(kb => ({
		...kb,
		key: overrides[kb.id] ?? kb.defaultKey,
	}));
});

const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().includes('MAC');

/**
 * Returns true if a KeyboardEvent matches a chord string like "cmd+shift+f".
 * "cmd" means the primary modifier: ⌘ on macOS, Ctrl on Windows/Linux.
 * "ctrl" means Ctrl specifically (not the primary modifier on macOS).
 */
export function matchesBinding(e: KeyboardEvent, chord: string): boolean {
	const parts = chord.toLowerCase().split('+');
	const key = parts[parts.length - 1];
	const needsCmd   = parts.includes('cmd') || parts.includes('meta');
	const needsCtrl  = parts.includes('ctrl');
	const needsShift = parts.includes('shift');
	const needsAlt   = parts.includes('alt') || parts.includes('opt');

	// Primary modifier is Meta on Mac, Ctrl on others
	const primaryHeld = isMac ? e.metaKey : e.ctrlKey;
	// Bare Ctrl (not acting as primary)
	const ctrlHeld = isMac ? (e.ctrlKey && !e.metaKey) : e.ctrlKey;

	if (needsCmd && !primaryHeld) return false;
	if (!needsCmd && primaryHeld) return false;
	if (needsCtrl && !ctrlHeld) return false;
	if (!needsCtrl && !needsCmd && ctrlHeld) return false;
	if (needsShift !== e.shiftKey) return false;
	if (needsAlt !== e.altKey) return false;

	return e.key.toLowerCase() === key;
}
