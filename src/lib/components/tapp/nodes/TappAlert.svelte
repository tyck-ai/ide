<script lang="ts">
  import type { UINode } from '../types';

  interface Props {
    node: UINode;
  }

  let { node }: Props = $props();

  const message = $derived(node.props.message as string || '');
  const variant = $derived(node.props.variant as string || 'info');
</script>

<div class="tapp-alert tapp-alert--{variant}">
  <span class="tapp-alert__icon">
    {#if variant === 'success'}
      ✓
    {:else if variant === 'warning'}
      ⚠
    {:else if variant === 'error'}
      ✕
    {:else}
      ℹ
    {/if}
  </span>
  <span class="tapp-alert__message">{message}</span>
</div>

<style>
  .tapp-alert {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 14px;
    border-radius: 4px;
    font-size: 13px;
  }

  .tapp-alert__icon {
    flex-shrink: 0;
    font-weight: bold;
  }

  .tapp-alert__message {
    flex: 1;
  }

  .tapp-alert--info {
    background: var(--color-info-bg, #1e3a5f);
    color: var(--color-info-text, #58a6ff);
  }

  .tapp-alert--success {
    background: var(--color-success-bg, #1c4428);
    color: var(--color-success-text, #56d364);
  }

  .tapp-alert--warning {
    background: var(--color-warning-bg, #4d3800);
    color: var(--color-warning-text, #e3b341);
  }

  .tapp-alert--error {
    background: var(--color-error-bg, #4d1f24);
    color: var(--color-error-text, #f85149);
  }
</style>
