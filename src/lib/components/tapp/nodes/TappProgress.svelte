<script lang="ts">
  import type { UINode } from '../types';

  interface Props {
    node: UINode;
  }

  let { node }: Props = $props();

  const value = $derived(Math.min(100, Math.max(0, (node.props.value as number) || 0)));
  const showLabel = $derived(node.props.show_label as boolean ?? true);
</script>

<div class="tapp-progress">
  <div class="tapp-progress__bar">
    <div class="tapp-progress__fill" style="width: {value}%"></div>
  </div>
  {#if showLabel}
    <span class="tapp-progress__label">{value.toFixed(0)}%</span>
  {/if}
</div>

<style>
  .tapp-progress {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .tapp-progress__bar {
    flex: 1;
    height: 6px;
    background: var(--color-progress-bg, #3c3c3c);
    border-radius: 3px;
    overflow: hidden;
  }

  .tapp-progress__fill {
    height: 100%;
    background: var(--color-primary, #007acc);
    border-radius: 3px;
    transition: width 0.2s ease;
  }

  .tapp-progress__label {
    font-size: 11px;
    color: var(--color-text-secondary, #a0a0a0);
    min-width: 36px;
    text-align: right;
  }
</style>
