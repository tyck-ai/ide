export interface SyntaxColors {
	keyword: string;
	string: string;
	number: string;
	function: string;
	comment: string;
	type: string;
	operator: string;
	variable: string;
	tag: string;
	attribute: string;
}

export interface TerminalColors {
	background: string;
	foreground: string;
	cursor: string;
	cursorAccent: string;
	selectionBackground: string;
	black: string;
	red: string;
	green: string;
	yellow: string;
	blue: string;
	magenta: string;
	cyan: string;
	white: string;
	brightBlack: string;
	brightRed: string;
	brightGreen: string;
	brightYellow: string;
	brightBlue: string;
	brightMagenta: string;
	brightCyan: string;
	brightWhite: string;
}

export interface ThemeColors {
	// Core UI
	base: string;
	surface: string;
	overlay: string;
	border: string;
	borderMuted: string;

	// Text
	text: string;
	textMuted: string;
	textSubtle: string;

	// Accent colors
	accent: string;
	accentHover: string;

	// Semantic
	success: string;
	warning: string;
	error: string;
	info: string;

	// Syntax highlighting
	syntax: SyntaxColors;

	// Terminal ANSI colors
	terminal: TerminalColors;
}

export interface Theme {
	id: string;
	name: string;
	type: 'dark' | 'light';
	colors: ThemeColors;
}

export interface CustomThemeMeta {
	id: string;
	name: string;
	type: 'dark' | 'light';
	path: string;
}
