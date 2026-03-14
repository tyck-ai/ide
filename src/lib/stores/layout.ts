import { writable } from 'svelte/store';

// Simple panel visibility — no fake "modes" until they have real functionality
export const showContext = writable(true);
export const showInsight = writable(true);

// Full-page views
export const showSettings = writable(false);
export const showGitView = writable(false);
export const gitViewTab = writable<'changes' | 'history' | 'branches' | 'stashes'>('changes');

// Modals
export const showBranchSwitcher = writable(false);
export const showQuickCommit = writable(false);
