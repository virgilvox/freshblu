<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface Props extends HTMLButtonAttributes {
    variant?: 'primary' | 'ghost' | 'muted' | 'signal';
    size?: 'sm' | 'md' | 'lg';
    href?: string;
    children: Snippet;
  }

  let { variant = 'primary', size = 'md', href, children, ...rest }: Props = $props();
</script>

{#if href}
  <a {href} class="btn btn-{variant} btn-{size}" {...rest}>
    {@render children()}
  </a>
{:else}
  <button class="btn btn-{variant} btn-{size}" {...rest}>
    {@render children()}
  </button>
{/if}

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-display);
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    padding: 10px 20px;
    border: 2px solid;
    border-radius: var(--r-none);
    cursor: pointer;
    text-decoration: none;
    transition: transform var(--dur-fast) var(--ease-snap),
                box-shadow var(--dur-fast) var(--ease-snap),
                background var(--dur-fast);
  }

  .btn:active { transform: translateY(0); box-shadow: none; }

  .btn-primary {
    background: var(--pulse);
    color: #fff;
    border-color: var(--pulse);
  }
  .btn-primary:hover {
    background: var(--pulse-up);
    border-color: var(--pulse-up);
    transform: translateY(-2px);
    box-shadow: var(--shadow-pulse);
  }

  .btn-ghost {
    background: transparent;
    color: var(--pulse);
    border-color: var(--pulse);
  }
  .btn-ghost:hover {
    background: var(--pulse-glow);
    transform: translateY(-2px);
    box-shadow: var(--shadow-pulse);
  }

  .btn-muted {
    background: transparent;
    color: var(--ink-soft);
    border-color: var(--border-strong);
  }
  .btn-muted:hover {
    color: var(--ink);
    border-color: var(--ink-soft);
    transform: translateY(-1px);
  }

  .btn-signal {
    background: var(--signal-dim);
    color: var(--signal);
    border-color: var(--signal);
  }
  .btn-signal:hover {
    background: rgba(0,207,255,0.15);
    transform: translateY(-2px);
    box-shadow: 3px 3px 0 rgba(0,207,255,0.4);
  }

  .btn-sm { font-size: var(--text-xs); padding: 6px 12px; }
  .btn-md { font-size: var(--text-sm); padding: 10px 20px; }
  .btn-lg { font-size: var(--text-base); padding: 14px 28px; }
</style>
