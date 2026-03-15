<script lang="ts">
  import type { UINode } from '../types';

  interface Props {
    node: UINode;
  }

  let { node }: Props = $props();

  const content = $derived(node.props.content as string || '');
  const language = $derived(node.props.language as string || '');
</script>

<pre class="tapp-code"><code class="language-{language}">{content}</code></pre>

<style>
  .tapp-code {
    margin: 0;
    padding: 12px;
    background: var(--color-code-bg, #1e1e1e);
    border-radius: 4px;
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Consolas', monospace;
    font-size: 12px;
    line-height: 1.5;
    overflow-x: auto;
    color: var(--color-text-primary, #e0e0e0);
  }

  .tapp-code code {
    font-family: inherit;
  }
</style>
