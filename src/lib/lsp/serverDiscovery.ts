import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { lspMissingServers, dismissedLspNotifications } from '$lib/stores/lsp';
import { getServerConfig } from './serverRegistry';

// ─── Types ────────────────────────────────────────────────────────────────────

interface DirEntry {
	name: string;
	path: string;
	is_dir: boolean;
	children: DirEntry[] | null;
}

export interface LspBinaryStatus {
	language: string;
	binary: string;
	found: boolean;
	path: string | null;
	install_hint: string | null;
}

// ─── Extension → canonical language map ───────────────────────────────────────

const EXT_TO_LANG: Record<string, string> = {
	ts: 'typescript', tsx: 'typescript', js: 'typescript', jsx: 'typescript',
	mjs: 'typescript', cjs: 'typescript',
	rs: 'rust',
	py: 'python',
	go: 'go',
	rb: 'ruby',
	svelte: 'svelte',
	css: 'css', scss: 'css', less: 'css',
	html: 'html',
	json: 'json', jsonc: 'json',
	yaml: 'yaml', yml: 'yaml',
	toml: 'toml',
	graphql: 'graphql', gql: 'graphql',
	ex: 'elixir', exs: 'elixir',
	cs: 'csharp', csx: 'csharp',
	java: 'java',
	kt: 'kotlin', kts: 'kotlin',
	swift: 'swift',
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Collect unique file extensions from up to 4 levels of a directory tree. */
function collectExtensions(entries: DirEntry[], depth = 0): Set<string> {
	const exts = new Set<string>();
	for (const entry of entries) {
		if (!entry.is_dir) {
			const dot = entry.name.lastIndexOf('.');
			if (dot !== -1) exts.add(entry.name.slice(dot + 1).toLowerCase());
		} else if (depth < 3 && entry.children) {
			for (const ext of collectExtensions(entry.children, depth + 1)) {
				exts.add(ext);
			}
		}
	}
	return exts;
}

// ─── Public API ───────────────────────────────────────────────────────────────

/** Check whether the binary for a single language is available. */
export async function checkSingleServer(language: string): Promise<LspBinaryStatus> {
	return invoke<LspBinaryStatus>('lsp_check_binary', { language });
}

/**
 * Scan the workspace root for source files and return the canonical language
 * ids detected (e.g. ['typescript', 'python', 'rust']).
 * Returns an empty array if the root is unreadable.
 */
export async function detectWorkspaceLanguages(root: string): Promise<string[]> {
	let entries: DirEntry[] = [];
	try {
		entries = await invoke<DirEntry[]>('read_directory', { path: root });
	} catch {
		return [];
	}
	const exts = collectExtensions(entries);
	const languages = new Set<string>();
	for (const ext of exts) {
		const lang = EXT_TO_LANG[ext];
		if (lang) languages.add(lang);
	}
	return [...languages];
}

/**
 * Scan the project root for source files, then check each detected language
 * server binary. Missing servers (not dismissed by the user) are added to
 * the `lspMissingServers` store so the notification banner can display them.
 */
export async function checkProjectOnOpen(root: string): Promise<void> {
	let entries: DirEntry[] = [];
	try {
		entries = await invoke<DirEntry[]>('read_directory', { path: root });
	} catch {
		return; // non-fatal — project might not have a readable root
	}

	const exts = collectExtensions(entries);
	const languages = new Set<string>();
	for (const ext of exts) {
		const lang = EXT_TO_LANG[ext];
		if (lang) languages.add(lang);
	}
	if (languages.size === 0) return;

	const dismissed = get(dismissedLspNotifications);
	const missing: Array<{ language: string; displayName: string; installHint: string }> = [];

	await Promise.all(
		[...languages].map(async (lang) => {
			if (dismissed.has(lang)) return;
			const config = getServerConfig(lang);
			if (!config) return;
			try {
				const status = await checkSingleServer(lang);
				if (!status.found && status.install_hint) {
					missing.push({
						language: lang,
						displayName: config.displayName,
						installHint: status.install_hint,
					});
				}
			} catch {
				// binary check failures are non-fatal
			}
		}),
	);

	if (missing.length > 0) {
		lspMissingServers.update((existing) => {
			const seen = new Set(existing.map((m) => m.language));
			for (const m of missing) {
				if (!seen.has(m.language)) existing.push(m);
			}
			return existing;
		});
	}
}

