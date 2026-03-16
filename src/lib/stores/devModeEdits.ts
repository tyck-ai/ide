import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { activeFilePath, openFileInEditor } from './editor';

export interface DevModeEdit {
	id: string;
	filePath: string;
	oldContent: string | null;   // null for Write (new file)
	newContent: string;
	status: 'pending' | 'accepted' | 'rejected';
	timestamp: number;
}

export const devModeEdits = writable<DevModeEdit[]>([]);

/** Pending edits for the currently active file */
export const activeFileEdits = derived(
	[devModeEdits, activeFilePath],
	([$edits, $path]) => $edits.filter(e => e.filePath === $path && e.status === 'pending')
);

/** Total count of all pending edits across all files */
export const pendingEditCount = derived(
	devModeEdits,
	$edits => $edits.filter(e => e.status === 'pending').length
);

/** Add a new edit from the agent */
export function addEdit(edit: Omit<DevModeEdit, 'status' | 'timestamp'>) {
	devModeEdits.update(edits => [...edits, {
		...edit,
		status: 'pending',
		timestamp: Date.now(),
	}]);

	// Auto-open the file if it's not open
	const currentPath = get(activeFilePath);
	if (currentPath !== edit.filePath) {
		const fileName = edit.filePath.split('/').pop() || edit.filePath;
		invoke<string>('read_file', { path: edit.filePath })
			.then(content => openFileInEditor(edit.filePath, fileName, content))
			.catch(() => { /* file may not exist yet for Write */ });
	}
}

/** Accept an edit — apply the change to the file on disk */
export async function acceptEdit(id: string) {
	const edits = get(devModeEdits);
	const edit = edits.find(e => e.id === id);
	if (!edit || edit.status !== 'pending') return;

	try {
		if (edit.oldContent !== null) {
			// Edit: read current file, apply the replacement, write back
			const current = await invoke<string>('read_file', { path: edit.filePath });
			const updated = current.replace(edit.oldContent, edit.newContent);
			await invoke('write_file', { path: edit.filePath, content: updated });
		} else {
			// Write: create/overwrite the file
			await invoke('write_file', { path: edit.filePath, content: edit.newContent });
		}

		devModeEdits.update(edits =>
			edits.map(e => e.id === id ? { ...e, status: 'accepted' as const } : e)
		);
	} catch (e) {
		console.error('Failed to accept edit:', e);
	}
}

/** Reject an edit — discard the change */
export function rejectEdit(id: string) {
	devModeEdits.update(edits =>
		edits.map(e => e.id === id ? { ...e, status: 'rejected' as const } : e)
	);
}

/** Accept all pending edits for a specific file */
export async function acceptAllForFile(filePath: string) {
	const edits = get(devModeEdits).filter(e => e.filePath === filePath && e.status === 'pending');
	for (const edit of edits) {
		await acceptEdit(edit.id);
	}
}

/** Reject all pending edits for a specific file */
export function rejectAllForFile(filePath: string) {
	devModeEdits.update(edits =>
		edits.map(e => e.filePath === filePath && e.status === 'pending'
			? { ...e, status: 'rejected' as const }
			: e
		)
	);
}

/** Clear all resolved (accepted/rejected) edits */
export function clearResolved() {
	devModeEdits.update(edits => edits.filter(e => e.status === 'pending'));
}
