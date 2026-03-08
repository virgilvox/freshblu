<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'default' | 'pulse' | 'signal' | 'fault';
    title?: string;
    meta?: string;
    children: Snippet;
  }

  let { variant = 'default', title, meta, children }: Props = $props();
</script>

<div class="card" class:card-pulse={variant === 'pulse'} class:card-signal={variant === 'signal'} class:card-fault={variant === 'fault'}>
  {#if title}
    <div class="card-header">
      <span class="card-title">{title}</span>
      {#if meta}
        <span class="card-meta-inline">{meta}</span>
      {/if}
    </div>
  {/if}
  <div class="card-body">
    {#if meta && !title}
      <div class="card-meta">{meta}</div>
    {/if}
    {@render children()}
  </div>
</div>

<style>
  .card {
    background: var(--void-lift);
    border: 1px solid var(--border);
    transition: transform var(--dur-med) var(--ease-snap), border-color var(--dur-med);
  }
  .card:hover {
    transform: translateY(-2px);
    border-color: var(--border-strong);
  }
  .card-header {
    padding: 14px 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .card-title {
    font-family: var(--font-display);
    font-size: var(--text-base);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .card-body { padding: 20px; }
  .card-meta, .card-meta-inline {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .card-meta { margin-bottom: 10px; }
  .card-pulse  { border-left: 3px solid var(--pulse); }
  .card-signal { border-left: 3px solid var(--signal); }
  .card-fault  { border-left: 3px solid var(--fault); }
</style>
