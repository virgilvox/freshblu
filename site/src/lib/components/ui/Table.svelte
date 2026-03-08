<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    columns: string[];
    children: Snippet;
  }

  let { columns, children }: Props = $props();
</script>

<div class="registry-demo">
  <table class="registry-table">
    <thead>
      <tr>
        {#each columns as col}
          <th>{col}</th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {@render children()}
    </tbody>
  </table>
</div>

<style>
  .registry-demo {
    border: 1px solid var(--border);
    overflow: hidden;
    overflow-x: auto;
  }
  .registry-table {
    width: 100%;
    border-collapse: collapse;
  }
  .registry-table th {
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--ink-muted);
    text-align: left;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
  }
  .registry-table :global(td) {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
    vertical-align: middle;
  }
  .registry-table :global(tr:last-child td) { border-bottom: none; }
  .registry-table :global(tr:hover td) { background: var(--void-lift); }
</style>
