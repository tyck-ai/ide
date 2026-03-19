import { log } from './log';

function fmtArg(a: unknown): string {
	if (a instanceof Error) return a.stack ?? a.message;
	if (typeof a === 'object' && a !== null) {
		try { return JSON.stringify(a); } catch { /* ignore */ }
	}
	return String(a);
}

export function initDiagnostics(): void {
	// Capture unhandled JS errors
	window.onerror = (_message, source, lineno, colno, error) => {
		const loc = source ? ` (${source}:${lineno}:${colno})` : '';
		log.error(`[uncaught]${loc}`, error ?? _message);
		return false;
	};

	// Capture unhandled promise rejections
	window.onunhandledrejection = (event) => {
		log.error('[unhandledrejection]', event.reason);
	};

	// Override console.error so existing calls are also captured
	const origError = console.error.bind(console);
	console.error = (...args: unknown[]) => {
		origError(...args);
		log.error('[console.error]', args.map(fmtArg).join(' '));
	};

	// Override console.warn
	const origWarn = console.warn.bind(console);
	console.warn = (...args: unknown[]) => {
		origWarn(...args);
		log.warn('[console.warn]', args.map(fmtArg).join(' '));
	};
}
