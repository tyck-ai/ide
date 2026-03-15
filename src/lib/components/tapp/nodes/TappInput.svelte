<script lang="ts">
  import type { UINode, UIEvent } from '../types';

  interface Props {
    node: UINode;
    onEvent: (event: UIEvent) => void;
    multiline?: boolean;
  }

  let { node, onEvent, multiline = false }: Props = $props();

  const value = $derived(node.props.value as string || '');
  const placeholder = $derived(node.props.placeholder as string || '');
  const VALID_INPUT_TYPES = ['text', 'password', 'email', 'number', 'search', 'url', 'tel', 'date', 'time', 'color', 'range'];
  const inputType = $derived(VALID_INPUT_TYPES.includes(node.props.type as string) ? node.props.type as string : 'text');
  const rows = $derived(node.props.rows as number || 3);
  const disabled = $derived(node.props.disabled as boolean || false);
  const className = $derived(node.props.class as string || '');

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement | HTMLTextAreaElement;
    if (node.props.on_change) {
      onEvent({
        component_id: node.id,
        event_type: 'change',
        data: { action: node.props.on_change as string, value: target.value },
      });
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !multiline && node.props.on_submit) {
      e.preventDefault();
      const target = e.target as HTMLInputElement;
      onEvent({
        component_id: node.id,
        event_type: 'submit',
        data: { action: node.props.on_submit as string, value: target.value },
      });
    }
  }
</script>

{#if multiline}
  <textarea
    class="tapp-input tapp-textarea {className}"
    data-tapp-action={node.props.on_change as string || ''}
    {placeholder}
    {disabled}
    {rows}
    oninput={handleInput}
    value={value}
  ></textarea>
{:else}
  <input
    class="tapp-input {className}"
    data-tapp-action={node.props.on_change as string || ''}
    type={inputType}
    {placeholder}
    {disabled}
    oninput={handleInput}
    onkeydown={handleKeyDown}
    value={value}
  />
{/if}

<style>
  .tapp-input {
    padding: 6px 10px;
    font-size: 13px;
    border-radius: 4px;
    border: 1px solid var(--color-border, #4a4a4a);
    background: var(--color-input-bg, #2d2d2d);
    color: var(--color-text-primary, #e0e0e0);
    outline: none;
    transition: border-color 0.15s ease;
    width: 100%;
  }

  .tapp-input:focus {
    border-color: var(--color-primary, #007acc);
  }

  .tapp-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tapp-input::placeholder {
    color: var(--color-text-muted, #666);
  }

  .tapp-textarea {
    resize: vertical;
    min-height: 60px;
    font-family: inherit;
  }
</style>
