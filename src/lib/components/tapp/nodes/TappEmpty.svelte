<script lang="ts">
  import type { UINode } from '../types';

  interface Props {
    node: UINode;
  }

  let { node }: Props = $props();

  const message = $derived(node.props.message as string || 'No items');
</script>

<div class="tapp-empty">
  <div class="tapp-empty__icon">
    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <rect x="3" y="3" width="18" height="18" rx="2" />
      <line x1="9" y1="9" x2="15" y2="15" />
      <line x1="15" y1="9" x2="9" y2="15" />
    </svg>
  </div>
  <p class="tapp-empty__message">{message}</p>
</div>

<style>
  .tapp-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px;
    text-align: center;
  }

  .tapp-empty__icon {
    color: var(--color-text-muted, #666);
    margin-bottom: 8px;
  }

  .tapp-empty__message {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-secondary, #a0a0a0);
  }
</style>
