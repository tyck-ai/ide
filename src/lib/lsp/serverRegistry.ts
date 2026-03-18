import type { LanguageClientOptions } from 'vscode-languageclient/browser';

// ─── Types ───────────────────────────────────────────────────────────────────

export interface ServerConfig {
	/** Human-readable name shown in status bar / settings UI */
	displayName: string;

	/**
	 * Monaco language ids this server handles.
	 * Used for the LSP document selector AND for lsp_check_binary language key.
	 */
	documentSelector: LanguageClientOptions['documentSelector'];

	/**
	 * Passed to the server in the LSP `initialize` request.
	 * Language-specific options (rubocop config, tsconfig preferences, etc.)
	 */
	initializationOptions?: Record<string, unknown>;

	/**
	 * Workspace configuration sent via `workspace/didChangeConfiguration`.
	 * Applied after initialization.
	 */
	settings?: Record<string, unknown>;
}

// ─── Registry ────────────────────────────────────────────────────────────────

const REGISTRY: Record<string, ServerConfig> = {
	// ─── TypeScript / JavaScript ─────────────────────────────────────────────
	typescript: {
		displayName: 'TypeScript Language Server',
		documentSelector: [
			{ language: 'typescript' },
			{ language: 'javascript' },
			{ language: 'typescriptreact' },
			{ language: 'javascriptreact' },
		],
		initializationOptions: {
			preferences: {
				importModuleSpecifierPreference: 'relative',
				includeInlayParameterNameHints: 'all',
				includeInlayVariableTypeHints: true,
				includeInlayFunctionLikeReturnTypeHints: true,
			},
		},
	},

	// ─── Rust ─────────────────────────────────────────────────────────────────
	rust: {
		displayName: 'rust-analyzer',
		documentSelector: [{ language: 'rust' }],
		initializationOptions: {
			checkOnSave: { command: 'clippy' },
			inlayHints: {
				parameterHints: { enable: true },
				typeHints: { enable: true },
				chainingHints: { enable: true },
			},
			lens: {
				enable: true,
				references: { enable: true },
				implementations: { enable: true },
			},
		},
	},

	// ─── Python ───────────────────────────────────────────────────────────────
	python: {
		displayName: 'Pyright',
		documentSelector: [{ language: 'python' }],
		settings: {
			python: {
				analysis: {
					typeCheckingMode: 'basic',
					autoImportCompletions: true,
					inlayHints: {
						variableTypes: true,
						functionReturnTypes: true,
						parameterTypes: true,
					},
				},
			},
		},
	},

	// ─── Go ───────────────────────────────────────────────────────────────────
	go: {
		displayName: 'gopls',
		documentSelector: [{ language: 'go' }],
		settings: {
			gopls: {
				hints: {
					assignVariableTypes: true,
					compositeLiteralFields: true,
					constantValues: true,
					functionTypeParameters: true,
					parameterNames: true,
					rangeVariableTypes: true,
				},
				analyses: {
					unusedparams: true,
					shadow: true,
				},
				staticcheck: true,
			},
		},
	},

	// ─── Ruby ─────────────────────────────────────────────────────────────────
	ruby: {
		displayName: 'ruby-lsp',
		documentSelector: [{ language: 'ruby' }],
		initializationOptions: {
			enabledFeatures: {
				diagnostics: true,
				formatting: true,
				inlayHint: true,
				completion: true,
				hover: true,
				signatureHelp: true,
				codeActions: true,
				codeLens: true,
				definition: true,
				references: true,
				rename: true,
				semanticHighlighting: true,
			},
			formatter: 'auto',
			linters: ['rubocop'],
		},
	},

	// ─── Svelte ───────────────────────────────────────────────────────────────
	svelte: {
		displayName: 'Svelte Language Server',
		documentSelector: [{ language: 'svelte' }],
		initializationOptions: {
			configuration: {
				svelte: { plugin: { typescript: { enable: true } } },
			},
		},
	},

	// ─── CSS / SCSS / Less ────────────────────────────────────────────────────
	css: {
		displayName: 'CSS Language Server',
		documentSelector: [
			{ language: 'css' },
			{ language: 'scss' },
			{ language: 'less' },
		],
		settings: {
			css: { validate: true, hover: { documentation: true, references: true } },
			scss: { validate: true },
			less: { validate: true },
		},
	},

	// ─── HTML ─────────────────────────────────────────────────────────────────
	html: {
		displayName: 'HTML Language Server',
		documentSelector: [{ language: 'html' }],
		settings: {
			html: {
				format: { enable: true },
				hover: { documentation: true, references: true },
			},
		},
	},

	// ─── JSON ─────────────────────────────────────────────────────────────────
	json: {
		displayName: 'JSON Language Server',
		documentSelector: [{ language: 'json' }, { language: 'jsonc' }],
		initializationOptions: { provideFormatter: true },
		settings: {
			json: {
				validate: { enable: true },
				format: { enable: true },
				schemas: [
					{
						fileMatch: ['package.json'],
						url: 'https://json.schemastore.org/package.json',
					},
					{
						fileMatch: ['tsconfig.json', 'tsconfig.*.json'],
						url: 'https://json.schemastore.org/tsconfig.json',
					},
				],
			},
		},
	},

	// ─── YAML ─────────────────────────────────────────────────────────────────
	yaml: {
		displayName: 'YAML Language Server',
		documentSelector: [{ language: 'yaml' }],
		settings: {
			yaml: {
				validate: true,
				hover: true,
				completion: true,
				schemas: {
					'https://json.schemastore.org/github-workflow.json':
						'.github/workflows/*.yml',
					'https://raw.githubusercontent.com/compose-spec/compose-spec/master/schema/compose-spec.json':
						'docker-compose*.yml',
				},
			},
		},
	},

	// ─── TOML ─────────────────────────────────────────────────────────────────
	toml: {
		displayName: 'Taplo TOML',
		documentSelector: [{ language: 'toml' }],
		settings: {
			evenBetterToml: { schema: { enabled: true, links: true } },
		},
	},

	// ─── GraphQL ──────────────────────────────────────────────────────────────
	graphql: {
		displayName: 'GraphQL Language Server',
		documentSelector: [
			{ language: 'graphql' },
			{ pattern: '**/*.graphql' },
			{ pattern: '**/*.gql' },
		],
	},

	// ─── Elixir ───────────────────────────────────────────────────────────────
	elixir: {
		displayName: 'ElixirLS',
		documentSelector: [{ language: 'elixir' }],
		settings: {
			elixirLS: {
				dialyzerEnabled: true,
				fetchDeps: true,
				suggestSpecs: true,
			},
		},
	},
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

/** Language ids that share a server with another canonical language */
const ALIASES: Record<string, string> = {
	javascript: 'typescript',
	typescriptreact: 'typescript',
	javascriptreact: 'typescript',
	scss: 'css',
	less: 'css',
	jsonc: 'json',
};

/**
 * Normalise a Monaco language id to the canonical key used in the registry.
 * e.g. 'javascriptreact' → 'typescript', 'scss' → 'css'
 */
export function normalizeLanguage(language: string): string {
	return ALIASES[language] ?? language;
}

/** Look up the server config for a Monaco language id. Returns null if unsupported. */
export function getServerConfig(language: string): ServerConfig | null {
	return REGISTRY[normalizeLanguage(language)] ?? null;
}

/** All canonical language keys that have a server configured. */
export function supportedLanguages(): string[] {
	return Object.keys(REGISTRY);
}
