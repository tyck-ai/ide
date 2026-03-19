<script lang="ts">
	import { settings, detectedProviders, updateSettings, refreshProviders } from '$lib/stores/settings';
	import { showSettings, pendingInstall } from '$lib/stores/layout';
	import { lspStatuses, lspClientManager } from '$lib/lsp/LspClientManager';
	import { lspMissingServers, dismissedLspNotifications } from '$lib/stores/lsp';
	import { checkProjectOnOpen } from '$lib/lsp/serverDiscovery';
	import { projectRoot } from '$lib/stores/editor';
	import { supportedLanguages, getServerConfig } from '$lib/lsp/serverRegistry';
	import { tapp, installedApps, storeListings, storeLoading, availableUpdates, type AppInfo, type AppListing } from '$lib/stores/tapp';
	import { open } from '@tauri-apps/plugin-dialog';
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
	import { log } from '$lib/log';
	import { activeKeybindings } from '$lib/stores/keybindings';
	import { DEFAULT_KEYBINDINGS, groupByCategory, formatChord } from '$lib/keybindings';

	const sections = [
		{ id: 'appearance', label: 'Appearance' },
		{ id: 'workspace', label: 'Workspace' },
		{ id: 'agents', label: 'Agents' },
		{ id: 'apps', label: 'Apps' },
		{ id: 'language-servers', label: 'Language Servers' },
		{ id: 'keybindings', label: 'Keyboard Shortcuts' },
	] as const;

	type SectionId = (typeof sections)[number]['id'];
	let activeSection = $state<SectionId>('appearance');
	let refreshing = $state(false);
	let editingTheme = $state<Theme | null>(null);
	let themeJson = $state('');
	let jsonError = $state('');
	let savingTheme = $state(false);

	// Keyboard Shortcuts section state
	let kbSearchQuery = $state('');
	let recordingId = $state<string | null>(null);
	let conflictInfo = $state<{ withId: string; withLabel: string } | null>(null);
	let pendingChord = $state<string | null>(null);

	const filteredBindings = $derived.by(() => {
		const q = kbSearchQuery.toLowerCase().trim();
		return $activeKeybindings.filter(kb =>
			!q || kb.label.toLowerCase().includes(q) || kb.key.toLowerCase().includes(q) || kb.category.toLowerCase().includes(q)
		);
	});

	function startRecording(id: string) {
		recordingId = id;
		conflictInfo = null;
		pendingChord = null;
	}

	function cancelRecording() {
		recordingId = null;
		conflictInfo = null;
		pendingChord = null;
	}

	function eventToChord(e: KeyboardEvent): string | null {
		const isMac = navigator.platform.toUpperCase().includes('MAC');
		const key = e.key.toLowerCase();
		// Ignore modifier-only presses
		if (['meta', 'control', 'alt', 'shift'].includes(key)) return null;
		const parts: string[] = [];
		if (isMac ? e.metaKey : e.ctrlKey) parts.push('cmd');
		if (isMac && e.ctrlKey && !e.metaKey) parts.push('ctrl');
		if (e.shiftKey) parts.push('shift');
		if (e.altKey) parts.push('alt');
		parts.push(key);
		return parts.join('+');
	}

	function onRecordKeydown(e: KeyboardEvent) {
		if (!recordingId) return;
		if (e.key === 'Escape') { cancelRecording(); return; }
		e.preventDefault();
		e.stopPropagation();
		const chord = eventToChord(e);
		if (!chord) return;

		// Check for conflicts
		const existing = $activeKeybindings.find(kb => kb.key === chord && kb.id !== recordingId);
		if (existing) {
			conflictInfo = { withId: existing.id, withLabel: existing.label };
			pendingChord = chord;
		} else {
			saveChord(recordingId, chord);
		}
	}

	async function saveChord(id: string, chord: string) {
		const overrides = { ...($settings.keybindings ?? {}), [id]: chord };
		// If chord matches default, remove the override
		const def = DEFAULT_KEYBINDINGS.find(kb => kb.id === id);
		if (def && def.defaultKey === chord) delete overrides[id];
		await updateSettings({ keybindings: overrides });
		cancelRecording();
	}

	async function confirmConflict() {
		if (!recordingId || !pendingChord || !conflictInfo) return;
		// Remove the conflicting binding's override (reset it or unset it)
		const overrides = { ...($settings.keybindings ?? {}), [recordingId]: pendingChord };
		// Clear the conflicting one so it gets a new default or empty
		delete overrides[conflictInfo.withId];
		await updateSettings({ keybindings: overrides });
		cancelRecording();
	}

	async function resetBinding(id: string) {
		const overrides = { ...($settings.keybindings ?? {}) };
		delete overrides[id];
		await updateSettings({ keybindings: overrides });
	}

	async function resetAllKeybindings() {
		await updateSettings({ keybindings: {} });
	}

	// Language Servers section state
	let lspRecheckInProgress = $state(false);
	let lspRestartingLang = $state<string | null>(null);

	async function restartServer(lang: string) {
		const root = $projectRoot;
		if (!root || lspRestartingLang === lang) return;
		lspRestartingLang = lang;
		await lspClientManager.stop(lang);
		await lspClientManager.getOrStart(lang, root).catch((e) => log.warn('[SettingsView] getOrStart', e));
		lspRestartingLang = null;
	}

	async function recheckServers() {
		const root = $projectRoot;
		if (!root || lspRecheckInProgress) return;
		lspRecheckInProgress = true;
		dismissedLspNotifications.set(new Set()); // clear dismissed so all recheck
		lspMissingServers.set([]);
		await checkProjectOnOpen(root).catch((e) => log.warn('[SettingsView] checkProjectOnOpen', e));
		lspRecheckInProgress = false;
	}

	const allLspLanguages = $derived(supportedLanguages().map((lang) => ({
		lang,
		config: getServerConfig(lang)!,
		status: $lspStatuses.get(lang) ?? null,
		missing: $lspMissingServers.find((m) => m.language === lang) ?? null,
	})));

	// Apps section state
	let appsView = $state<'installed' | 'store'>('installed');
	let storeSearchQuery = $state('');
	let uninstalling = $state<string | null>(null);
	let toggling = $state<string | null>(null);

	onMount(() => {
		loadCustomThemes();
		tapp.refresh();
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

	// Apps section functions
	async function toggleAppEnabled(app: AppInfo) {
		toggling = app.id;
		try {
			if (app.enabled) {
				await tapp.disable(app.id);
			} else {
				await tapp.enable(app.id);
			}
		} catch (e) {
			console.error('Failed to toggle app:', e);
		}
		toggling = null;
	}

	async function uninstallApp(app: AppInfo) {
		if (!confirm(`Uninstall "${app.name}"? This cannot be undone.`)) return;
		uninstalling = app.id;
		try {
			await tapp.uninstall(app.id);
		} catch (e) {
			console.error('Failed to uninstall app:', e);
		}
		uninstalling = null;
	}

	async function installFromFile() {
		const selected = await open({
			filters: [{ name: 'Manifest', extensions: ['json'] }],
			multiple: false,
		});
		if (selected && typeof selected === 'string') {
			pendingInstall.set({ source: 'file', path: selected });
		}
	}

	async function searchStore() {
		await tapp.searchStore(storeSearchQuery);
	}

	function initiateStoreInstall(listing: AppListing) {
		pendingInstall.set({ source: 'store', listing, appId: listing.id });
	}

	function isInstalled(appId: string): boolean {
		return $installedApps.some(a => a.id === appId);
	}

	function hasUpdate(appId: string): boolean {
		return $availableUpdates.some(u => u.app_id === appId);
	}

	function formatPermission(perm: string): string {
		const names: Record<string, string> = {
			'fs:read': 'Read Files',
			'fs:write': 'Write Files',
			'fs:system': 'System Files',
			'network:fetch': 'Network',
			'network:unrestricted': 'Full Network',
			'storage:session': 'Session Storage',
			'storage:persistent': 'Persistent Storage',
			'agent:inject': 'Inject Context',
			'agent:tools': 'AI Tools',
			'agent:hooks': 'AI Hooks',
			'agent:spawn': 'Spawn Agents',
		};
		return names[perm] || perm;
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="settings-page" onkeydown={onRecordKeydown} role="dialog">
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

		{:else if activeSection === 'workspace'}
			<div class="content-header">
				<h1 class="content-title">Workspace Mode</h1>
				<p class="content-desc">Choose how AI agents interact with your code.</p>
			</div>

			<div class="lsp-format-row" style="margin-bottom: 24px;">
				<div class="toggle-info">
					<span class="toggle-label">Auto Save</span>
					<span class="toggle-desc">Automatically save files after a short delay when changes are made.</span>
				</div>
				<select
					class="select-input"
					value={$settings.autoSave ?? 'off'}
					onchange={(e) => updateSettings({ autoSave: (e.target as HTMLSelectElement).value as 'off' | 'afterDelay' | 'onFocusChange' })}
				>
					<option value="off">Off</option>
					<option value="afterDelay">After Delay (500ms)</option>
				</select>
			</div>

			<div class="mode-cards">
				<button
					class="mode-card"
					class:selected={$settings.workspaceMode === 'dev'}
					onclick={() => updateSettings({ workspaceMode: 'dev' })}
				>
					<div class="mode-card-header">
						<span class="mode-radio">{$settings.workspaceMode === 'dev' ? '◉' : '○'}</span>
						<span class="mode-card-title">Dev Mode</span>
					</div>
					<p class="mode-card-desc">
						The agent edits your files directly in the workspace, just like a pair programmer typing alongside you.
					</p>
					<p class="mode-card-detail">
						You manage git yourself — commit, branch, and push when you're ready.
					</p>
					<span class="mode-card-best">Best for: quick fixes, exploration, pair programming</span>
				</button>

				<button
					class="mode-card"
					class:selected={$settings.workspaceMode === 'agent'}
					onclick={() => updateSettings({ workspaceMode: 'agent' })}
				>
					<div class="mode-card-header">
						<span class="mode-radio">{$settings.workspaceMode === 'agent' ? '◉' : '○'}</span>
						<span class="mode-card-title">Agent Mode</span>
					</div>
					<p class="mode-card-desc">
						Each agent session gets its own isolated branch and worktree. The agent can't touch your main code until you review and approve.
					</p>
					<p class="mode-card-detail">
						Run multiple agents in parallel on different tasks. Review changes file-by-file, then merge or create a PR.
					</p>
					<span class="mode-card-best">Best for: features, refactors, multi-agent workflows</span>
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

		{:else if activeSection === 'apps'}
			<div class="content-header">
				<div class="content-title-row">
					<h1 class="content-title">Apps</h1>
					<div class="apps-view-toggle">
						<button 
							class="view-toggle-btn" 
							class:active={appsView === 'installed'}
							onclick={() => appsView = 'installed'}
						>Installed</button>
						<button 
							class="view-toggle-btn" 
							class:active={appsView === 'store'}
							onclick={() => { appsView = 'store'; tapp.refreshStore(); }}
						>Browse Store</button>
					</div>
				</div>
				<p class="content-desc">
					{#if appsView === 'installed'}
						Manage installed apps. Enable or disable apps, check for updates, or uninstall.
					{:else}
						Browse and install apps from the Tyck app store.
					{/if}
				</p>
			</div>

			{#if appsView === 'installed'}
				<div class="apps-section">
					{#if $installedApps.length === 0}
						<div class="empty-apps">
							<span class="empty-apps-icon">📦</span>
							<span class="empty-apps-text">No apps installed yet</span>
							<span class="empty-apps-hint">Install apps from the store or from a local file</span>
						</div>
					{:else}
						<div class="app-list">
							{#each $installedApps as app (app.id)}
								<div class="app-row" class:disabled={!app.enabled}>
									<div class="app-info">
										<div class="app-header">
											<span class="app-name">{app.name}</span>
											<span class="app-version">v{app.version}</span>
											{#if app.running}
												<span class="running-badge">Running</span>
											{/if}
											{#if hasUpdate(app.id)}
												<span class="update-badge">Update Available</span>
											{/if}
										</div>
										{#if app.description}
											<span class="app-desc">{app.description}</span>
										{/if}
									</div>
									<div class="app-actions">
										<button
											class="toggle-switch"
											class:on={app.enabled}
											onclick={() => toggleAppEnabled(app)}
											disabled={toggling === app.id}
											role="switch"
											aria-checked={app.enabled}
										>
											<span class="toggle-knob"></span>
										</button>
										<button 
											class="uninstall-btn" 
											onclick={() => uninstallApp(app)}
											disabled={uninstalling === app.id}
										>
											{uninstalling === app.id ? '...' : 'Uninstall'}
										</button>
									</div>
								</div>
							{/each}
						</div>
					{/if}

					<div class="install-from-file">
						<button class="install-file-btn" onclick={installFromFile}>
							Install from File...
						</button>
					</div>
				</div>

			{:else}
				<div class="store-section">
					<div class="store-search">
						<input
							type="text"
							class="store-search-input"
							placeholder="Search apps..."
							bind:value={storeSearchQuery}
							onkeydown={(e) => e.key === 'Enter' && searchStore()}
						/>
						<button class="store-search-btn" onclick={searchStore} disabled={$storeLoading}>
							{$storeLoading ? 'Searching...' : 'Search'}
						</button>
					</div>

					{#if $storeListings.length === 0}
						<div class="empty-apps">
							<span class="empty-apps-icon">🔍</span>
							<span class="empty-apps-text">No apps found</span>
							<span class="empty-apps-hint">Try searching for something or refresh the store</span>
						</div>
					{:else}
						<div class="store-grid">
							{#each $storeListings as listing (listing.id)}
								<div class="store-card">
									<div class="store-card-header">
										<span class="store-card-name">{listing.name}</span>
										<span class="store-card-version">v{listing.version}</span>
									</div>
									<p class="store-card-desc">{listing.description}</p>
									<div class="store-card-meta">
										<span class="store-card-author">by {listing.author}</span>
										{#if listing.downloads > 0}
											<span class="store-card-downloads">{listing.downloads.toLocaleString()} downloads</span>
										{/if}
									</div>
									<div class="store-card-permissions">
										{#each listing.permissions.slice(0, 3) as perm}
											<span class="permission-badge">{formatPermission(perm)}</span>
										{/each}
										{#if listing.permissions.length > 3}
											<span class="permission-badge more">+{listing.permissions.length - 3}</span>
										{/if}
									</div>
									<div class="store-card-actions">
										{#if isInstalled(listing.id)}
											<span class="installed-badge">Installed</span>
										{:else}
											<button class="install-btn" onclick={() => initiateStoreInstall(listing)}>
												Install
											</button>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/if}
		{:else if activeSection === 'language-servers'}
			<div class="content-header">
				<div class="content-title-row">
					<h1 class="content-title">Language Servers</h1>
					<button class="refresh-btn" onclick={recheckServers} disabled={lspRecheckInProgress}>
						{lspRecheckInProgress ? 'Checking...' : 'Recheck'}
					</button>
				</div>
				<p class="content-desc">
					LSP servers provide completions, diagnostics, go-to-definition, and more. Install any
					missing servers to enable full IDE features for that language.
				</p>
			</div>

			<div class="lsp-format-row">
				<div class="toggle-info">
					<span class="toggle-label">Format on save</span>
					<span class="toggle-desc">Automatically format the file with its language server when you save.</span>
				</div>
				<button
					class="toggle-switch"
					class:on={$settings.lspFormatOnSave}
					onclick={() => updateSettings({ lspFormatOnSave: !$settings.lspFormatOnSave })}
					role="switch"
					aria-checked={!!$settings.lspFormatOnSave}
				>
					<span class="toggle-knob"></span>
				</button>
			</div>

			<div class="section-label" style="margin-top: 24px; margin-bottom: 12px;">Installed / Available</div>
			<div class="lsp-server-list">
				{#each allLspLanguages as { lang, config, status, missing } (lang)}
					<div class="lsp-server-row">
						<div class="lsp-server-info">
							<div class="lsp-server-header">
								<span class="lsp-server-name">{config.displayName}</span>
								{#if status?.state === 'running'}
									<span class="lsp-badge running">Running</span>
								{:else if status?.state === 'starting'}
									<span class="lsp-badge starting">Starting</span>
								{:else if status?.state === 'error'}
									<span class="lsp-badge error" title={status.error ?? ''}>Error</span>
								{:else if status?.state === 'not-installed' || missing}
									<span class="lsp-badge missing">Not installed</span>
								{:else}
									<span class="lsp-badge idle">Idle</span>
								{/if}
							</div>
							{#if status?.state === 'error' && status.error}
								<code class="lsp-error-detail">{status.error}</code>
							{/if}
							{#if missing}
								<code class="lsp-install-hint">{missing.installHint}</code>
							{/if}
						</div>
						{#if status?.state !== 'not-installed' && !missing}
							<button
								class="lsp-restart-btn"
								onclick={() => restartServer(lang)}
								disabled={lspRestartingLang === lang || status?.state === 'starting'}
								title={status?.state === 'running' ? 'Restart server' : status?.state === 'error' ? 'Retry' : 'Start server'}
							>
								{lspRestartingLang === lang ? '...' : status?.state === 'running' ? 'Restart' : status?.state === 'error' ? 'Retry' : 'Start'}
							</button>
						{/if}
					</div>
				{/each}
			</div>
		{:else if activeSection === 'keybindings'}
			<div class="content-header">
				<div class="content-title-row">
					<h1 class="content-title">Keyboard Shortcuts</h1>
					<button class="refresh-btn" onclick={resetAllKeybindings}>Reset All</button>
				</div>
				<p class="content-desc">Click a shortcut to reassign it. Press Escape to cancel.</p>
			</div>

			<div class="kb-search-row">
				<input
					class="kb-search"
					bind:value={kbSearchQuery}
					placeholder="Search shortcuts..."
				/>
			</div>

			{#each [...groupByCategory(filteredBindings)] as [category, bindings]}
				<div class="kb-category-label">{category}</div>
				{#each bindings as kb}
					{@const isCustom = ($settings.keybindings ?? {})[kb.id] !== undefined}
					{@const isRecording = recordingId === kb.id}
					<div class="kb-row" class:recording={isRecording}>
						<span class="kb-action-label">{kb.label}</span>
						<div class="kb-chord-area">
							{#if isRecording}
								{#if conflictInfo && pendingChord}
									<span class="kb-conflict-msg">
										⚠ Already used by <strong>{conflictInfo.withLabel}</strong>
									</span>
									<button class="kb-btn-reassign" onclick={confirmConflict}>Reassign</button>
									<button class="kb-btn-cancel" onclick={cancelRecording}>Cancel</button>
								{:else}
									<span class="kb-recording-hint">Press shortcut…</span>
									<button class="kb-btn-cancel" onclick={cancelRecording}>Cancel</button>
								{/if}
							{:else}
								<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
								<span
									class="kb-chord"
									class:customised={isCustom}
									onclick={() => startRecording(kb.id)}
									title="Click to reassign"
								>{formatChord(kb.key)}</span>
								{#if isCustom}
									<button class="kb-btn-reset" onclick={() => resetBinding(kb.id)} title="Reset to default">↺</button>
								{/if}
							{/if}
						</div>
					</div>
				{/each}
			{/each}
		{/if}
	</main>
</div>

<style>
	.settings-page {
		position: fixed;
		inset: 0;
		z-index: 100;
		display: flex;
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
	.mode-cards {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 0 16px;
	}
	.mode-card {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 16px 20px;
		background: var(--color-surface);
		border: 2px solid var(--color-border-muted);
		border-radius: 10px;
		cursor: pointer;
		text-align: left;
		transition: border-color 0.15s, background 0.15s;
	}
	.mode-card:hover {
		border-color: var(--color-border);
	}
	.mode-card.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 5%, var(--color-surface));
	}
	.mode-card-header {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.mode-radio {
		font-size: 16px;
		color: var(--color-accent);
	}
	.mode-card-title {
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
	}
	.mode-card-desc {
		font-size: 13px;
		color: var(--color-text-secondary);
		line-height: 1.5;
	}
	.mode-card-detail {
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.5;
	}
	.mode-card-best {
		font-size: 11px;
		color: var(--color-accent);
		font-weight: 500;
		margin-top: 4px;
	}

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

	/* ── Apps Section ── */
	.apps-view-toggle {
		display: flex;
		gap: 4px;
		background: var(--color-surface);
		padding: 3px;
		border-radius: 8px;
		border: 1px solid var(--color-border-muted);
	}
	.view-toggle-btn {
		padding: 6px 14px;
		font-size: 12px;
		font-weight: 500;
		color: var(--color-text-muted);
		background: none;
		border: none;
		border-radius: 5px;
		cursor: pointer;
		transition: all 0.15s;
	}
	.view-toggle-btn:hover {
		color: var(--color-text);
	}
	.view-toggle-btn.active {
		background: var(--color-base);
		color: var(--color-text);
		font-weight: 600;
	}

	.apps-section, .store-section {
		max-width: 700px;
	}

	.empty-apps {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 48px 24px;
		background: var(--color-surface);
		border: 1px dashed var(--color-border-muted);
		border-radius: 12px;
		text-align: center;
	}
	.empty-apps-icon {
		font-size: 32px;
		margin-bottom: 12px;
	}
	.empty-apps-text {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
		margin-bottom: 4px;
	}
	.empty-apps-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.app-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.app-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 14px 16px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		transition: opacity 0.15s;
	}
	.app-row.disabled {
		opacity: 0.5;
	}
	.app-info {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
		flex: 1;
	}
	.app-header {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.app-name {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}
	.app-version {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-family: 'SF Mono', 'Fira Code', monospace;
	}
	.app-desc {
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.4;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.running-badge {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 15%, transparent);
		padding: 2px 6px;
		border-radius: 4px;
	}
	.update-badge {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-warning);
		background: color-mix(in srgb, var(--color-warning) 15%, transparent);
		padding: 2px 6px;
		border-radius: 4px;
	}
	.app-actions {
		display: flex;
		align-items: center;
		gap: 12px;
		flex-shrink: 0;
		margin-left: 16px;
	}
	.uninstall-btn {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-error);
		background: none;
		border: 1px solid color-mix(in srgb, var(--color-error) 25%, transparent);
		border-radius: 6px;
		padding: 5px 12px;
		cursor: pointer;
	}
	.uninstall-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-error) 10%, transparent);
		border-color: var(--color-error);
	}
	.uninstall-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.install-from-file {
		margin-top: 16px;
	}
	.install-file-btn {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-accent);
		background: none;
		border: 1px dashed color-mix(in srgb, var(--color-accent) 40%, transparent);
		border-radius: 8px;
		padding: 12px 20px;
		cursor: pointer;
		width: 100%;
	}
	.install-file-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		border-color: var(--color-accent);
	}

	/* ── Store Section ── */
	.store-search {
		display: flex;
		gap: 8px;
		margin-bottom: 20px;
	}
	.store-search-input {
		flex: 1;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		padding: 10px 14px;
		color: var(--color-text);
		font-size: 13px;
		outline: none;
	}
	.store-search-input:focus {
		border-color: var(--color-accent);
	}
	.store-search-input::placeholder {
		color: var(--color-text-subtle);
	}
	.store-search-btn {
		background: var(--color-accent);
		border: none;
		border-radius: 8px;
		color: var(--color-base);
		font-size: 12px;
		font-weight: 600;
		padding: 10px 20px;
		cursor: pointer;
		flex-shrink: 0;
	}
	.store-search-btn:hover:not(:disabled) {
		filter: brightness(1.1);
	}
	.store-search-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.store-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: 12px;
	}
	.store-card {
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 10px;
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.store-card-header {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.store-card-name {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}
	.store-card-version {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-family: 'SF Mono', 'Fira Code', monospace;
	}
	.store-card-desc {
		font-size: 12px;
		color: var(--color-text-muted);
		line-height: 1.4;
		margin: 0;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	.store-card-meta {
		display: flex;
		gap: 12px;
		font-size: 11px;
		color: var(--color-text-subtle);
	}
	.store-card-permissions {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.permission-badge {
		font-size: 10px;
		font-weight: 500;
		color: var(--color-text-muted);
		background: var(--color-overlay);
		padding: 2px 6px;
		border-radius: 4px;
	}
	.permission-badge.more {
		color: var(--color-text-subtle);
	}
	.store-card-actions {
		margin-top: auto;
		padding-top: 8px;
	}
	.install-btn {
		width: 100%;
		background: var(--color-accent);
		border: none;
		border-radius: 6px;
		color: var(--color-base);
		font-size: 12px;
		font-weight: 600;
		padding: 8px 16px;
		cursor: pointer;
	}
	.install-btn:hover {
		filter: brightness(1.1);
	}
	.installed-badge {
		display: block;
		text-align: center;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-success) 25%, transparent);
		padding: 6px 12px;
		border-radius: 6px;
	}

	.select-input {
		padding: 5px 10px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 12px;
		cursor: pointer;
		outline: none;
	}
	.select-input:focus {
		border-color: var(--color-accent);
	}

	/* ── Language Servers ── */
	.lsp-format-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
		max-width: 560px;
	}

	.lsp-server-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-width: 560px;
	}

	.lsp-server-row {
		display: flex;
		align-items: center;
		padding: 12px 16px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 8px;
	}

	.lsp-server-info {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.lsp-server-header {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.lsp-server-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
	}

	.lsp-badge {
		font-size: 10px;
		font-weight: 700;
		padding: 2px 7px;
		border-radius: 4px;
	}

	.lsp-badge.running {
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
	}

	.lsp-badge.starting {
		color: var(--color-warning);
		background: color-mix(in srgb, var(--color-warning) 12%, transparent);
	}

	.lsp-badge.error {
		color: var(--color-error);
		background: color-mix(in srgb, var(--color-error) 12%, transparent);
	}

	.lsp-badge.missing {
		color: var(--color-error);
		background: color-mix(in srgb, var(--color-error) 8%, transparent);
	}

	.lsp-badge.idle {
		color: var(--color-text-subtle);
		background: var(--color-overlay);
	}

	.lsp-error-detail {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 10px;
		color: var(--color-error);
		word-break: break-all;
	}

	.lsp-install-hint {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 10px;
		color: var(--color-text-subtle);
	}

	.lsp-restart-btn {
		flex-shrink: 0;
		background: var(--color-overlay);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		color: var(--color-text-subtle);
		font-size: 11px;
		font-weight: 600;
		padding: 3px 10px;
		cursor: pointer;
		margin-left: 12px;
	}

	.lsp-restart-btn:hover:not(:disabled) {
		background: var(--color-surface);
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.lsp-restart-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* ── Keyboard Shortcuts ── */
	.kb-search-row {
		padding: 0 0 16px;
	}
	.kb-search {
		width: 100%;
		padding: 7px 10px;
		background: var(--color-base);
		border: 1px solid var(--color-border-muted);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		outline: none;
		font-family: inherit;
	}
	.kb-search:focus { border-color: var(--color-accent); }
	.kb-category-label {
		font-size: 10px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.6px;
		color: var(--color-text-subtle);
		padding: 12px 0 6px;
		border-bottom: 1px solid var(--color-border-muted);
		margin-bottom: 2px;
	}
	.kb-row {
		display: flex;
		align-items: center;
		padding: 6px 4px;
		border-radius: 5px;
		gap: 12px;
	}
	.kb-row:hover { background: var(--color-base); }
	.kb-row.recording { background: color-mix(in srgb, var(--color-accent) 6%, transparent); }
	.kb-action-label {
		flex: 1;
		font-size: 13px;
		color: var(--color-text);
	}
	.kb-chord-area {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.kb-chord {
		font-size: 12px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		background: var(--color-surface);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		padding: 2px 8px;
		color: var(--color-text-muted);
		cursor: pointer;
		user-select: none;
		white-space: nowrap;
	}
	.kb-chord:hover { border-color: var(--color-accent); color: var(--color-text); }
	.kb-chord.customised { border-color: var(--color-accent); color: var(--color-accent); }
	.kb-recording-hint {
		font-size: 12px;
		color: var(--color-accent);
		font-style: italic;
		padding: 2px 8px;
		border: 1px dashed var(--color-accent);
		border-radius: 4px;
	}
	.kb-conflict-msg {
		font-size: 12px;
		color: var(--color-warning);
	}
	.kb-btn-reassign {
		font-size: 11px;
		padding: 2px 8px;
		background: var(--color-warning);
		color: var(--color-base);
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-weight: 600;
	}
	.kb-btn-cancel, .kb-btn-reset {
		font-size: 11px;
		padding: 2px 8px;
		background: none;
		color: var(--color-text-subtle);
		border: 1px solid var(--color-border-muted);
		border-radius: 4px;
		cursor: pointer;
	}
	.kb-btn-cancel:hover, .kb-btn-reset:hover { color: var(--color-text); border-color: var(--color-border); }
</style>
