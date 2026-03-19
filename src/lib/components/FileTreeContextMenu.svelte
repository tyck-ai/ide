<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { toast } from '$lib/stores/toast';
	import { activeSessionId, focusAgentTerminal } from '$lib/stores/activeSession';
	import { projectRoot } from '$lib/stores/editor';
	import { get } from 'svelte/store';

	interface DirEntry {
		name: string;
		path: string;
		is_dir: boolean;
		children: DirEntry[] | null;
	}

	interface Props {
		visible: boolean;
		x: number;
		y: number;
		entry: DirEntry | null;
		parentPath: string;
		onclose: () => void;
		onrefresh: () => void;
		onrename: (entry: DirEntry) => void;
		oncreate: (parentPath: string, isDir: boolean) => void;
	}

	let { visible, x, y, entry, parentPath, onclose, onrefresh, onrename, oncreate }: Props = $props();

	let confirmDelete = $state(false);

	$effect(() => {
		if (!visible) confirmDelete = false;
	});

	async function openFile() {
		onclose();
	}

	async function deleteEntry() {
		if (!entry) return;
		try {
			await invoke('delete_path', { path: entry.path });
			toast.info(`Deleted ${entry.name}`);
			onrefresh();
		} catch (e) {
			toast.error(`Failed to delete: ${e}`);
		}
		onclose();
	}

	async function copyName() {
		if (!entry) return;
		try { await navigator.clipboard.writeText(entry.name); } catch { toast.error('Clipboard unavailable'); }
		onclose();
	}

	async function copyPath() {
		if (!entry) return;
		try { await navigator.clipboard.writeText(entry.path); } catch { toast.error('Clipboard unavailable'); }
		onclose();
	}

	async function copyRelPath() {
		if (!entry) return;
		const rel = entry.path.slice(parentPath.length).replace(/^\//, '');
		try { await navigator.clipboard.writeText(rel); } catch { toast.error('Clipboard unavailable'); }
		onclose();
	}

	async function reveal() {
		if (!entry) return;
		try {
			await invoke('reveal_in_file_manager', { path: entry.path });
		} catch (e) {
			toast.error(`Failed to reveal: ${e}`);
		}
		onclose();
	}

	function rename() {
		if (!entry) return;
		onrename(entry);
		onclose();
	}

	async function sendPathToAgent() {
		if (!entry) return;
		const sessionId = get(activeSessionId);
		if (!sessionId) {
			toast.error('No active agent session');
			onclose();
			return;
		}
		const root = get(projectRoot) ?? parentPath;
		const rel = entry.path.startsWith(root)
			? entry.path.slice(root.length).replace(/^\//, '')
			: entry.path;
		const text = entry.is_dir ? `${rel}/` : rel;
		try {
			await invoke('write_terminal', { id: sessionId, data: text });
			focusAgentTerminal.update(n => n + 1);
		} catch (e) {
			toast.error(`Failed to send path: ${e}`);
		}
		onclose();
	}

	function newFile() {
		const target = entry?.is_dir ? entry.path : parentPath;
		oncreate(target, false);
		onclose();
	}

	function newFolder() {
		const target = entry?.is_dir ? entry.path : parentPath;
		oncreate(target, true);
		onclose();
	}

	function menuStyle(): string {
		// Keep menu on screen
		const menuW = 200;
		const menuH = 280;
		let left = x;
		let top = y;
		if (typeof window !== 'undefined') {
			if (left + menuW > window.innerWidth) left = Math.max(0, window.innerWidth - menuW - 8);
			if (top + menuH > window.innerHeight) top = Math.max(0, window.innerHeight - menuH - 8);
		}
		return `left: ${left}px; top: ${top}px`;
	}
</script>

{#if visible}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={onclose}></div>
	<div class="context-menu" style={menuStyle()} role="menu">
		{#if entry && !entry.is_dir}
			<button class="menu-item" role="menuitem" onclick={openFile}>Open</button>
			<div class="separator"></div>
		{/if}

		<button class="menu-item" role="menuitem" onclick={newFile}>New File...</button>
		<button class="menu-item" role="menuitem" onclick={newFolder}>New Folder...</button>

		{#if entry}
			<div class="separator"></div>
			<button class="menu-item" role="menuitem" onclick={rename}>
				Rename <span class="shortcut">F2</span>
			</button>
			{#if !confirmDelete}
				<button class="menu-item danger" role="menuitem" onclick={() => confirmDelete = true}>
					Delete <span class="shortcut">⌫</span>
				</button>
			{:else}
				<div class="delete-confirm">
					<span class="confirm-text">Delete "{entry.name}"?</span>
					<div class="confirm-btns">
						<button class="confirm-btn cancel" onclick={() => confirmDelete = false}>Cancel</button>
						<button class="confirm-btn delete" onclick={deleteEntry}>Delete</button>
					</div>
				</div>
			{/if}
			<div class="separator"></div>
			{#if !entry.is_dir}
				<button class="menu-item" role="menuitem" onclick={copyName}>Copy Name</button>
			{/if}
			<button class="menu-item" role="menuitem" onclick={copyPath}>Copy Path</button>
			<button class="menu-item" role="menuitem" onclick={copyRelPath}>Copy Relative Path</button>
			<div class="separator"></div>
			<button class="menu-item" role="menuitem" onclick={sendPathToAgent}>Send Path to Agent</button>
			<div class="separator"></div>
			<button class="menu-item" role="menuitem" onclick={reveal}>Reveal in Finder</button>
		{/if}
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 600;
	}
	.context-menu {
		position: fixed;
		z-index: 601;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
		padding: 4px 0;
		min-width: 200px;
	}
	.menu-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 5px 14px;
		background: none;
		border: none;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		text-align: left;
	}
	.menu-item:hover {
		background: var(--color-overlay);
	}
	.menu-item.danger {
		color: var(--color-error);
	}
	.shortcut {
		font-size: 10px;
		color: var(--color-text-subtle);
		margin-left: 12px;
	}
	.separator {
		height: 1px;
		background: var(--color-border-muted);
		margin: 3px 0;
	}
	.delete-confirm {
		padding: 6px 10px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.confirm-text {
		font-size: 11px;
		color: var(--color-error);
	}
	.confirm-btns {
		display: flex;
		gap: 6px;
		justify-content: flex-end;
	}
	.confirm-btn {
		padding: 3px 10px;
		border-radius: 4px;
		font-size: 11px;
		font-weight: 600;
		cursor: pointer;
		border: none;
	}
	.confirm-btn.cancel {
		background: var(--color-overlay);
		color: var(--color-text);
	}
	.confirm-btn.delete {
		background: var(--color-error);
		color: white;
	}
	.confirm-btn:hover {
		filter: brightness(1.1);
	}
</style>
