<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { get } from 'svelte/store';
  import { activeApp } from '$lib/stores/tapp';
  import type { UITree, UIEvent, AppInfo, LayoutMode } from './types';
  import TappRenderer from './TappRenderer.svelte';

  interface Props {
    appId: string;
    layout?: LayoutMode;
    onClose?: () => void;
  }

  let { appId, layout = 'full', onClose }: Props = $props();

  let uiTree: UITree | null = $state(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let appInfo: AppInfo | null = $state(null);
  let pendingRaf: number | null = null;
  let loadPhase = $state<'resolving' | 'starting' | 'rendering'>('resolving');

  async function loadApp() {
    try {
      loading = true;
      error = null;
      loadPhase = 'resolving';

      const apps = await invoke<AppInfo[]>('tapp_list_apps');
      appInfo = apps.find(a => a.id === appId) || null;

      if (appInfo && !appInfo.running) {
        loadPhase = 'starting';
        await invoke('tapp_start_app', { appId });
        const updatedApps = await invoke<AppInfo[]>('tapp_list_apps');
        appInfo = updatedApps.find(a => a.id === appId) || null;
      }

      loadPhase = 'rendering';
      try {
        uiTree = await invoke<UITree>('tapp_get_ui', { appId });
      } catch (e) {
        console.log('No UI available for app:', appId, e);
        uiTree = null;
      }

      loading = false;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      loading = false;
    }
  }

  async function handleEvent(event: UIEvent) {
    try {
      const eventData = (event.data || {}) as Record<string, unknown>;
      const { action: actionName, ...rest } = eventData;
      const action = {
        name: actionName || event.event_type,
        data: rest,
      };

      const response = await invoke<{ render?: boolean; status?: string }>('tapp_dispatch_action', {
        appId,
        action,
      });

      if (response?.render !== false) {
        // Capture focus state before re-render
        const active = document.activeElement as HTMLElement | null;
        const focusAction = active?.getAttribute('data-tapp-action') || '';
        const selStart = (active as HTMLInputElement)?.selectionStart ?? null;
        const selEnd = (active as HTMLInputElement)?.selectionEnd ?? null;

        try {
          uiTree = await invoke<UITree>('tapp_get_ui', { appId });
        } catch (e) {
          console.log('Failed to refresh UI:', e);
        }

        // Restore focus after Svelte updates the DOM
        if (focusAction) {
          if (pendingRaf !== null) cancelAnimationFrame(pendingRaf);
          pendingRaf = requestAnimationFrame(() => {
            pendingRaf = null;
            const el = document.querySelector(`[data-tapp-action="${CSS.escape(focusAction)}"]`) as HTMLElement;
            el?.focus();
            if (selStart !== null && selEnd !== null && el && 'setSelectionRange' in el) {
              (el as HTMLInputElement).setSelectionRange(selStart, selEnd);
            }
          });
        }
      }
    } catch (e) {
      console.error('Error handling tapp event:', e);
    }
  }

  async function stopApp() {
    try {
      await invoke('tapp_stop_app', { appId });
    } catch (e) {
      console.error('Error stopping app:', e);
    }
  }

  function handleClose() {
    stopApp();
    onClose?.();
  }

  onMount(() => {
    loadApp();
  });

  onDestroy(() => {
    if (pendingRaf !== null) cancelAnimationFrame(pendingRaf);

    // Only stop the app if it's still the active one. During an app swap,
    // tapp.start() already stopped the previous app before mounting the
    // new container, so we don't need to stop it again (which would
    // contend on the WASM lock and cause a hang).
    const current = get(activeApp);
    if (!current || current.id === appId) {
      stopApp();
    }
  });
</script>

<div class="tapp-container tapp-container--{layout}">
  <div class="tapp-container__header">
    <span class="tapp-container__title">
      {appInfo?.name || appId}
    </span>
    {#if onClose}
      <button class="tapp-container__close" onclick={handleClose}>
        ✕
      </button>
    {/if}
  </div>

  <div class="tapp-container__content">
    {#if loading}
      <div class="tapp-loading">
        <div class="tapp-loading__icon">
          <svg class="tapp-loading__ring" viewBox="0 0 48 48">
            <circle cx="24" cy="24" r="20" fill="none" stroke-width="3" stroke="var(--color-border-muted, #3c3c3c)" />
            <circle cx="24" cy="24" r="20" fill="none" stroke-width="3" stroke="var(--color-accent, #007acc)" stroke-dasharray="80 126" stroke-linecap="round" />
          </svg>
        </div>
        <span class="tapp-loading__name">{appInfo?.name || appId}</span>
        <span class="tapp-loading__phase">
          {#if loadPhase === 'resolving'}
            Resolving app...
          {:else if loadPhase === 'starting'}
            Starting WASM runtime...
          {:else}
            Rendering UI...
          {/if}
        </span>
        <div class="tapp-loading__steps">
          <div class="tapp-loading__step" class:done={loadPhase !== 'resolving'} class:active={loadPhase === 'resolving'}>
            <span class="tapp-loading__dot"></span> Resolve
          </div>
          <div class="tapp-loading__step" class:done={loadPhase === 'rendering'} class:active={loadPhase === 'starting'}>
            <span class="tapp-loading__dot"></span> Start
          </div>
          <div class="tapp-loading__step" class:active={loadPhase === 'rendering'}>
            <span class="tapp-loading__dot"></span> Render
          </div>
        </div>
      </div>
    {:else if error}
      <div class="tapp-container__error">
        <span class="tapp-container__error-icon">⚠</span>
        <span>{error}</span>
        <button onclick={loadApp}>Retry</button>
      </div>
    {:else if uiTree}
      <TappRenderer node={uiTree} onEvent={handleEvent} />
    {:else}
      <div class="tapp-container__empty">
        <div class="tapp-container__status">
          <span class="tapp-container__status-icon">✓</span>
          <span class="tapp-container__status-text">App Started</span>
        </div>
        <span class="tapp-container__status-name">{appInfo?.name || appId}</span>
        <span class="tapp-container__status-hint">
          This app has no UI component.<br/>
          Agent tools are available.
        </span>
      </div>
    {/if}
  </div>
</div>

<style>
  .tapp-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg, #1e1e1e);
    color: var(--color-text-primary, #e0e0e0);
  }

  .tapp-container__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border, #3c3c3c);
    background: var(--color-panel-header-bg, #2d2d2d);
  }

  .tapp-container__title {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--color-text-secondary, #a0a0a0);
  }

  .tapp-container__close {
    background: none;
    border: none;
    color: var(--color-text-secondary, #a0a0a0);
    cursor: pointer;
    padding: 2px 6px;
    font-size: 14px;
    border-radius: 3px;
  }

  .tapp-container__close:hover {
    background: var(--color-button-hover-bg, #4a4a4a);
    color: var(--color-text-primary, #e0e0e0);
  }

  .tapp-container__content {
    flex: 1;
    overflow: auto;
    padding: 12px;
  }

  .tapp-container__error,
  .tapp-container__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    color: var(--color-text-secondary, #a0a0a0);
    font-size: 13px;
  }

  .tapp-container__status {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--color-success, #238636);
    border-radius: 6px;
  }

  .tapp-container__status-icon {
    color: white;
    font-size: 14px;
  }

  .tapp-container__status-text {
    color: white;
    font-weight: 600;
    font-size: 12px;
  }

  .tapp-container__status-name {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary, #e0e0e0);
    margin-top: 8px;
  }

  .tapp-container__status-hint {
    font-size: 12px;
    text-align: center;
    line-height: 1.5;
    opacity: 0.7;
    margin-top: 8px;
  }

  .tapp-container__error {
    color: var(--color-error-text, #f85149);
  }

  .tapp-container__error-icon {
    font-size: 24px;
  }

  .tapp-container__error button {
    margin-top: 8px;
    padding: 6px 12px;
    background: var(--color-button-bg, #3c3c3c);
    border: 1px solid var(--color-border, #4a4a4a);
    border-radius: 4px;
    color: var(--color-text-primary, #e0e0e0);
    cursor: pointer;
    font-size: 12px;
  }

  .tapp-container__error button:hover {
    background: var(--color-button-hover-bg, #4a4a4a);
  }

  /* Loading screen */
  .tapp-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 16px;
    animation: fadeIn 0.2s ease-out;
  }

  .tapp-loading__icon {
    width: 48px;
    height: 48px;
  }

  .tapp-loading__ring {
    width: 100%;
    height: 100%;
    animation: spin 1.2s linear infinite;
  }

  .tapp-loading__name {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary, #e0e0e0);
  }

  .tapp-loading__phase {
    font-size: 12px;
    color: var(--color-text-secondary, #a0a0a0);
    min-height: 18px;
  }

  .tapp-loading__steps {
    display: flex;
    gap: 24px;
    margin-top: 8px;
  }

  .tapp-loading__step {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--color-text-secondary, #555);
    transition: color 0.2s;
  }

  .tapp-loading__step.active {
    color: var(--color-accent, #007acc);
  }

  .tapp-loading__step.done {
    color: var(--color-success, #3fb950);
  }

  .tapp-loading__dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
    transition: background 0.2s;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  /* Layout variants */
  .tapp-container--full {
    /* Fills its grid cell — the page layout handles placement */
  }

  .tapp-container--modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 80%;
    max-width: 800px;
    max-height: 80vh;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    z-index: 100;
  }
</style>
