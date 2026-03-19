export interface Keybinding {
	id: string;
	label: string;
	category: string;
	defaultKey: string;
	monacoActionId?: string;
}

export const DEFAULT_KEYBINDINGS: Keybinding[] = [
	// Navigation
	{ id: 'go-to-file',       label: 'Go to File',            category: 'Navigation', defaultKey: 'cmd+p' },
	{ id: 'go-to-line',       label: 'Go to Line',            category: 'Navigation', defaultKey: 'ctrl+g' },
	{ id: 'go-to-symbol',     label: 'Go to Symbol',          category: 'Navigation', defaultKey: 'cmd+shift+o' },

	// Search
	{ id: 'find-in-files',    label: 'Find in Files',         category: 'Search',     defaultKey: 'cmd+shift+f' },
	{ id: 'replace-in-files', label: 'Replace in Files',      category: 'Search',     defaultKey: 'cmd+shift+h' },

	// Editor
	{ id: 'save-file',        label: 'Save File',             category: 'Editor',     defaultKey: 'cmd+s' },
	{ id: 'format-document',  label: 'Format Document',       category: 'Editor',     defaultKey: 'shift+alt+f', monacoActionId: 'editor.action.formatDocument' },
	{ id: 'toggle-comment',   label: 'Toggle Line Comment',   category: 'Editor',     defaultKey: 'cmd+/', monacoActionId: 'editor.action.commentLine' },
	{ id: 'rename-symbol',    label: 'Rename Symbol',         category: 'Editor',     defaultKey: 'f2', monacoActionId: 'editor.action.rename' },
	{ id: 'go-to-definition', label: 'Go to Definition',      category: 'Editor',     defaultKey: 'f12', monacoActionId: 'editor.action.revealDefinition' },
	{ id: 'find-references',  label: 'Find All References',   category: 'Editor',     defaultKey: 'shift+f12', monacoActionId: 'editor.action.referenceSearch.trigger' },
	{ id: 'quick-fix',        label: 'Quick Fix',             category: 'Editor',     defaultKey: 'cmd+.', monacoActionId: 'editor.action.quickFix' },

	// View
	{ id: 'toggle-terminal',  label: 'Toggle Terminal',       category: 'View',       defaultKey: 'cmd+t' },
	{ id: 'toggle-sidebar',   label: 'Toggle Sidebar',        category: 'View',       defaultKey: 'cmd+b' },
	{ id: 'open-settings',    label: 'Open Settings',         category: 'View',       defaultKey: 'cmd+,' },
	{ id: 'problems-panel',   label: 'Toggle Problems',       category: 'View',       defaultKey: 'cmd+shift+m' },

	{ id: 'send-to-agent',    label: 'Send Selection to Agent', category: 'View',      defaultKey: 'cmd+shift+enter' },

	// Git
	{ id: 'git-view',         label: 'Open Git View',         category: 'Git',        defaultKey: 'cmd+g' },
	{ id: 'quick-commit',     label: 'Quick Commit',          category: 'Git',        defaultKey: 'cmd+shift+g' },
	{ id: 'branch-switcher',  label: 'Switch Branch',         category: 'Git',        defaultKey: 'cmd+shift+b' },
];

/** Group bindings by category, preserving order. */
export function groupByCategory(bindings: (Keybinding & { key: string })[]) {
	const groups = new Map<string, (Keybinding & { key: string })[]>();
	for (const kb of bindings) {
		if (!groups.has(kb.category)) groups.set(kb.category, []);
		groups.get(kb.category)!.push(kb);
	}
	return groups;
}

/** Display a chord like "cmd+shift+f" as "⌘⇧F". */
export function formatChord(chord: string): string {
	const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().includes('MAC');
	return chord
		.split('+')
		.map(part => {
			switch (part.toLowerCase()) {
				case 'cmd': case 'meta': return isMac ? '⌘' : 'Ctrl';
				case 'ctrl': return isMac ? '⌃' : 'Ctrl';
				case 'shift': return isMac ? '⇧' : 'Shift';
				case 'alt': case 'opt': return isMac ? '⌥' : 'Alt';
				case '`': return '`';
				case ',': return ',';
				case '.': return '.';
				case '/': return '/';
				default: return part.toUpperCase();
			}
		})
		.join('');
}
