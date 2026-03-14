export { catppuccinMocha } from './catppuccin-mocha';
export { catppuccinLatte } from './catppuccin-latte';
export { oneDark } from './one-dark';
export { githubDark } from './github-dark';
export { solarizedDark } from './solarized-dark';

import { catppuccinMocha } from './catppuccin-mocha';
import { catppuccinLatte } from './catppuccin-latte';
import { oneDark } from './one-dark';
import { githubDark } from './github-dark';
import { solarizedDark } from './solarized-dark';
import type { Theme } from '../schema';

export const builtinThemes: Theme[] = [
	catppuccinMocha,
	catppuccinLatte,
	oneDark,
	githubDark,
	solarizedDark,
];
