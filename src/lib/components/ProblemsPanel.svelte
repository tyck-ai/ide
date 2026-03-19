<script lang="ts">
	import { diagnostics, type DiagnosticMarker } from '$lib/stores/lsp';
	import { openFileInEditor } from '$lib/stores/editor';
	import { pendingEditorAction } from '$lib/stores/editor';
	import { invoke } from '@tauri-apps/api/core';
	import { activeSessionId, focusAgentTerminal } from '$lib/stores/activeSession';
	import { isAgentMode } from '$lib/stores/settings';
	import { get } from 'svelte/store';

	// Group diagnostics by file
	const grouped = $derived.by(() => {
		const map = new Map<string, typeof $diagnostics>();
		for (const d of $diagnostics) {
			if (!map.has(d.filePath)) map.set(d.filePath, []);
			map.get(d.filePath)!.push(d);
		}
		// Sort: files with errors first, then warnings
		return [...map.entries()].sort((a, b) => {
			const aMin = Math.min(...a[1].map(d => d.severity));
			const bMin = Math.min(...b[1].map(d => d.severity));
			return aMin - bMin;
		});
	});

	const errorCount = $derived($diagnostics.filter(d => d.severity === 1).length);
	const warnCount  = $derived($diagnostics.filter(d => d.severity === 2).length);

	let collapsedFiles = $state(new Set<string>());

	function toggleFile(path: string) {
		collapsedFiles = new Set(collapsedFiles);
		if (collapsedFiles.has(path)) collapsedFiles.delete(path);
		else collapsedFiles.add(path);
	}

	async function openDiagnostic(absPath: string, relPath: string, line: number) {
		try {
			const content: string = await invoke('read_file', { path: absPath });
			const name = relPath.split('/').pop() ?? relPath;
			openFileInEditor(absPath, name, content);
			pendingEditorAction.set({ type: 'goto-line', line });
		} catch {
			// ignore
		}
	}

	async function fixWithAgent(e: MouseEvent, d: DiagnosticMarker) {
		e.stopPropagation();
		const sessionId = get(activeSessionId);
		if (!sessionId) return;
		const text = `Fix this error in ${d.filePath}:${d.line}:\n${d.message}\n`;
		await invoke('write_terminal', { id: sessionId, data: text });
		focusAgentTerminal.update(n => n + 1);
	}

	function severityIcon(sev: number): string {
		switch (sev) {
			case 1: return '✕';
			case 2: return '⚠';
			case 3: return 'ℹ';
			default: return '·';
		}
	}

	function severityClass(sev: number): string {
		switch (sev) {
			case 1: return 'error';
			case 2: return 'warn';
			case 3: return 'info';
			default: return 'hint';
		}
	}
</script>

<div class="problems-panel">
	{#if $diagnostics.length === 0}
		<div class="empty">No problems detected</div>
	{:else}
		<div class="summary">
			{#if errorCount > 0}<span class="sev-count error">✕ {errorCount}</span>{/if}
			{#if warnCount > 0}<span class="sev-count warn">⚠ {warnCount}</span>{/if}
		</div>
		<div class="list">
			{#each grouped as [filePath, markers]}
				{@const collapsed = collapsedFiles.has(filePath)}
				{@const fileErrors = markers.filter(m => m.severity === 1).length}
				{@const fileWarns  = markers.filter(m => m.severity === 2).length}
				<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
				<div class="file-row" onclick={() => toggleFile(filePath)} role="button" tabindex="0">
					<span class="file-chevron">{collapsed ? '▸' : '▾'}</span>
					<span class="file-name">{filePath.split('/').pop()}</span>
					<span class="file-path-dir">{filePath.split('/').slice(0, -1).join('/')}</span>
					<span class="file-counts">
						{#if fileErrors > 0}<span class="sev-badge error">{fileErrors}</span>{/if}
						{#if fileWarns > 0}<span class="sev-badge warn">{fileWarns}</span>{/if}
					</span>
				</div>
				{#if !collapsed}
					{#each markers as d}
						<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
						<div
							class="diag-row {severityClass(d.severity)}"
							onclick={() => openDiagnostic(d.absPath, d.filePath, d.line)}
							role="button"
							tabindex="0"
						>
							<span class="diag-icon {severityClass(d.severity)}">{severityIcon(d.severity)}</span>
							<span class="diag-message">{d.message}</span>
							<span class="diag-loc">Ln {d.line}</span>
							{#if $isAgentMode && $activeSessionId}
								<button class="fix-btn" onclick={(e) => fixWithAgent(e, d)} title="Send to agent">→ Agent</button>
							{/if}
						</div>
					{/each}
				{/if}
			{/each}
		</div>
	{/if}
</div>

<style>
	.problems-panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
		font-size: 12px;
	}
	.empty {
		padding: 16px;
		color: var(--color-text-subtle);
		font-size: 12px;
	}
	.summary {
		display: flex;
		gap: 12px;
		padding: 4px 12px;
		border-bottom: 1px solid var(--color-border-muted);
		flex-shrink: 0;
	}
	.sev-count {
		font-size: 11px;
		font-weight: 600;
	}
	.sev-count.error { color: var(--color-error); }
	.sev-count.warn  { color: var(--color-warning); }
	.list {
		flex: 1;
		overflow-y: auto;
	}
	.file-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		cursor: pointer;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border-muted);
		user-select: none;
	}
	.file-row:hover { background: var(--color-overlay); }
	.file-chevron { font-size: 9px; color: var(--color-text-subtle); width: 10px; }
	.file-name { font-weight: 600; color: var(--color-text); font-size: 12px; }
	.file-path-dir { color: var(--color-text-subtle); font-size: 11px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
	.file-counts { display: flex; gap: 4px; flex-shrink: 0; }
	.sev-badge {
		font-size: 10px;
		font-weight: 700;
		padding: 0 5px;
		border-radius: 8px;
	}
	.sev-badge.error { background: color-mix(in srgb, var(--color-error) 15%, transparent); color: var(--color-error); }
	.sev-badge.warn  { background: color-mix(in srgb, var(--color-warning) 15%, transparent); color: var(--color-warning); }

	.diag-row {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 3px 10px 3px 24px;
		cursor: pointer;
	}
	.diag-row:hover { background: var(--color-overlay); }
	.diag-icon { font-size: 11px; flex-shrink: 0; margin-top: 1px; }
	.diag-icon.error { color: var(--color-error); }
	.diag-icon.warn  { color: var(--color-warning); }
	.diag-icon.info  { color: var(--color-accent); }
	.diag-icon.hint  { color: var(--color-text-subtle); }
	.diag-message { flex: 1; color: var(--color-text); line-height: 1.4; word-break: break-word; }
	.diag-loc { font-size: 11px; color: var(--color-text-subtle); white-space: nowrap; flex-shrink: 0; }
	.fix-btn {
		display: none;
		background: var(--color-overlay);
		border: 1px solid var(--color-border-muted);
		border-radius: 3px;
		color: var(--color-accent);
		font-size: 10px;
		padding: 1px 6px;
		cursor: pointer;
		white-space: nowrap;
		flex-shrink: 0;
	}
	.fix-btn:hover { background: var(--color-surface); }
	.diag-row:hover .fix-btn { display: inline-flex; }
</style>
