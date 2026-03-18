import { MonacoLanguageClient } from 'monaco-languageclient';
import { CloseAction, ErrorAction, State } from 'vscode-languageclient/browser';
import { writable, get } from 'svelte/store';
import { createTauriTransport, type LspTransport } from './TauriTransport';
import { getServerConfig, normalizeLanguage } from './serverRegistry';

// ─── Status store (read by StatusBar, settings panel, etc.) ──────────────────

export interface LspStatus {
	language: string;
	displayName: string;
	state: 'starting' | 'running' | 'stopped' | 'error';
}

export const lspStatuses = writable<Map<string, LspStatus>>(new Map());

function setStatus(language: string, displayName: string, state: LspStatus['state']) {
	lspStatuses.update((map) => {
		map.set(language, { language, displayName, state });
		return map;
	});
}

function removeStatus(language: string) {
	lspStatuses.update((map) => {
		map.delete(language);
		return map;
	});
}

// ─── Manager ─────────────────────────────────────────────────────────────────

interface ManagedClient {
	client: MonacoLanguageClient;
	transport: LspTransport;
}

class LspClientManager {
	private clients = new Map<string, ManagedClient>();
	// Tracks in-progress starts to prevent double-spawning
	private starting = new Map<string, Promise<ManagedClient | null>>();

	/**
	 * Get the running client for a language, starting it if not already running.
	 * Returns null if the language has no configured server.
	 */
	async getOrStart(language: string, workspaceRoot: string): Promise<MonacoLanguageClient | null> {
		const canonical = normalizeLanguage(language);
		const config = getServerConfig(canonical);
		if (!config) return null;

		// Return existing healthy client
		const existing = this.clients.get(canonical);
		if (existing && existing.client.state !== State.Stopped) {
			return existing.client;
		}

		// If a start is already in progress, wait for it
		const inProgress = this.starting.get(canonical);
		if (inProgress) {
			const managed = await inProgress;
			return managed?.client ?? null;
		}

		// Start a new client
		const startPromise = this.startClient(canonical, workspaceRoot);
		this.starting.set(canonical, startPromise);

		try {
			const managed = await startPromise;
			if (managed) {
				this.clients.set(canonical, managed);
			}
			return managed?.client ?? null;
		} catch (e) {
			console.error(`[lsp] Failed to start client for '${canonical}':`, e);
			setStatus(canonical, config.displayName, 'error');
			return null;
		} finally {
			this.starting.delete(canonical);
		}
	}

	private async startClient(
		language: string,
		workspaceRoot: string,
	): Promise<ManagedClient | null> {
		const config = getServerConfig(language);
		if (!config) return null;

		setStatus(language, config.displayName, 'starting');

		let transport: LspTransport;
		try {
			transport = await createTauriTransport(language, workspaceRoot);
		} catch (e) {
			console.error(`[lsp] Transport creation failed for '${language}':`, e);
			setStatus(language, config.displayName, 'error');
			return null;
		}

		const client = new MonacoLanguageClient({
			name: config.displayName,
			clientOptions: {
				documentSelector: config.documentSelector,
				initializationOptions: config.initializationOptions,
				errorHandler: {
					error: () => ({ action: ErrorAction.Continue }),
					closed: () => ({ action: CloseAction.DoNotRestart }),
				},
			},
			messageTransports: {
				reader: transport.reader,
				writer: transport.writer,
			},
		});

		client.onDidChangeState(({ newState }) => {
			const state: LspStatus['state'] =
				newState === State.Running
					? 'running'
					: newState === State.Starting
						? 'starting'
						: 'stopped';
			setStatus(language, config.displayName, state);

			// Send workspace/didChangeConfiguration once the server is ready
			if (newState === State.Running && config.settings) {
				client
					.sendNotification('workspace/didChangeConfiguration', {
						settings: config.settings,
					})
					.catch(() => {});
			}
		});

		await client.start();
		return { client, transport };
	}

	/** Get the active client for a language id (no side effects). */
	getActiveClient(language: string): MonacoLanguageClient | null {
		const canonical = normalizeLanguage(language);
		const managed = this.clients.get(canonical);
		if (!managed || managed.client.state === State.Stopped) return null;
		return managed.client;
	}

	/** Stop a specific language client. */
	async stop(language: string): Promise<void> {
		const canonical = normalizeLanguage(language);
		const managed = this.clients.get(canonical);
		if (!managed) return;

		this.clients.delete(canonical);
		await managed.client.stop().catch(() => {});
		await managed.transport.dispose();
		removeStatus(canonical);
	}

	/** Stop all running clients. Called on project close / window teardown. */
	async stopAll(): Promise<void> {
		const stops = [...this.clients.entries()].map(async ([lang, managed]) => {
			await managed.client.stop().catch(() => {});
			await managed.transport.dispose();
			removeStatus(lang);
		});
		this.clients.clear();
		await Promise.all(stops);
	}

	/** Returns the status of all managed clients for the current file's language. */
	getStatusForLanguage(language: string): LspStatus | null {
		const canonical = normalizeLanguage(language);
		return get(lspStatuses).get(canonical) ?? null;
	}
}

// Singleton — one manager per app instance
export const lspClientManager = new LspClientManager();
