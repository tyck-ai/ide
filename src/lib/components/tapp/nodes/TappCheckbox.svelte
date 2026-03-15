<script lang="ts">
  import type { UINode, UIEvent } from '../types';

  interface Props {
    node: UINode;
    onEvent: (event: UIEvent) => void;
  }

  let { node, onEvent }: Props = $props();

  const checked = $derived(node.props.checked as boolean || false);
  const label = $derived(node.props.label as string || '');
  const disabled = $derived(node.props.disabled as boolean || false);

  function handleChange(e: Event) {
    const target = e.target as HTMLInputElement;
    if (node.props.on_change) {
      onEvent({
        component_id: node.id,
        event_type: 'change',
        data: { action: node.props.on_change as string, checked: target.checked },
      });
    }
  }
</script>

<label class="tapp-checkbox" class:tapp-checkbox--disabled={disabled}>
  <input
    type="checkbox"
    {checked}
    {disabled}
    onchange={handleChange}
  />
  <span class="tapp-checkbox__box">
    {#if checked}
      <svg viewBox="0 0 16 16" fill="currentColor">
        <path d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.75.75 0 011.06-1.06L6 10.94l6.72-6.72a.75.75 0 011.06 0z"/>
      </svg>
    {/if}
  </span>
  {#if label}
    <span class="tapp-checkbox__label">{label}</span>
  {/if}
</label>

<style>
  .tapp-checkbox {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
    color: var(--color-text-primary, #e0e0e0);
  }

  .tapp-checkbox--disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tapp-checkbox input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .tapp-checkbox__box {
    width: 16px;
    height: 16px;
    border: 1px solid var(--color-border, #4a4a4a);
    border-radius: 3px;
    background: var(--color-input-bg, #2d2d2d);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .tapp-checkbox input:checked + .tapp-checkbox__box {
    background: var(--color-primary, #007acc);
    border-color: var(--color-primary, #007acc);
  }

  .tapp-checkbox__box svg {
    width: 12px;
    height: 12px;
    color: white;
  }

  .tapp-checkbox__label {
    user-select: none;
  }
</style>
