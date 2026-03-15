<script lang="ts">
  import type { UINode, UIEvent } from '../types';

  interface Props {
    node: UINode;
    onEvent: (event: UIEvent) => void;
  }

  let { node, onEvent }: Props = $props();

  const enabled = $derived(node.props.enabled as boolean || false);
  const label = $derived(node.props.label as string || '');
  const disabled = $derived(node.props.disabled as boolean || false);

  function handleClick() {
    if (!disabled && node.props.on_change) {
      onEvent({
        component_id: node.id,
        event_type: 'change',
        data: { action: node.props.on_change as string, enabled: !enabled },
      });
    }
  }
</script>

<button
  class="tapp-toggle"
  class:tapp-toggle--enabled={enabled}
  class:tapp-toggle--disabled={disabled}
  {disabled}
  onclick={handleClick}
  type="button"
  role="switch"
  aria-checked={enabled}
>
  <span class="tapp-toggle__track">
    <span class="tapp-toggle__thumb"></span>
  </span>
  {#if label}
    <span class="tapp-toggle__label">{label}</span>
  {/if}
</button>

<style>
  .tapp-toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font-size: 13px;
    color: var(--color-text-primary, #e0e0e0);
  }

  .tapp-toggle--disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tapp-toggle__track {
    width: 36px;
    height: 20px;
    border-radius: 10px;
    background: var(--color-toggle-bg, #3c3c3c);
    position: relative;
    transition: background 0.15s ease;
  }

  .tapp-toggle--enabled .tapp-toggle__track {
    background: var(--color-primary, #007acc);
  }

  .tapp-toggle__thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: white;
    transition: transform 0.15s ease;
  }

  .tapp-toggle--enabled .tapp-toggle__thumb {
    transform: translateX(16px);
  }

  .tapp-toggle__label {
    user-select: none;
  }
</style>
