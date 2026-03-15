<script lang="ts">
  import type { UINode } from '../types';

  interface Props {
    node: UINode;
  }

  let { node }: Props = $props();

  const text = $derived(node.props.text as string || '');
  const variant = $derived(node.props.variant as string || 'default');
</script>

<span class="tapp-badge tapp-badge--{variant}">
  {text}
</span>

<style>
  .tapp-badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    font-size: 11px;
    font-weight: 500;
    border-radius: 10px;
  }

  .tapp-badge--default {
    background: var(--color-badge-bg, #3c3c3c);
    color: var(--color-text-secondary, #a0a0a0);
  }

  .tapp-badge--primary {
    background: var(--color-primary, #007acc);
    color: white;
  }

  .tapp-badge--success {
    background: var(--color-success-bg, #1c4428);
    color: var(--color-success-text, #56d364);
  }

  .tapp-badge--warning {
    background: var(--color-warning-bg, #4d3800);
    color: var(--color-warning-text, #e3b341);
  }

  .tapp-badge--error {
    background: var(--color-error-bg, #4d1f24);
    color: var(--color-error-text, #f85149);
  }
</style>
