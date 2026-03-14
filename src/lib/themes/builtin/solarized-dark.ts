import type { Theme } from '../schema';

export const solarizedDark: Theme = {
	id: 'solarized-dark',
	name: 'Solarized Dark',
	type: 'dark',
	colors: {
		base: '#002b36',
		surface: '#073642',
		overlay: '#094959',
		border: '#586e75',
		borderMuted: '#073642',

		text: '#839496',
		textMuted: '#657b83',
		textSubtle: '#586e75',

		accent: '#268bd2',
		accentHover: '#2aa198',

		success: '#859900',
		warning: '#b58900',
		error: '#dc322f',
		info: '#2aa198',

		syntax: {
			keyword: '#859900',
			string: '#2aa198',
			number: '#d33682',
			function: '#268bd2',
			comment: '#586e75',
			type: '#b58900',
			operator: '#839496',
			variable: '#cb4b16',
			tag: '#268bd2',
			attribute: '#b58900',
		},

		terminal: {
			background: '#002b36',
			foreground: '#839496',
			cursor: '#839496',
			cursorAccent: '#002b36',
			selectionBackground: '#073642',
			black: '#073642',
			red: '#dc322f',
			green: '#859900',
			yellow: '#b58900',
			blue: '#268bd2',
			magenta: '#d33682',
			cyan: '#2aa198',
			white: '#eee8d5',
			brightBlack: '#002b36',
			brightRed: '#cb4b16',
			brightGreen: '#586e75',
			brightYellow: '#657b83',
			brightBlue: '#839496',
			brightMagenta: '#6c71c4',
			brightCyan: '#93a1a1',
			brightWhite: '#fdf6e3',
		},
	},
};
