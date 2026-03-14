import type { Theme } from '../schema';

export const githubDark: Theme = {
	id: 'github-dark',
	name: 'GitHub Dark',
	type: 'dark',
	colors: {
		base: '#0d1117',
		surface: '#161b22',
		overlay: '#21262d',
		border: '#30363d',
		borderMuted: '#21262d',

		text: '#c9d1d9',
		textMuted: '#8b949e',
		textSubtle: '#6e7681',

		accent: '#58a6ff',
		accentHover: '#79c0ff',

		success: '#3fb950',
		warning: '#d29922',
		error: '#f85149',
		info: '#39c5cf',

		syntax: {
			keyword: '#ff7b72',
			string: '#a5d6ff',
			number: '#79c0ff',
			function: '#d2a8ff',
			comment: '#8b949e',
			type: '#ffa657',
			operator: '#79c0ff',
			variable: '#ffa657',
			tag: '#7ee787',
			attribute: '#79c0ff',
		},

		terminal: {
			background: '#0d1117',
			foreground: '#c9d1d9',
			cursor: '#c9d1d9',
			cursorAccent: '#0d1117',
			selectionBackground: '#264f7866',
			black: '#484f58',
			red: '#ff7b72',
			green: '#3fb950',
			yellow: '#d29922',
			blue: '#58a6ff',
			magenta: '#bc8cff',
			cyan: '#39c5cf',
			white: '#b1bac4',
			brightBlack: '#6e7681',
			brightRed: '#ffa198',
			brightGreen: '#56d364',
			brightYellow: '#e3b341',
			brightBlue: '#79c0ff',
			brightMagenta: '#d2a8ff',
			brightCyan: '#56d4dd',
			brightWhite: '#f0f6fc',
		},
	},
};
