import type { Theme } from '../schema';

export const catppuccinLatte: Theme = {
	id: 'catppuccin-latte',
	name: 'Catppuccin Latte',
	type: 'light',
	colors: {
		base: '#eff1f5',
		surface: '#e6e9ef',
		overlay: '#ccd0da',
		border: '#bcc0cc',
		borderMuted: '#ccd0da',

		text: '#4c4f69',
		textMuted: '#6c6f85',
		textSubtle: '#9ca0b0',

		accent: '#1e66f5',
		accentHover: '#7287fd',

		success: '#40a02b',
		warning: '#df8e1d',
		error: '#d20f39',
		info: '#179299',

		syntax: {
			keyword: '#8839ef',
			string: '#40a02b',
			number: '#fe640b',
			function: '#1e66f5',
			comment: '#9ca0b0',
			type: '#df8e1d',
			operator: '#04a5e5',
			variable: '#4c4f69',
			tag: '#8839ef',
			attribute: '#df8e1d',
		},

		terminal: {
			background: '#eff1f5',
			foreground: '#4c4f69',
			cursor: '#dc8a78',
			cursorAccent: '#eff1f5',
			selectionBackground: '#acb0be66',
			black: '#5c5f77',
			red: '#d20f39',
			green: '#40a02b',
			yellow: '#df8e1d',
			blue: '#1e66f5',
			magenta: '#ea76cb',
			cyan: '#179299',
			white: '#acb0be',
			brightBlack: '#6c6f85',
			brightRed: '#d20f39',
			brightGreen: '#40a02b',
			brightYellow: '#df8e1d',
			brightBlue: '#1e66f5',
			brightMagenta: '#ea76cb',
			brightCyan: '#179299',
			brightWhite: '#bcc0cc',
		},
	},
};
