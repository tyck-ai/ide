import { writable } from 'svelte/store';
import type { AppListing } from './tapp';

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
export const showAppLauncher = writable(false);

// Pending app install (triggers permission review modal)
export interface PendingInstall {
  source: 'file' | 'store';
  listing?: AppListing;
  appId?: string;
  path?: string;
}
export const pendingInstall = writable<PendingInstall | null>(null);

// Session sidebar visibility (agent mode)
export const showSessionSidebar = writable(false);
