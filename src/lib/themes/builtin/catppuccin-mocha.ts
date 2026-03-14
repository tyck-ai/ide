import type { Theme } from '../schema';

export const catppuccinMocha: Theme = {
	id: 'catppuccin-mocha',
	name: 'Catppuccin Mocha',
	type: 'dark',
	colors: {
		base: '#1e1e2e',
		surface: '#181825',
		overlay: '#313244',
		border: '#45475a',
		borderMuted: '#313244',

		text: '#cdd6f4',
		textMuted: '#a6adc8',
		textSubtle: '#6c7086',

		accent: '#89b4fa',
		accentHover: '#b4befe',

		success: '#a6e3a1',
		warning: '#f9e2af',
		error: '#f38ba8',
		info: '#94e2d5',

		syntax: {
			keyword: '#cba6f7',
			string: '#a6e3a1',
			number: '#fab387',
			function: '#89b4fa',
			comment: '#6c7086',
			type: '#f9e2af',
			operator: '#89dceb',
			variable: '#cdd6f4',
			tag: '#cba6f7',
			attribute: '#f9e2af',
		},

		terminal: {
			background: '#1e1e2e',
			foreground: '#cdd6f4',
			cursor: '#f5e0dc',
			cursorAccent: '#1e1e2e',
			selectionBackground: '#585b7066',
			black: '#45475a',
			red: '#f38ba8',
			green: '#a6e3a1',
			yellow: '#f9e2af',
			blue: '#89b4fa',
			magenta: '#f5c2e7',
			cyan: '#94e2d5',
			white: '#bac2de',
			brightBlack: '#585b70',
			brightRed: '#f38ba8',
			brightGreen: '#a6e3a1',
			brightYellow: '#f9e2af',
			brightBlue: '#89b4fa',
			brightMagenta: '#f5c2e7',
			brightCyan: '#94e2d5',
			brightWhite: '#a6adc8',
		},
	},
};
