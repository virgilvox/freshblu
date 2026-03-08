<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'online' | 'warn' | 'fault' | 'pending' | 'pulse' | 'muted';
    children: Snippet;
  }

  let { variant = 'pulse', children }: Props = $props();
</script>

<span class="badge badge-{variant}">
  {@render children()}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-family: var(--font-ui);
    font-size: 9px;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    padding: 3px 8px;
    border: 1px solid;
    border-radius: var(--r-sm);
  }

  .badge-online  { color: var(--online);  border-color: var(--online);  background: rgba(0,209,122,0.1); }
  .badge-warn    { color: var(--warn);    border-color: var(--warn);    background: rgba(245,166,35,0.1); }
  .badge-fault   { color: var(--fault);   border-color: var(--fault);   background: rgba(255,68,85,0.1); }
  .badge-pending { color: var(--pending); border-color: var(--pending); background: rgba(155,126,255,0.1); }
  .badge-pulse   { color: var(--pulse);   border-color: var(--pulse);   background: var(--pulse-glow); }
  .badge-muted   { color: var(--ink-muted); border-color: var(--border-strong); background: transparent; }
</style>
