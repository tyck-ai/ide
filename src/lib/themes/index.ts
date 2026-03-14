import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Theme, ThemeColors } from './schema';
import { builtinThemes } from './builtin';

export type { Theme, ThemeColors, SyntaxColors, TerminalColors } from './schema';
export { builtinThemes } from './builtin';

export const customThemes = writable<Theme[]>([]);
export const activeThemeId = writable<string>('catppuccin-mocha');

export const allThemes = derived(
	[customThemes],
	([$customThemes]) => [...builtinThemes, ...$customThemes]
);

export const activeTheme = derived(
	[activeThemeId, allThemes],
	([$activeThemeId, $allThemes]) => {
		const theme = $allThemes.find(t => t.id === $activeThemeId);
		return theme ?? builtinThemes[0];
	}
);

export async function loadCustomThemes(): Promise<void> {
	try {
		const themes = await invoke<Theme[]>('list_custom_themes');
		customThemes.set(themes);
	} catch {
		customThemes.set([]);
	}
}

export async function saveCustomTheme(theme: Theme): Promise<void> {
	await invoke('save_custom_theme', { theme });
	await loadCustomThemes();
}

export async function deleteCustomTheme(id: string): Promise<void> {
	await invoke('delete_custom_theme', { id });
	await loadCustomThemes();
}

export function injectCSSVariables(theme: Theme): void {
	const root = document.documentElement;
	const { colors } = theme;

	root.style.setProperty('--color-base', colors.base);
	root.style.setProperty('--color-surface', colors.surface);
	root.style.setProperty('--color-overlay', colors.overlay);
	root.style.setProperty('--color-border', colors.border);
	root.style.setProperty('--color-border-muted', colors.borderMuted);

	root.style.setProperty('--color-text', colors.text);
	root.style.setProperty('--color-text-muted', colors.textMuted);
	root.style.setProperty('--color-text-subtle', colors.textSubtle);

	root.style.setProperty('--color-accent', colors.accent);
	root.style.setProperty('--color-accent-hover', colors.accentHover);

	root.style.setProperty('--color-success', colors.success);
	root.style.setProperty('--color-warning', colors.warning);
	root.style.setProperty('--color-error', colors.error);
	root.style.setProperty('--color-info', colors.info);

	root.style.setProperty('--color-syntax-keyword', colors.syntax.keyword);
	root.style.setProperty('--color-syntax-string', colors.syntax.string);
	root.style.setProperty('--color-syntax-number', colors.syntax.number);
	root.style.setProperty('--color-syntax-function', colors.syntax.function);
	root.style.setProperty('--color-syntax-comment', colors.syntax.comment);
	root.style.setProperty('--color-syntax-type', colors.syntax.type);
	root.style.setProperty('--color-syntax-operator', colors.syntax.operator);
	root.style.setProperty('--color-syntax-variable', colors.syntax.variable);
	root.style.setProperty('--color-syntax-tag', colors.syntax.tag);
	root.style.setProperty('--color-syntax-attribute', colors.syntax.attribute);
}

export function generateMonacoTheme(theme: Theme): {
	base: 'vs' | 'vs-dark';
	inherit: boolean;
	rules: Array<{ token: string; foreground?: string; fontStyle?: string }>;
	colors: Record<string, string>;
} {
	const { colors } = theme;
	const stripHash = (color: string) => color.replace('#', '');

	return {
		base: theme.type === 'dark' ? 'vs-dark' : 'vs',
		inherit: true,
		rules: [
			{ token: '', foreground: stripHash(colors.text) },
			{ token: 'comment', foreground: stripHash(colors.syntax.comment), fontStyle: 'italic' },
			{ token: 'keyword', foreground: stripHash(colors.syntax.keyword) },
			{ token: 'keyword.control', foreground: stripHash(colors.syntax.keyword) },
			{ token: 'storage', foreground: stripHash(colors.syntax.keyword) },
			{ token: 'storage.type', foreground: stripHash(colors.syntax.type) },
			{ token: 'type', foreground: stripHash(colors.syntax.type) },
			{ token: 'type.identifier', foreground: stripHash(colors.syntax.type) },
			{ token: 'string', foreground: stripHash(colors.syntax.string) },
			{ token: 'string.escape', foreground: stripHash(colors.syntax.string) },
			{ token: 'number', foreground: stripHash(colors.syntax.number) },
			{ token: 'number.hex', foreground: stripHash(colors.syntax.number) },
			{ token: 'constant', foreground: stripHash(colors.syntax.number) },
			{ token: 'regexp', foreground: stripHash(colors.syntax.string) },
			{ token: 'variable', foreground: stripHash(colors.syntax.variable) },
			{ token: 'variable.predefined', foreground: stripHash(colors.error) },
			{ token: 'variable.parameter', foreground: stripHash(colors.syntax.variable) },
			{ token: 'function', foreground: stripHash(colors.syntax.function) },
			{ token: 'function.declaration', foreground: stripHash(colors.syntax.function) },
			{ token: 'operator', foreground: stripHash(colors.syntax.operator) },
			{ token: 'delimiter', foreground: stripHash(colors.textMuted) },
			{ token: 'delimiter.bracket', foreground: stripHash(colors.textMuted) },
			{ token: 'tag', foreground: stripHash(colors.syntax.tag) },
			{ token: 'attribute.name', foreground: stripHash(colors.syntax.attribute) },
			{ token: 'attribute.value', foreground: stripHash(colors.syntax.string) },
			{ token: 'metatag', foreground: stripHash(colors.error) },
			{ token: 'annotation', foreground: stripHash(colors.syntax.type) },
			{ token: 'namespace', foreground: stripHash(colors.syntax.type) },
			{ token: 'meta.embedded', foreground: stripHash(colors.text) },
		],
		colors: {
			'editor.background': colors.base,
			'editor.foreground': colors.text,
			'editor.lineHighlightBackground': colors.overlay + '20',
			'editor.selectionBackground': colors.border + '66',
			'editor.inactiveSelectionBackground': colors.border + '33',
			'editor.selectionHighlightBackground': colors.border + '40',
			'editor.findMatchBackground': colors.warning + '40',
			'editor.findMatchHighlightBackground': colors.warning + '20',
			'editorCursor.foreground': colors.terminal.cursor,
			'editorWhitespace.foreground': colors.overlay + '80',
			'editorIndentGuide.background': colors.overlay + '40',
			'editorIndentGuide.activeBackground': colors.border,
			'editorLineNumber.foreground': colors.border,
			'editorLineNumber.activeForeground': colors.textMuted,
			'editorBracketMatch.background': colors.border + '40',
			'editorBracketMatch.border': colors.border,
			'editorGutter.background': colors.base,
			'editorOverviewRuler.border': colors.surface,
			'editorWidget.background': colors.surface,
			'editorWidget.border': colors.overlay,
			'editorSuggestWidget.background': colors.surface,
			'editorSuggestWidget.border': colors.overlay,
			'editorSuggestWidget.selectedBackground': colors.overlay,
			'editorSuggestWidget.highlightForeground': colors.accent,
			'editorHoverWidget.background': colors.surface,
			'editorHoverWidget.border': colors.overlay,
			'peekView.border': colors.overlay,
			'peekViewEditor.background': colors.surface,
			'peekViewResult.background': colors.base,
			'minimap.background': colors.base,
			'scrollbar.shadow': colors.surface,
			'scrollbarSlider.background': colors.border + '40',
			'scrollbarSlider.hoverBackground': colors.border + '80',
			'scrollbarSlider.activeBackground': colors.border,
		},
	};
}

export function getXtermTheme(theme: Theme): Theme['colors']['terminal'] {
	return theme.colors.terminal;
}

let monacoInstance: typeof import('monaco-editor') | null = null;

export function setMonacoInstance(monaco: typeof import('monaco-editor')): void {
	monacoInstance = monaco;
}

export function applyTheme(theme: Theme): void {
	injectCSSVariables(theme);

	if (monacoInstance) {
		const monacoTheme = generateMonacoTheme(theme);
		monacoInstance.editor.defineTheme('tyck-theme', monacoTheme);
		monacoInstance.editor.setTheme('tyck-theme');
	}
}

export function getCurrentTheme(): Theme {
	return get(activeTheme);
}

export function setActiveTheme(themeId: string): void {
	activeThemeId.set(themeId);
	const theme = get(activeTheme);
	applyTheme(theme);
}
