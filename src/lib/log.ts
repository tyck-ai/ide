import { invoke } from '@tauri-apps/api/core';

function formatErr(err: unknown): string {
	if (err instanceof Error) {
		return err.stack ? `${err.message}\n${err.stack}` : err.message;
	}
	if (typeof err === 'object' && err !== null) {
		try { return JSON.stringify(err); } catch { /* ignore */ }
	}
	return String(err);
}

function send(level: string, context: string, err?: unknown): void {
	const message = err !== undefined ? `${context} — ${formatErr(err)}` : context;
	invoke('append_log', { level, message }).catch(() => { /* log sink must never throw */ });
}

export const log = {
	error: (context: string, err?: unknown) => send('error', context, err),
	warn:  (context: string, err?: unknown) => send('warn',  context, err),
	info:  (context: string, err?: unknown) => send('info',  context, err),
};
