import { invoke } from '@tauri-apps/api/core';
import { Channel } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
	AbstractMessageReader,
	AbstractMessageWriter,
	type DataCallback,
	type Message,
	type Disposable,
} from 'vscode-languageserver-protocol/browser';

// ─── Reader ──────────────────────────────────────────────────────────────────

/**
 * Receives LSP JSON-RPC messages from the Rust backend via a Tauri Channel.
 * One instance per language server process.
 */
export class TauriMessageReader extends AbstractMessageReader {
	private channel: Channel<string>;
	private callback: DataCallback | null = null;
	private closeUnlisten: (() => void) | null = null;

	constructor(channel: Channel<string>, serverId: string) {
		super();
		this.channel = channel;

		// Route channel messages to the LSP callback
		this.channel.onmessage = (rawJson: string) => {
			if (this.callback) {
				try {
					const message = JSON.parse(rawJson) as Message;
					this.callback(message);
				} catch (e) {
					this.fireError(new Error(`Failed to parse LSP message: ${e}`));
				}
			}
		};

		// Detect server process exit (emitted by Rust when the stdout loop ends)
		listen<void>(`lsp-server-closed-${serverId}`, () => {
			this.fireClose();
		}).then((unlisten) => {
			this.closeUnlisten = unlisten;
		}).catch((e) => {
			console.warn(`[lsp] Failed to register close listener for server ${serverId}:`, e);
		});
	}

	listen(callback: DataCallback): Disposable {
		this.callback = callback;
		return {
			dispose: () => {
				this.callback = null;
			},
		};
	}

	dispose() {
		this.callback = null;
		this.closeUnlisten?.();
		this.closeUnlisten = null;
		super.dispose();
	}
}

// ─── Writer ──────────────────────────────────────────────────────────────────

/**
 * Sends LSP JSON-RPC messages to the Rust backend via `lsp_send` invoke.
 * The backend writes them to the language server's stdin with Content-Length framing.
 */
export class TauriMessageWriter extends AbstractMessageWriter {
	constructor(private readonly serverId: string) {
		super();
	}

	async write(message: Message): Promise<void> {
		const json = JSON.stringify(message);
		try {
			await invoke('lsp_send', { serverId: this.serverId, message: json });
		} catch (e) {
			this.fireError(new Error(`lsp_send failed: ${e}`), message, 0);
			throw e;
		}
	}

	end(): void {
		// Nothing to flush — each message is written synchronously in Rust
	}

	dispose() {
		super.dispose();
	}
}

// ─── Factory ─────────────────────────────────────────────────────────────────

export interface LspTransport {
	reader: TauriMessageReader;
	writer: TauriMessageWriter;
	serverId: string;
	dispose: () => Promise<void>;
}

/**
 * Starts a language server via Tauri and returns a ready-to-use transport pair.
 *
 * @param language    Monaco language id (e.g. 'rust', 'typescript', 'ruby')
 * @param workspaceRoot  Absolute path to the project root
 */
export async function createTauriTransport(
	language: string,
	workspaceRoot: string,
): Promise<LspTransport> {
	const channel = new Channel<string>();

	const serverId = await invoke<string>('lsp_start', {
		language,
		workspaceRoot,
		onMessage: channel,
	});

	const reader = new TauriMessageReader(channel, serverId);
	const writer = new TauriMessageWriter(serverId);

	return {
		reader,
		writer,
		serverId,
		dispose: async () => {
			reader.dispose();
			writer.dispose();
			await invoke('lsp_stop', { serverId }).catch(() => {});
		},
	};
}
