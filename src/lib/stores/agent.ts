import { writable, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { isDevMode } from './settings';
import { addEdit } from './devModeEdits';

export type AgentBlockType = 'text' | 'tool_use' | 'tool_result' | 'thinking';

export interface AgentBlock {
	type: AgentBlockType;
	content: string;
	toolName?: string;
	toolInput?: string;
	toolId?: string;
	isStreaming: boolean;
}

export interface AgentTurn {
	role: 'assistant' | 'user';
	blocks: AgentBlock[];
}

export interface PendingEdit {
	filePath: string;
	oldContent: string;
	newContent: string;
	toolId: string;
	status: 'pending' | 'accepted' | 'rejected';
}

export interface AgentSession {
	id: string;
	turns: AgentTurn[];
	isRunning: boolean;
	error?: string;
}

export const agentSession = writable<AgentSession | null>(null);
export const pendingEdits = writable<PendingEdit[]>([]);
/** The file the agent is currently editing (set during tool calls, cleared after) */
export const agentActiveFile = writable<string | null>(null);

let eventUnlisten: (() => void) | null = null;

/**
 * Claude Code stream-json --verbose format:
 *   {"type":"system","subtype":"init",...}
 *   {"type":"assistant","message":{"content":[{"type":"text","text":"..."},{"type":"tool_use","name":"Edit","input":{...}}],...}}
 *   {"type":"tool","tool":{"name":"Read","result":"..."},...}
 *   {"type":"assistant","message":{"content":[...],...}}
 *   {"type":"result","subtype":"success","result":"...","is_error":false,...}
 */
function processEvent(data: any) {
	agentSession.update(session => {
		if (!session) return session;

		switch (data.type) {
			case 'assistant': {
				const msg = data.message;
				if (!msg?.content) break;

				const blocks: AgentBlock[] = [];
				for (const block of msg.content) {
					if (block.type === 'text') {
						blocks.push({
							type: 'text',
							content: block.text || '',
							isStreaming: false,
						});
					} else if (block.type === 'thinking') {
						blocks.push({
							type: 'thinking',
							content: block.thinking || '',
							isStreaming: false,
						});
					} else if (block.type === 'tool_use') {
						const inputStr = typeof block.input === 'string'
							? block.input
							: JSON.stringify(block.input, null, 2);
						blocks.push({
							type: 'tool_use',
							content: '',
							toolName: block.name,
							toolId: block.id,
							toolInput: inputStr,
							isStreaming: false,
						});

						// Extract pending edits from Edit/Write tool calls
						if (block.name === 'Edit' || block.name === 'Write') {
							const input = typeof block.input === 'string'
								? (() => { try { return JSON.parse(block.input); } catch { return null; } })()
								: block.input;
							if (input) {
								// Track which file the agent is actively editing
								agentActiveFile.set(input.file_path || null);
								if (get(isDevMode)) {
									// Dev mode: route to inline review system
									addEdit({
										id: block.id || crypto.randomUUID(),
										filePath: input.file_path || '',
										oldContent: block.name === 'Edit' ? (input.old_string || null) : null,
										newContent: input.new_string || input.content || '',
									});
								} else {
									// Agent mode: track as pending edits (worktree-based review)
									pendingEdits.update(edits => [...edits, {
										filePath: input.file_path || '',
										oldContent: input.old_string || '',
										newContent: input.new_string || input.content || '',
										toolId: block.id || '',
										status: 'pending' as const,
									}]);
								}
							}
						}
					}
				}

				if (blocks.length > 0) {
					session.turns = [...session.turns, { role: 'assistant', blocks }];
				}
				break;
			}

			case 'tool': {
				// Tool result — agent finished editing the file
				agentActiveFile.set(null);

				const tool = data.tool;
				if (!tool) break;

				const resultContent = typeof tool.result === 'string'
					? tool.result
					: JSON.stringify(tool.result, null, 2);

				const block: AgentBlock = {
					type: 'tool_result',
					content: resultContent || '(no output)',
					toolName: tool.name,
					isStreaming: false,
				};

				// Append to last assistant turn or create new one
				const lastTurn = session.turns[session.turns.length - 1];
				if (lastTurn && lastTurn.role === 'assistant') {
					lastTurn.blocks = [...lastTurn.blocks, block];
					session.turns = [...session.turns];
				} else {
					session.turns = [...session.turns, { role: 'assistant', blocks: [block] }];
				}
				break;
			}

			case 'result': {
				agentActiveFile.set(null);
				// Final result — add text if no assistant content was shown yet
				if (data.result && typeof data.result === 'string' && data.result.trim()) {
					const hasAssistantText = session.turns.some(
						t => t.role === 'assistant' && t.blocks.some(b => b.type === 'text' && b.content)
					);
					if (!hasAssistantText) {
						session.turns = [...session.turns, {
							role: 'assistant',
							blocks: [{ type: 'text', content: data.result, isStreaming: false }],
						}];
					}
				}
				if (data.is_error) {
					session.error = typeof data.result === 'string' ? data.result : 'Agent error';
				}
				session.isRunning = false;
				break;
			}

			case 'done': {
				session.isRunning = false;
				if (data.error) {
					session.error = data.error;
				}
				break;
			}

			// Ignore: system, rate_limit_event, etc.
		}

		return { ...session };
	});
}

export async function initAgentListener() {
	if (eventUnlisten) return;
	eventUnlisten = await listen<{ session_id: string; data: any }>('agent-event', (event) => {
		processEvent(event.payload.data);
	});
}

export async function startAgent(
	prompt: string,
	context?: { filePath?: string; selection?: string; cursorLine?: number },
	cwd?: string,
) {
	const sessionId = crypto.randomUUID();

	let fullPrompt = prompt;
	if (context?.filePath) {
		fullPrompt = `[Context: Active file: ${context.filePath}`;
		if (context.cursorLine !== undefined) fullPrompt += `, cursor at line ${context.cursorLine}`;
		if (context.selection) fullPrompt += `, selected text:\n${context.selection}`;
		fullPrompt += `]\n\n${prompt}`;
	}

	const session: AgentSession = {
		id: sessionId,
		turns: [{ role: 'user', blocks: [{ type: 'text', content: prompt, isStreaming: false }] }],
		isRunning: true,
	};

	agentSession.set(session);
	pendingEdits.set([]);

	await initAgentListener();

	try {
		await invoke('start_agent', { sessionId, prompt: fullPrompt, cwd: cwd || null });
	} catch (e) {
		agentSession.update(s => s ? { ...s, isRunning: false, error: String(e) } : s);
	}
}

export async function stopAgent() {
	const session = get(agentSession);
	if (session) {
		try {
			await invoke('stop_agent', { sessionId: session.id });
		} catch { /* ignore */ }
		agentSession.update(s => s ? { ...s, isRunning: false } : s);
	}
}

export function clearSession() {
	agentSession.set(null);
	pendingEdits.set([]);
}

export function updateEditStatus(toolId: string, status: 'accepted' | 'rejected') {
	pendingEdits.update(edits =>
		edits.map(e => e.toolId === toolId ? { ...e, status } : e)
	);
}
