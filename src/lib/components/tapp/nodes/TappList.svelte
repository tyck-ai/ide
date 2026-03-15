<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { UINode, UIEvent } from '../types';

  interface Props {
    node: UINode;
    onEvent: (event: UIEvent) => void;
    children?: Snippet;
  }

  let { node, onEvent, children }: Props = $props();

  const className = $derived(node.props.class as string || '');
</script>

<div class="tapp-list {className}">
  {#if children}
    {@render children()}
  {/if}
</div>

<style>
  .tapp-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
</style>
