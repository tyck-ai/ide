<script lang="ts">
  import type { UINode } from '../types';

  interface Props {
    node: UINode;
  }

  let { node }: Props = $props();

  const size = $derived(node.props.size as string || 'medium');
</script>

<div class="tapp-spinner tapp-spinner--{size}">
  <div class="tapp-spinner__circle"></div>
</div>

<style>
  .tapp-spinner {
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .tapp-spinner__circle {
    border-radius: 50%;
    border: 2px solid var(--color-border, #4a4a4a);
    border-top-color: var(--color-primary, #007acc);
    animation: spin 0.8s linear infinite;
  }

  .tapp-spinner--small .tapp-spinner__circle {
    width: 16px;
    height: 16px;
  }

  .tapp-spinner--medium .tapp-spinner__circle {
    width: 24px;
    height: 24px;
  }

  .tapp-spinner--large .tapp-spinner__circle {
    width: 36px;
    height: 36px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
