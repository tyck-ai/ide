<script lang="ts">
  import type { UINode, UIEvent } from './types';
  import TappView from './nodes/TappView.svelte';
  import TappText from './nodes/TappText.svelte';
  import TappButton from './nodes/TappButton.svelte';
  import TappInput from './nodes/TappInput.svelte';
  import TappPanel from './nodes/TappPanel.svelte';
  import TappList from './nodes/TappList.svelte';
  import TappSpinner from './nodes/TappSpinner.svelte';
  import TappProgress from './nodes/TappProgress.svelte';
  import TappEmpty from './nodes/TappEmpty.svelte';
  import TappAlert from './nodes/TappAlert.svelte';
  import TappBadge from './nodes/TappBadge.svelte';
  import TappCode from './nodes/TappCode.svelte';
  import TappCheckbox from './nodes/TappCheckbox.svelte';
  import TappToggle from './nodes/TappToggle.svelte';

  interface Props {
    node: UINode;
    onEvent: (event: UIEvent) => void;
    depth?: number;
  }

  const MAX_DEPTH = 64;

  let { node, onEvent, depth = 0 }: Props = $props();

  import { sanitizeStyle, sanitizeSrc } from './sanitize';
</script>

{#if depth >= MAX_DEPTH}
  <div class="tapp-depth-limit">Node tree too deep</div>
{:else if node.node_type === 'view'}
  <TappView {node} {onEvent}>
    {#each node.children as child (child.id)}
      <svelte:self node={child} {onEvent} depth={depth + 1} />
    {/each}
  </TappView>
{:else if node.node_type === 'hstack'}
  <div class="tapp-hstack" style={sanitizeStyle(node.props.style)}>
    {#each node.children as child (child.id)}
      <svelte:self node={child} {onEvent} depth={depth + 1} />
    {/each}
  </div>
{:else if node.node_type === 'vstack'}
  <div class="tapp-vstack" style={sanitizeStyle(node.props.style)}>
    {#each node.children as child (child.id)}
      <svelte:self node={child} {onEvent} depth={depth + 1} />
    {/each}
  </div>
{:else if node.node_type === 'panel'}
  <TappPanel {node} {onEvent}>
    {#each node.children as child (child.id)}
      <svelte:self node={child} {onEvent} depth={depth + 1} />
    {/each}
  </TappPanel>
{:else if node.node_type === 'text'}
  <TappText {node} />
{:else if node.node_type === 'code'}
  <TappCode {node} />
{:else if node.node_type === 'button'}
  <TappButton {node} {onEvent} />
{:else if node.node_type === 'input'}
  <TappInput {node} {onEvent} />
{:else if node.node_type === 'textarea'}
  <TappInput {node} {onEvent} multiline />
{:else if node.node_type === 'checkbox'}
  <TappCheckbox {node} {onEvent} />
{:else if node.node_type === 'toggle'}
  <TappToggle {node} {onEvent} />
{:else if node.node_type === 'list'}
  <TappList {node} {onEvent}>
    {#each node.children as child (child.id)}
      <svelte:self node={child} {onEvent} depth={depth + 1} />
    {/each}
  </TappList>
{:else if node.node_type === 'badge'}
  <TappBadge {node} />
{:else if node.node_type === 'spinner'}
  <TappSpinner {node} />
{:else if node.node_type === 'progress'}
  <TappProgress {node} />
{:else if node.node_type === 'empty'}
  <TappEmpty {node} />
{:else if node.node_type === 'alert'}
  <TappAlert {node} />
{:else if node.node_type === 'select'}
  <select
    class="tapp-select"
    value={node.props.value as string || ''}
    disabled={node.props.disabled as boolean || false}
    onchange={(e) => {
      if (node.props.on_change) {
        onEvent({
          component_id: node.id,
          event_type: 'change',
          data: { action: node.props.on_change as string, value: (e.target as HTMLSelectElement).value },
        });
      }
    }}
  >
    {#if node.props.placeholder}
      <option value="" disabled selected={!(node.props.value as string)}>{node.props.placeholder as string}</option>
    {/if}
    {#each (Array.isArray(node.props.options) ? node.props.options : []) as opt}
      <option value={String(opt?.value ?? '')}>{String(opt?.label ?? '')}</option>
    {/each}
  </select>
{:else if node.node_type === 'table'}
  <div class="tapp-table-wrapper">
    <table class="tapp-table">
      {#if Array.isArray(node.props.headers)}
        <thead>
          <tr>
            {#each node.props.headers as header}
              <th>{String(header)}</th>
            {/each}
          </tr>
        </thead>
      {/if}
      <tbody>
        {#each node.children as child (child.id)}
          <svelte:self node={child} {onEvent} depth={depth + 1} />
        {/each}
      </tbody>
    </table>
  </div>
{:else if node.node_type === 'image'}
  <img
    class="tapp-image"
    src={sanitizeSrc(node.props.src)}
    alt={node.props.alt as string || ''}
    width={node.props.width as number || undefined}
    height={node.props.height as number || undefined}
    style={sanitizeStyle(node.props.style)}
  />
{:else if node.node_type === 'icon'}
  <span class="tapp-icon" title={node.props.name as string || ''}>
    {node.props.name as string || '?'}
  </span>
{:else if node.node_type === 'modal'}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="tapp-modal-backdrop" onclick={() => {
    if (node.props.on_close) {
      onEvent({ component_id: node.id, event_type: 'click', data: { action: node.props.on_close as string } });
    }
  }} onkeydown={(e) => {
    if (e.key === 'Escape' && node.props.on_close) {
      onEvent({ component_id: node.id, event_type: 'click', data: { action: node.props.on_close as string } });
    }
  }} role="dialog" aria-modal="true">
    <div class="tapp-modal" onclick={(e) => e.stopPropagation()}>
      {#if node.props.title}
        <div class="tapp-modal__header">
          <span>{node.props.title as string}</span>
          {#if node.props.on_close}
            <button class="tapp-modal__close" onclick={() => {
              onEvent({ component_id: node.id, event_type: 'click', data: { action: node.props.on_close as string } });
            }}>✕</button>
          {/if}
        </div>
      {/if}
      <div class="tapp-modal__body">
        {#each node.children as child (child.id)}
          <svelte:self node={child} {onEvent} depth={depth + 1} />
        {/each}
      </div>
    </div>
  </div>
{:else if node.node_type === 'slider'}
  <input
    class="tapp-slider"
    type="range"
    min={Math.min(Number.isFinite(node.props.min as number) ? node.props.min as number : 0, Number.isFinite(node.props.max as number) ? node.props.max as number : 100)}
    max={Math.max(Number.isFinite(node.props.min as number) ? node.props.min as number : 0, Number.isFinite(node.props.max as number) ? node.props.max as number : 100)}
    step={Math.max(0.001, Number.isFinite(node.props.step as number) ? node.props.step as number : 1)}
    value={Number.isFinite(node.props.value as number) ? node.props.value as number : 50}
    disabled={node.props.disabled as boolean || false}
    oninput={(e) => {
      if (node.props.on_change) {
        onEvent({
          component_id: node.id,
          event_type: 'change',
          data: { action: node.props.on_change as string, value: Number((e.target as HTMLInputElement).value) },
        });
      }
    }}
  />
{:else if node.node_type === 'scroll'}
  <div class="tapp-scroll" style={sanitizeStyle(node.props.style)}>
    {#each node.children as child (child.id)}
      <svelte:self node={child} {onEvent} depth={depth + 1} />
    {/each}
  </div>
{:else if node.node_type === 'tabs'}
  <div class="tapp-tabs">
    {#each node.children as child (child.id)}
      <svelte:self node={child} {onEvent} depth={depth + 1} />
    {/each}
  </div>
{:else}
  <!-- Unimplemented node type: render children as fallback -->
  <div class="tapp-fallback" title={`Unsupported: ${node.node_type}`}>
    {#each node.children as child (child.id)}
      <svelte:self node={child} {onEvent} depth={depth + 1} />
    {/each}
  </div>
{/if}

<style>
  .tapp-hstack {
    display: flex;
    flex-direction: row;
    gap: 8px;
    align-items: center;
  }

  .tapp-vstack {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .tapp-scroll {
    overflow: auto;
    max-height: 100%;
  }

  .tapp-tabs {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .tapp-fallback {
    display: contents;
  }

  .tapp-depth-limit {
    color: var(--color-error-text, #f85149);
    font-size: 11px;
    padding: 4px;
  }

  .tapp-select {
    padding: 6px 10px;
    font-size: 13px;
    border-radius: 4px;
    border: 1px solid var(--color-border, #4a4a4a);
    background: var(--color-input-bg, #2d2d2d);
    color: var(--color-text-primary, #e0e0e0);
    outline: none;
    cursor: pointer;
    min-width: 120px;
  }
  .tapp-select:focus {
    border-color: var(--color-primary, #007acc);
  }
  .tapp-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tapp-table-wrapper {
    overflow-x: auto;
    width: 100%;
  }
  .tapp-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }
  .tapp-table th,
  .tapp-table :global(td) {
    padding: 6px 10px;
    border: 1px solid var(--color-border, #3c3c3c);
    text-align: left;
  }
  .tapp-table th {
    background: var(--color-panel-header-bg, #2d2d2d);
    font-weight: 600;
    font-size: 12px;
    color: var(--color-text-secondary, #a0a0a0);
  }

  .tapp-image {
    max-width: 100%;
    height: auto;
    border-radius: 4px;
  }

  .tapp-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    width: 1.2em;
    height: 1.2em;
  }

  .tapp-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .tapp-modal {
    background: var(--color-bg, #1e1e1e);
    border: 1px solid var(--color-border, #3c3c3c);
    border-radius: 8px;
    min-width: 300px;
    max-width: 80vw;
    max-height: 80vh;
    overflow: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }
  .tapp-modal__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border, #3c3c3c);
    font-weight: 600;
    font-size: 14px;
  }
  .tapp-modal__close {
    background: none;
    border: none;
    color: var(--color-text-secondary, #a0a0a0);
    cursor: pointer;
    padding: 2px 6px;
    font-size: 14px;
    border-radius: 3px;
  }
  .tapp-modal__close:hover {
    background: var(--color-button-hover-bg, #4a4a4a);
    color: var(--color-text-primary, #e0e0e0);
  }
  .tapp-modal__body {
    padding: 16px;
  }

  .tapp-slider {
    width: 100%;
    cursor: pointer;
    accent-color: var(--color-primary, #007acc);
  }
  .tapp-slider:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
