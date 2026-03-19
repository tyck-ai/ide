import { writable } from 'svelte/store';

export interface SearchMatch {
	filePath: string;
	lineNumber: number;
	column: number;
	lineText: string;
	matchStart: number;
	matchEnd: number;
}

export interface SearchResultFile {
	filePath: string;
	matches: SearchMatch[];
	expanded: boolean;
	selectedForReplace: boolean;
}

export interface SearchState {
	query: string;
	replacement: string;
	isRegex: boolean;
	caseSensitive: boolean;
	includeGlob: string;
	results: SearchResultFile[];
	loading: boolean;
	replaceMode: boolean;
	error: string | null;
}

export const searchState = writable<SearchState>({
	query: '',
	replacement: '',
	isRegex: false,
	caseSensitive: false,
	includeGlob: '',
	results: [],
	loading: false,
	replaceMode: false,
	error: null,
});
