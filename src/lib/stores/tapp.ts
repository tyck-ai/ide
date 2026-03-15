import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface AppInfo {
  id: string;
  name: string;
  version: string;
  description?: string;
  enabled: boolean;
  running: boolean;
  layout: 'full' | 'sidebar' | 'panel' | 'modal';
}

export interface AppListing {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  icon_url: string | null;
  download_url: string;
  download_size: number;
  sha256: string;
  signature: string | null;
  public_key: string | null;
  permissions: string[];
  created_at: string;
  updated_at: string;
  downloads: number;
  rating: number | null;
}

export interface UpdateInfo {
  app_id: string;
  current_version: string;
  latest_version: string;
  download_url: string;
  changelog: string | null;
  is_critical: boolean;
}

interface TappState {
  apps: AppInfo[];
  activeAppId: string | null;
  loading: boolean;
  error: string | null;
  storeListings: AppListing[];
  storeLoading: boolean;
  storeQuery: string;
  updates: UpdateInfo[];
  recentAppIds: string[];
}

const RECENT_APPS_KEY = 'tyck:recentApps';
const MAX_RECENT_APPS = 5;

function loadRecentApps(): string[] {
  try {
    const stored = localStorage.getItem(RECENT_APPS_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

function saveRecentApps(ids: string[]) {
  try {
    localStorage.setItem(RECENT_APPS_KEY, JSON.stringify(ids));
  } catch {
    // ignore storage errors
  }
}

const initialState: TappState = {
  apps: [],
  activeAppId: null,
  loading: false,
  error: null,
  storeListings: [],
  storeLoading: false,
  storeQuery: '',
  updates: [],
  recentAppIds: loadRecentApps(),
};

function createTappStore() {
  const { subscribe, set, update } = writable<TappState>(initialState);

  return {
    subscribe,

    async refresh() {
      update(s => ({ ...s, loading: true, error: null }));

      try {
        const apps = await invoke<AppInfo[]>('tapp_list_apps');
        update(s => ({ ...s, apps, loading: false }));
      } catch (e) {
        update(s => ({
          ...s,
          loading: false,
          error: e instanceof Error ? e.message : String(e),
        }));
      }
    },

    async install(manifestPath: string) {
      try {
        const appId = await invoke<string>('tapp_install_app', { manifestPath });
        await this.refresh();
        return appId;
      } catch (e) {
        throw e;
      }
    },

    async uninstall(appId: string) {
      try {
        await invoke('tapp_uninstall_app', { appId });
        await this.refresh();
      } catch (e) {
        throw e;
      }
    },

    async start(appId: string) {
      try {
        // Stop the currently active app first (sequentially) to avoid
        // lock contention between the old app's shutdown and the new
        // app's get_ui call.
        let previousAppId: string | null = null;
        tapp.subscribe(s => { previousAppId = s.activeAppId; })();
        if (previousAppId && previousAppId !== appId) {
          try {
            await invoke('tapp_stop_app', { appId: previousAppId });
          } catch (e) {
            console.warn('Failed to stop previous app:', e);
          }
          update(s => ({
            ...s,
            activeAppId: null,
            apps: s.apps.map(a =>
              a.id === previousAppId ? { ...a, running: false } : a
            ),
          }));
        }

        await invoke('tapp_start_app', { appId });
        update(s => ({
          ...s,
          activeAppId: appId,
          apps: s.apps.map(a =>
            a.id === appId ? { ...a, running: true } : a
          ),
        }));
      } catch (e) {
        throw e;
      }
    },

    async stop(appId: string) {
      try {
        await invoke('tapp_stop_app', { appId });
        update(s => ({
          ...s,
          activeAppId: s.activeAppId === appId ? null : s.activeAppId,
          apps: s.apps.map(a =>
            a.id === appId ? { ...a, running: false } : a
          ),
        }));
      } catch (e) {
        throw e;
      }
    },

    setActiveApp(appId: string | null) {
      update(s => ({ ...s, activeAppId: appId }));
    },

    clearError() {
      update(s => ({ ...s, error: null }));
    },

    async enable(appId: string) {
      try {
        await invoke('tapp_enable_app', { appId });
        update(s => ({
          ...s,
          apps: s.apps.map(a =>
            a.id === appId ? { ...a, enabled: true } : a
          ),
        }));
      } catch (e) {
        throw e;
      }
    },

    async disable(appId: string) {
      try {
        await invoke('tapp_disable_app', { appId });
        update(s => ({
          ...s,
          apps: s.apps.map(a =>
            a.id === appId ? { ...a, enabled: false, running: false } : a
          ),
        }));
      } catch (e) {
        throw e;
      }
    },

    async searchStore(query: string) {
      update(s => ({ ...s, storeLoading: true, storeQuery: query }));
      try {
        const listings = await invoke<AppListing[]>('tapp_store_search', { query });
        update(s => ({ ...s, storeListings: listings, storeLoading: false }));
      } catch (e) {
        update(s => ({
          ...s,
          storeLoading: false,
          error: e instanceof Error ? e.message : String(e),
        }));
      }
    },

    async refreshStore() {
      update(s => ({ ...s, storeLoading: true }));
      try {
        await invoke('tapp_store_refresh');
        const listings = await invoke<AppListing[]>('tapp_store_search', { query: '' });
        update(s => ({ ...s, storeListings: listings, storeLoading: false }));
      } catch (e) {
        update(s => ({
          ...s,
          storeLoading: false,
          error: e instanceof Error ? e.message : String(e),
        }));
      }
    },

    async checkUpdates() {
      try {
        const updates = await invoke<UpdateInfo[]>('tapp_store_check_updates');
        update(s => ({ ...s, updates }));
        return updates;
      } catch (e) {
        update(s => ({
          ...s,
          error: e instanceof Error ? e.message : String(e),
        }));
        return [];
      }
    },

    async installFromStore(appId: string) {
      try {
        const manifestPath = await invoke<string>('tapp_store_download', { appId });
        const installedId = await invoke<string>('tapp_install_app', { manifestPath });
        await this.refresh();
        return installedId;
      } catch (e) {
        throw e;
      }
    },

    trackRecent(appId: string) {
      update(s => {
        const filtered = s.recentAppIds.filter(id => id !== appId);
        const newRecent = [appId, ...filtered].slice(0, MAX_RECENT_APPS);
        saveRecentApps(newRecent);
        return { ...s, recentAppIds: newRecent };
      });
    },
  };
}

export const tapp = createTappStore();

export const installedApps = derived(tapp, $tapp => $tapp.apps);
export const enabledApps = derived(tapp, $tapp => $tapp.apps.filter(a => a.enabled));
export const runningApps = derived(tapp, $tapp => $tapp.apps.filter(a => a.running));
export const activeApp = derived(tapp, $tapp =>
  $tapp.activeAppId ? $tapp.apps.find(a => a.id === $tapp.activeAppId) : null
);
export const storeListings = derived(tapp, $tapp => $tapp.storeListings);
export const storeLoading = derived(tapp, $tapp => $tapp.storeLoading);
export const availableUpdates = derived(tapp, $tapp => $tapp.updates);
export const recentApps = derived(tapp, $tapp => {
  return $tapp.recentAppIds
    .map(id => $tapp.apps.find(a => a.id === id))
    .filter((a): a is AppInfo => a !== undefined);
});
