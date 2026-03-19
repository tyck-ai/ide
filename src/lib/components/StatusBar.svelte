<script lang="ts">
	import { activeFile, cursorLine, cursorColumn } from '$lib/stores/editor';
	import { showQuickOpen, quickOpenMode, showProblems } from '$lib/stores/layout';
	import { diagnostics } from '$lib/stores/lsp';
	import { terminalVisible } from '$lib/stores/terminal';

	const errorCount = $derived($diagnostics.filter(d => d.severity === 1).length);
	const warnCount  = $derived($diagnostics.filter(d => d.severity === 2).length);

	function toggleProblems() {
		const open = !$showProblems;
		showProblems.set(open);
		if (open) terminalVisible.set(false);
	}

	function getLanguage(filename: string): string {
		const ext = filename.split('.').pop() ?? '';
		const map: Record<string, string> = {
			ts: 'TypeScript', tsx: 'TypeScript JSX', js: 'JavaScript', jsx: 'JavaScript JSX',
			svelte: 'Svelte', rs: 'Rust', json: 'JSON', css: 'CSS', scss: 'SCSS',
			sass: 'Sass', less: 'Less', html: 'HTML', xml: 'XML', md: 'Markdown', mdx: 'MDX',
			toml: 'TOML', py: 'Python', go: 'Go', yaml: 'YAML', yml: 'YAML',
			sh: 'Shell', bash: 'Shell', zsh: 'Shell', rb: 'Ruby',
			graphql: 'GraphQL', gql: 'GraphQL', cs: 'C#', java: 'Java',
			kt: 'Kotlin', swift: 'Swift', c: 'C', cpp: 'C++', cc: 'C++', h: 'C/C++ Header',
			php: 'PHP', dart: 'Dart', vue: 'Vue', lock: 'Lockfile',
		};
		return map[ext] ?? 'Plain Text';
	}

	function openGotoLine() {
		quickOpenMode.set('line');
		showQuickOpen.set(true);
	}
</script>

{#if $activeFile}
	<div class="status-bar">
		<span class="segment language">
			{getLanguage($activeFile.name)}
		</span>
		<span class="separator">|</span>
		<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
		<span class="segment cursor clickable" onclick={openGotoLine} title="Go to Line (Ctrl+G)">
			Ln {$cursorLine}, Col {$cursorColumn}
		</span>
		{#if errorCount > 0 || warnCount > 0}
			<span class="separator">|</span>
			<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
			<span class="segment diag-counts clickable" onclick={toggleProblems} title="Toggle Problems (Cmd+Shift+M)">
				{#if errorCount > 0}<span class="diag-error">✕ {errorCount}</span>{/if}
				{#if warnCount > 0}<span class="diag-warn">⚠ {warnCount}</span>{/if}
			</span>
		{/if}
	</div>
{/if}

<style>
	.status-bar {
		display: flex;
		align-items: center;
		gap: 0;
		height: 22px;
		padding: 0 10px;
		background: var(--color-surface);
		border-top: 1px solid var(--color-border-muted);
		font-size: 11px;
		color: var(--color-text-subtle);
		flex-shrink: 0;
		user-select: none;
	}
	.segment {
		padding: 0 6px;
	}
	.separator {
		color: var(--color-border);
	}
	.clickable {
		cursor: pointer;
		border-radius: 3px;
	}
	.clickable:hover {
		background: var(--color-overlay);
		color: var(--color-text);
	}
	.diag-counts {
		display: flex;
		gap: 8px;
	}
	.diag-error { color: var(--color-error); }
	.diag-warn  { color: var(--color-warning); }
</style>
