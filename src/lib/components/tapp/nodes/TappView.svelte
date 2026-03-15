<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { UINode, UIEvent } from '../types';
  import { sanitizeStyle } from '../sanitize';

  interface Props {
    node: UINode;
    onEvent: (event: UIEvent) => void;
    children?: Snippet;
  }

  let { node, onEvent, children }: Props = $props();

  const className = $derived(node.props.class as string || '');
  const draggable = $derived(node.props.draggable as boolean || false);

  function handleClick(e: MouseEvent) {
    if (node.props.on_click) {
      onEvent({
        component_id: node.id,
        event_type: 'click',
        data: {
          action: node.props.on_click as string,
          ...(node.props.on_click_data as Record<string, unknown> || {}),
        },
      });
    }
  }

  function handleDragStart(e: DragEvent) {
    if (node.props.on_drag_start) {
      e.dataTransfer?.setData('text/plain', JSON.stringify(node.props.on_drag_start_data));
      onEvent({
        component_id: node.id,
        event_type: 'drag_start',
        data: node.props.on_drag_start_data,
      });
    }
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    if (node.props.on_drop) {
      const raw = e.dataTransfer?.getData('text/plain');
      let dragData: unknown = null;
      if (raw) {
        try { dragData = JSON.parse(raw); } catch { dragData = raw; }
      }
      onEvent({
        component_id: node.id,
        event_type: 'drop',
        data: {
          ...node.props.on_drop_data as Record<string, unknown>,
          dragData,
        },
      });
    }
  }

  function handleDragOver(e: DragEvent) {
    if (node.props.on_drop) {
      e.preventDefault();
    }
  }
</script>

<div
  class="tapp-view {className}"
  style={sanitizeStyle(node.props.style)}
  {draggable}
  onclick={handleClick}
  ondragstart={handleDragStart}
  ondrop={handleDrop}
  ondragover={handleDragOver}
  role={node.props.on_click ? 'button' : undefined}
  tabindex={node.props.on_click ? 0 : undefined}
>
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .tapp-view {
    display: flex;
    flex-direction: column;
  }

  .tapp-view[role='button'] {
    cursor: pointer;
  }

  .tapp-view[role='button']:hover {
    opacity: 0.9;
  }
</style>
