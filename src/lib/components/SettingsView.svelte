<script lang="ts">
	import { settings, detectedProviders, updateSettings, refreshProviders } from '$lib/stores/settings';
	import { showSettings } from '$lib/stores/layout';
	import {
		allThemes,
		customThemes,
		activeThemeId,
		setActiveTheme,
		loadCustomThemes,
		saveCustomTheme,
		deleteCustomTheme,
		builtinThemes,
		type Theme,
	} from '$lib/themes';
	import { onMount } from 'svelte';

	const sections = [
		{ id: 'appearance', label: 'Appearance' },
		{ id: 'agents', label: 'Agents' },
		{ id: 'review', label: 'Review' },
	] as const;

	type SectionId = (typeof sections)[number]['id'];
	let activeSection = $state<SectionId>('appearance');
	let refreshing = $state(false);
	let editingTheme = $state<Theme | null>(null);
	let themeJson = $state('');
	let jsonError = $state('');
	let savingTheme = $state(false);

	onMount(() => {
		loadCustomThemes();
	});

	async function setDefault(providerId: string) {
		await updateSettings({ defaultProvider: providerId });
	}

	async function onRefresh() {
		refreshing = true;
		await refreshProviders();
		refreshing = false;
	}

	async function selectTheme(themeId: string) {
		setActiveTheme(themeId);
		await updateSettings({ activeTheme: themeId });
	}

	function isBuiltinTheme(themeId: string): boolean {
		return builtinThemes.some(t => t.id === themeId);
	}

	function startEditTheme(theme: Theme) {
		editingTheme = theme;
		themeJson = JSON.stringify(theme, null, 2);
		jsonError = '';
	}

	function startCreateTheme() {
		const currentTheme = $allThemes.find(t => t.id === $activeThemeId);
		if (!currentTheme) return;

		const newTheme: Theme = {
			...JSON.parse(JSON.stringify(currentTheme)),
			id: `custom-${Date.now()}`,
			name: 'My Custom Theme',
		};
		editingTheme = newTheme;
		themeJson = JSON.stringify(newTheme, null, 2);
		jsonError = '';
	}

	async function saveEditedTheme() {
		try {
			const parsed = JSON.parse(themeJson) as Theme;
			if (!parsed.id || !parsed.name || !parsed.type || !parsed.colors) {
				jsonError = 'Invalid theme structure. Must have id, name, type, and colors.';
				return;
			}
			savingTheme = true;
			await saveCustomTheme(parsed);
			setActiveTheme(parsed.id);
			await updateSettings({ activeTheme: parsed.id });
			editingTheme = null;
			savingTheme = false;
		} catch (e) {
			jsonError = `Invalid JSON: ${e instanceof Error ? e.message : 'Unknown error'}`;
			savingTheme = false;
		}
	}

	function cancelEdit() {
		editingTheme = null;
		themeJson = '';
		jsonError = '';
	}

	async function onDeleteTheme(themeId: string) {
		if (!confirm('Delete this custom theme?')) return;
		await deleteCustomTheme(themeId);
		if ($activeThemeId === themeId) {
			await selectTheme('catppuccin-mocha');
		}
	}
</script>

<div class="settings-page">
	<!-- Sidebar -->
	<nav class="sidebar">
		<div class="sidebar-header">
			<span class="sidebar-title">Settings</span>
		</div>
		<div class="sidebar-nav">
			{#each sections as section (section.id)}
				<button
					class="nav-item"
					class:active={activeSection === section.id}
					onclick={() => activeSection = section.id}
				>{section.label}</button>
			{/each}
		</div>
		<div class="sidebar-footer">
			<button class="back-btn" onclick={() => showSettings.set(false)}>
				Back to Editor
			</button>
		</div>
	</nav>

	<!-- Content -->
	<main class="content">
		{#if activeSection === 'appearance'}
			<div class="content-header">
				<div class="content-title-row">
					<h1 class="content-title">Appearance</h1>
					<button class="create-theme-btn" onclick={startCreateTheme}>
						+ Create Custom Theme
					</button>
				</div>
				<p class="content-desc">Customize the look and feel of the IDE. Choose from built-in themes or create your own.</p>
			</div>

			{#if editingTheme}
				<div class="theme-editor">
					<div class="theme-editor-header">
						<span class="theme-editor-title">
							{isBuiltinTheme(editingTheme.id) ? 'Create Custom Theme (based on ' + editingTheme.name + ')' : 'Edit Theme: ' + editingTheme.name}
						</span>
						<div class="theme-editor-actions">
							<button class="cancel-btn" onclick={cancelEdit}>Cancel</button>
							<button class="save-btn" onclick={saveEditedTheme} disabled={savingTheme}>
								{savingTheme ? 'Saving...' : 'Save Theme'}
							</button>
						</div>
					</div>
					{#if jsonError}
						<div class="json-error">{jsonError}</div>
					{/if}
					<textarea
						class="theme-json-editor"
						bind:value={themeJson}
						rows="30"
						spellcheck="false"
					></textarea>
				</div>
			{:else}
				<div class="theme-section">
					<div class="section-label">Built-in Themes</div>
					<div class="theme-grid">
						{#each builtinThemes as theme (theme.id)}
							<button
								class="theme-card"
								class:active={$activeThemeId === theme.id}
								onclick={() => selectTheme(theme.id)}
							>
								<div class="theme-preview" style="background: {theme.colors.base}; border-color: {theme.colors.border};">
									<div class="preview-bar" style="background: {theme.colors.surface}; border-color: {theme.colors.border};">
										<span class="preview-dot" style="background: {theme.colors.error};"></span>
										<span class="preview-dot" style="background: {theme.colors.warning};"></span>
										<span class="preview-dot" style="background: {theme.colors.success};"></span>
									</div>
									<div class="preview-content">
										<div class="preview-line" style="background: {theme.colors.syntax.keyword}; width: 30%;"></div>
										<div class="preview-line" style="background: {theme.colors.syntax.function}; width: 50%;"></div>
										<div class="preview-line" style="background: {theme.colors.syntax.string}; width: 40%;"></div>
										<div class="preview-line" style="background: {theme.colors.text}; width: 60%;"></div>
									</div>
								</div>
								<div class="theme-info">
									<span class="theme-name">{theme.name}</span>
									<span class="theme-type">{theme.type}</span>
								</div>
								{#if $activeThemeId === theme.id}
									<span class="active-badge">Active</span>
								{/if}
							</button>
						{/each}
					</div>
				</div>

				{#if $customThemes.length > 0}
					<div class="theme-section">
						<div class="section-label">Custom Themes</div>
						<div class="theme-grid">
							{#each $customThemes as theme (theme.id)}
								<div class="theme-card-wrapper">
									<button
										class="theme-card"
										class:active={$activeThemeId === theme.id}
										onclick={() => selectTheme(theme.id)}
									>
										<div class="theme-preview" style="background: {theme.colors.base}; border-color: {theme.colors.border};">
											<div class="preview-bar" style="background: {theme.colors.surface}; border-color: {theme.colors.border};">
												<span class="preview-dot" style="background: {theme.colors.error};"></span>
												<span class="preview-dot" style="background: {theme.colors.warning};"></span>
												<span class="preview-dot" style="background: {theme.colors.success};"></span>
											</div>
											<div class="preview-content">
												<div class="preview-line" style="background: {theme.colors.syntax.keyword}; width: 30%;"></div>
												<div class="preview-line" style="background: {theme.colors.syntax.function}; width: 50%;"></div>
												<div class="preview-line" style="background: {theme.colors.syntax.string}; width: 40%;"></div>
												<div class="preview-line" style="background: {theme.colors.text}; width: 60%;"></div>
											</div>
										</div>
										<div class="theme-info">
											<span class="theme-name">{theme.name}</span>
											<span class="theme-type">{theme.type}</span>
										</div>
										{#if $activeThemeId === theme.id}
											<span class="active-badge">Active</span>
										{/if}
									</button>
									<div class="theme-card-actions">
										<button class="edit-btn" onclick={() => startEditTheme(theme)}>Edit</button>
										<button class="delete-btn" onclick={() => onDeleteTheme(theme.id)}>Delete</button>
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			{/if}

		{:else if activeSection === 'review'}
			<div class="content-header">
				<h1 class="content-title">Review</h1>
				<p class="content-desc">Control how AI-generated changes are handled. When review mode is enabled, changes made by agents require your approval before they appear in the editor.</p>
			</div>

			<div class="toggle-row">
				<div class="toggle-info">
					<span class="toggle-label">Review Mode</span>
					<span class="toggle-desc">Require approval for agent changes before they appear in the editor. When off, agent changes are applied immediately.</span>
				</div>
				<button
					class="toggle-switch"
					class:on={$settings.reviewEnabled}
					onclick={() => updateSettings({ reviewEnabled: !$settings.reviewEnabled })}
					role="switch"
					aria-checked={$settings.reviewEnabled}
				>
					<span class="toggle-knob"></span>
				</button>
			</div>
		{:else if activeSection === 'agents'}
			<div class="content-header">
				<div class="content-title-row">
					<h1 class="content-title">Agents</h1>
					<button class="refresh-btn" onclick={onRefresh} disabled={refreshing}>
						{refreshing ? 'Detecting...' : 'Re-detect'}
					</button>
				</div>
				<p class="content-desc">Manage coding agent CLI tools. The default agent is used when launching new sessions.</p>
			</div>

			<div class="provider-list">
				{#each $detectedProviders as provider (provider.id)}
					<div class="provider-row" class:unavailable={!provider.installed}>
						<div class="provider-info">
							<span class="provider-name">{provider.displayName}</span>
							{#if provider.installed}
								<span class="provider-path">{provider.resolvedPath}</span>
							{:else}
								<span class="provider-missing">Not installed</span>
							{/if}
						</div>
						<div class="provider-actions">
							{#if provider.installed}
								{#if $settings.defaultProvider === provider.id}
									<span class="default-badge">Default</span>
								{:else}
									<button class="set-default-btn" onclick={() => setDefault(provider.id)}>
										Set as default
									</button>
								{/if}
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</main>
</div>

<style>
	.settings-page {
		display: flex;
		height: 100%;
		background: var(--color-base);
		overflow: hidden;
	}

	/* ── Sidebar ── */
	.sidebar {
		width: 200px;
		flex-shrink: 0;
		background: var(--color-surface);
		border-right: 1px solid var(--color-border-muted);
		display: flex;
		flex-direction: column;
	}
	.sidebar-header {
		padding: 16px 16px 12px;
	}
	.sidebar-title {
		font-size: 11px;
		font-weight: 700;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.8px;
	}
	.sidebar-nav {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 1px;
		padding: 0 8px;
	}
	.nav-item {
		display: block;
		width: 100%;
		padding: 8px 12px;
		background: none;
		border: none;
		border-radius: 6px;
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		text-align: left;
		cursor: pointer;
	}
	.nav-item:hover {
		background: var(--color-base);
		color: var(--color-text);
	}
	.nav-item.active {
		background: var(--color-overlay);
		color: var(--color-text);
		font-weight: 600;
	}
	.sidebar-footer {
		padding: 12px;
		border-top: 1px solid var(--color-border-muted);
	}
	.back-btn {
		display: block;
		width: 100%;
		background: none;
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 600;
		padding: 6px 0;
		cursor: pointer;
		text-align: center;
	}
	.back-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
	}

	/* ── Content ── */
	.content {
		flex: 1;
		overflow-y: auto;
		padding: 32px 40px;
	}
	.content-header {
		margin-bottom: 24px;
	}
	.content-title-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.content-title {
		font-size: 20px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 6px;
	}
	.refresh-btn, .create-theme-btn {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-muted);
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		padding: 5px 12px;
		cursor: pointer;
	}
	.refresh-btn:hover:not(:disabled), .create-theme-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
	}
	.create-theme-btn {
		color: var(--color-accent);
		border-color: color-mix(in srgb, var(--color-accent) 40%, transparent);
	}
	.create-theme-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
		border-color: var(--color-accent);
	}
	.refresh-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.content-desc {
		font-size: 13px;
		color: var(--color-text-subtle);
		margin: 0;
		line-height: 1.5;
	}

	/* ── Theme Section ── */
	.theme-section {
		margin-bottom: 32px;
	}
	.section-label {
		font-size: 11px;
		font-weight: 700;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 12px;
	}
	.theme-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		gap: 12px;
		max-width: 800px;
	}
	.theme-card-wrapper {
		position: relative;
	}
	.theme-card {
		width: 100%;
		padding: 0;
		background: var(--color-surface);
		border: 2px solid var(--color-border-muted);
		border-radius: 10px;
		cursor: pointer;
		overflow: hidden;
		transition: all 0.15s;
	}
	.theme-card:hover {
		border-color: var(--color-border);
	}
	.theme-card.active {
		border-color: var(--color-accent);
	}
	.theme-preview {
		height: 80px;
		padding: 8px;
		border-bottom: 1px solid;
	}
	.preview-bar {
		display: flex;
		gap: 4px;
		padding: 4px 6px;
		border-radius: 4px;
		margin-bottom: 6px;
		border: 1px solid;
	}
	.preview-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
	}
	.preview-content {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.preview-line {
		height: 4px;
		border-radius: 2px;
		opacity: 0.8;
	}
	.theme-info {
		padding: 10px 12px;
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.theme-name {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text);
	}
	.theme-type {
		font-size: 10px;
		color: var(--color-text-subtle);
		text-transform: capitalize;
	}
	.active-badge {
		position: absolute;
		top: 8px;
		right: 8px;
		font-size: 9px;
		font-weight: 700;
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 20%, var(--color-surface));
		padding: 2px 6px;
		border-radius: 4px;
	}
	.theme-card-actions {
		display: flex;
		gap: 4px;
		margin-top: 4px;
	}
	.edit-btn, .delete-btn {
		flex: 1;
		padding: 4px 8px;
		font-size: 10px;
		font-weight: 600;
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		background: none;
		cursor: pointer;
	}
	.edit-btn {
		color: var(--color-accent);
	}
	.edit-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		border-color: var(--color-accent);
	}
	.delete-btn {
		color: var(--color-error);
	}
	.delete-btn:hover {
		background: color-mix(in srgb, var(--color-error) 10%, transparent);
		border-color: var(--color-error);
	}

	/* ── Theme Editor ── */
	.theme-editor {
		max-width: 800px;
	}
	.theme-editor-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 12px;
	}
	.theme-editor-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}
	.theme-editor-actions {
		display: flex;
		gap: 8px;
	}
	.cancel-btn, .save-btn {
		padding: 6px 14px;
		font-size: 12px;
		font-weight: 600;
		border-radius: 6px;
		cursor: pointer;
	}
	.cancel-btn {
		background: none;
		border: 1px solid var(--color-border-muted);
		color: var(--color-text-muted);
	}
	.cancel-btn:hover {
		color: var(--color-text);
		border-color: var(--color-border);
	}
	.save-btn {
		background: var(--color-accent);
		border: none;
		color: var(--color-base);
	}
	.save-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}
	.save-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
	.json-error {
		padding: 8px 12px;
		background: color-mix(in srgb, var(--color-error) 10%, transparent);
		border: 1px solid var(--color-error);
		border-radius: 6px;
		color: var(--color-error);
		font-size: 12px;
		margin-bottom: 12px;
	}
	.theme-json-editor {
		width: 100%;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		padding: 12px;
		color: var(--color-text);
		font-size: 12px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		resize: vertical;
		outline: none;
	}
	.theme-json-editor:focus {
		border-color: var(--color-accent);
	}

	/* ── Provider list ── */
	.provider-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-width: 560px;
	}
	.provider-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
	}
	.provider-row.unavailable {
		opacity: 0.45;
	}
	.provider-info {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
	}
	.provider-name {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}
	.provider-path {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-family: 'SF Mono', 'Fira Code', monospace;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 320px;
	}
	.provider-missing {
		font-size: 11px;
		color: var(--color-error);
	}
	.provider-actions {
		flex-shrink: 0;
		margin-left: 16px;
	}
	.default-badge {
		font-size: 11px;
		font-weight: 700;
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-success) 25%, transparent);
		padding: 4px 10px;
		border-radius: 6px;
	}
	.set-default-btn {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-accent);
		background: none;
		border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
		border-radius: 6px;
		padding: 4px 10px;
		cursor: pointer;
	}
	.set-default-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		border-color: var(--color-accent);
	}

	/* ── Toggle row ── */
	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		max-width: 560px;
	}
	.toggle-info {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
		margin-right: 16px;
	}
	.toggle-label {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}
	.toggle-desc {
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.4;
	}
	.toggle-switch {
		position: relative;
		width: 44px;
		height: 24px;
		border-radius: 12px;
		border: none;
		background: var(--color-border);
		cursor: pointer;
		flex-shrink: 0;
		transition: background 0.2s;
		padding: 0;
	}
	.toggle-switch.on {
		background: var(--color-accent);
	}
	.toggle-knob {
		position: absolute;
		top: 3px;
		left: 3px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: var(--color-text);
		transition: transform 0.2s;
	}
	.toggle-switch.on .toggle-knob {
		transform: translateX(20px);
	}
</style>
