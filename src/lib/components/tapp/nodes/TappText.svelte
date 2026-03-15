<script lang="ts">
  import type { UINode } from '../types';
  import { sanitizeStyle } from '../sanitize';

  interface Props {
    node: UINode;
  }

  let { node }: Props = $props();

  const content = $derived(node.props.content as string || '');
  const className = $derived(node.props.class as string || '');
</script>

<span class="tapp-text {className}" style={sanitizeStyle(node.props.style)}>
  {content}
</span>

<style>
  .tapp-text {
    font-size: 13px;
    line-height: 1.5;
    color: var(--color-text-primary, #e0e0e0);
  }
</style>
