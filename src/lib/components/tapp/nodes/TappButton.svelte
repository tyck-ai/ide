<script lang="ts">
  import type { UINode, UIEvent } from '../types';

  interface Props {
    node: UINode;
    onEvent: (event: UIEvent) => void;
  }

  let { node, onEvent }: Props = $props();

  const label = $derived(node.props.label as string || '');
  const variant = $derived(node.props.variant as string || 'default');
  const disabled = $derived(node.props.disabled as boolean || false);
  const className = $derived(node.props.class as string || '');

  function handleClick() {
    if (!disabled && node.props.on_click) {
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
</script>

<button
  class="tapp-button tapp-button--{variant} {className}"
  {disabled}
  onclick={handleClick}
>
  {label}
</button>

<style>
  .tapp-button {
    padding: 6px 12px;
    font-size: 13px;
    font-weight: 500;
    border-radius: 4px;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.15s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .tapp-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tapp-button--default {
    background: var(--color-button-bg, #3c3c3c);
    color: var(--color-button-text, #e0e0e0);
    border-color: var(--color-border, #4a4a4a);
  }

  .tapp-button--default:hover:not(:disabled) {
    background: var(--color-button-hover-bg, #4a4a4a);
  }

  .tapp-button--primary {
    background: var(--color-primary, #007acc);
    color: white;
  }

  .tapp-button--primary:hover:not(:disabled) {
    background: var(--color-primary-hover, #005a9e);
  }

  .tapp-button--secondary {
    background: transparent;
    color: var(--color-text-secondary, #a0a0a0);
    border-color: var(--color-border, #4a4a4a);
  }

  .tapp-button--secondary:hover:not(:disabled) {
    background: var(--color-button-hover-bg, #4a4a4a);
    color: var(--color-text-primary, #e0e0e0);
  }

  .tapp-button--danger {
    background: var(--color-danger, #d73a49);
    color: white;
  }

  .tapp-button--danger:hover:not(:disabled) {
    background: var(--color-danger-hover, #cb2431);
  }
</style>
