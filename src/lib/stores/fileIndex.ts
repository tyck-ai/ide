import { writable } from 'svelte/store';

export interface FileEntry {
	name: string;
	path: string; // relative to project root
	absPath: string;
}

interface DirEntry {
	name: string;
	path: string;
	is_dir: boolean;
	children: DirEntry[] | null;
}

export const fileIndex = writable<FileEntry[]>([]);

export function buildFileIndex(tree: DirEntry[], rootPath: string): FileEntry[] {
	const entries: FileEntry[] = [];
	function walk(nodes: DirEntry[]) {
		for (const node of nodes) {
			if (!node.is_dir) {
				const relPath = node.path.startsWith(rootPath)
					? node.path.slice(rootPath.length).replace(/^\//, '')
					: node.path;
				entries.push({ name: node.name, path: relPath, absPath: node.path });
			}
			if (node.is_dir && node.children) {
				walk(node.children);
			}
		}
	}
	walk(tree);
	return entries;
}

export function fuzzyScore(query: string, entry: FileEntry): number {
	const q = query.toLowerCase();
	const name = entry.name.toLowerCase();
	const path = entry.path.toLowerCase();
	if (name === q) return 1000;
	if (name.startsWith(q)) return 900;
	if (name.includes(q)) return 800;
	if (path.includes(q)) return 700;
	return subsequenceScore(q, path);
}

function subsequenceScore(query: string, target: string): number {
	let qi = 0;
	let score = 0;
	let consecutive = 0;
	for (let ti = 0; ti < target.length && qi < query.length; ti++) {
		if (target[ti] === query[qi]) {
			qi++;
			consecutive++;
			score += consecutive * 10;
		} else {
			consecutive = 0;
		}
	}
	if (qi < query.length) return -1; // not a match
	return score;
}
