import type { Theme } from '../schema';

export const oneDark: Theme = {
	id: 'one-dark',
	name: 'One Dark',
	type: 'dark',
	colors: {
		base: '#282c34',
		surface: '#21252b',
		overlay: '#2c313a',
		border: '#3e4451',
		borderMuted: '#2c313a',

		text: '#abb2bf',
		textMuted: '#828997',
		textSubtle: '#5c6370',

		accent: '#61afef',
		accentHover: '#528bff',

		success: '#98c379',
		warning: '#e5c07b',
		error: '#e06c75',
		info: '#56b6c2',

		syntax: {
			keyword: '#c678dd',
			string: '#98c379',
			number: '#d19a66',
			function: '#61afef',
			comment: '#5c6370',
			type: '#e5c07b',
			operator: '#56b6c2',
			variable: '#e06c75',
			tag: '#e06c75',
			attribute: '#d19a66',
		},

		terminal: {
			background: '#282c34',
			foreground: '#abb2bf',
			cursor: '#528bff',
			cursorAccent: '#282c34',
			selectionBackground: '#3e445166',
			black: '#3f4451',
			red: '#e06c75',
			green: '#98c379',
			yellow: '#e5c07b',
			blue: '#61afef',
			magenta: '#c678dd',
			cyan: '#56b6c2',
			white: '#abb2bf',
			brightBlack: '#4f5666',
			brightRed: '#e06c75',
			brightGreen: '#98c379',
			brightYellow: '#e5c07b',
			brightBlue: '#61afef',
			brightMagenta: '#c678dd',
			brightCyan: '#56b6c2',
			brightWhite: '#d7dae0',
		},
	},
};
