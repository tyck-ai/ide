import { writable } from 'svelte/store';

export interface Toast {
	id: string;
	message: string;
	type: 'info' | 'success' | 'error' | 'warning';
	duration?: number;
}

const { subscribe, update } = writable<Toast[]>([]);

let counter = 0;

function add(message: string, type: Toast['type'] = 'info', duration = 4000) {
	const id = `toast-${++counter}`;
	update(toasts => [...toasts, { id, message, type, duration }]);
	if (duration > 0) {
		setTimeout(() => dismiss(id), duration);
	}
	return id;
}

function dismiss(id: string) {
	update(toasts => toasts.filter(t => t.id !== id));
}

export const toasts = { subscribe };

export const toast = {
	info: (msg: string, duration?: number) => add(msg, 'info', duration),
	success: (msg: string, duration?: number) => add(msg, 'success', duration),
	error: (msg: string, duration?: number) => add(msg, 'error', duration ?? 6000),
	warning: (msg: string, duration?: number) => add(msg, 'warning', duration),
	dismiss,
};
