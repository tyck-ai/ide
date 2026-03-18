import { writable } from 'svelte/store';

export interface PendingEdit {
	filePath: string;
	oldContent: string;
	newContent: string;
	toolId: string;
	status: 'pending' | 'accepted' | 'rejected';
}

export const pendingEdits = writable<PendingEdit[]>([]);

export function updateEditStatus(toolId: string, status: 'accepted' | 'rejected') {
	pendingEdits.update(edits =>
		edits.map(e => e.toolId === toolId ? { ...e, status } : e)
	);
}
