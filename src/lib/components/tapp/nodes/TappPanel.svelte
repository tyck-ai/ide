<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { UINode, UIEvent } from '../types';

  interface Props {
    node: UINode;
    onEvent: (event: UIEvent) => void;
    children?: Snippet;
  }

  let { node, onEvent, children }: Props = $props();

  const title = $derived(node.props.title as string || '');
  const className = $derived(node.props.class as string || '');
</script>

<div class="tapp-panel {className}">
  {#if title}
    <div class="tapp-panel__header">
      <h3 class="tapp-panel__title">{title}</h3>
    </div>
  {/if}
  <div class="tapp-panel__content">
    {#if children}
      {@render children()}
    {/if}
  </div>
</div>

<style>
  .tapp-panel {
    background: var(--color-panel-bg, #252526);
    border: 1px solid var(--color-border, #3c3c3c);
    border-radius: 6px;
    overflow: hidden;
  }

  .tapp-panel__header {
    padding: 10px 14px;
    border-bottom: 1px solid var(--color-border, #3c3c3c);
    background: var(--color-panel-header-bg, #2d2d2d);
  }

  .tapp-panel__title {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-primary, #e0e0e0);
  }

  .tapp-panel__content {
    padding: 14px;
  }
</style>
