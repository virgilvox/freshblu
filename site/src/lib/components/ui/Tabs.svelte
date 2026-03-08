<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    tabs: string[];
    active?: string;
    onchange?: (tab: string) => void;
    children: Snippet;
  }

  let { tabs, active = $bindable(tabs[0]), onchange, children }: Props = $props();

  function select(tab: string) {
    active = tab;
    onchange?.(tab);
  }

  function handleKeydown(e: KeyboardEvent) {
    const idx = tabs.indexOf(active);
    let next = -1;
    if (e.key === 'ArrowRight') next = (idx + 1) % tabs.length;
    else if (e.key === 'ArrowLeft') next = (idx - 1 + tabs.length) % tabs.length;
    if (next >= 0) {
      e.preventDefault();
      select(tabs[next]);
      const btn = (e.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>('.tab-btn')[next];
      btn?.focus();
    }
  }
</script>

<div class="tabs-container">
  <div class="tabs-bar" role="tablist" onkeydown={handleKeydown}>
    {#each tabs as tab}
      <button
        class="tab-btn"
        class:active={active === tab}
        role="tab"
        aria-selected={active === tab}
        tabindex={active === tab ? 0 : -1}
        onclick={() => select(tab)}
      >
        {tab}
      </button>
    {/each}
  </div>
  <div class="tabs-panel" role="tabpanel">
    {@render children()}
  </div>
</div>

<style>
  .tabs-container {
    border: 1px solid var(--border);
  }
  .tabs-bar {
    display: flex;
    border-bottom: 1px solid var(--border);
    background: var(--void-lift);
    overflow-x: auto;
  }
  .tab-btn {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-muted);
    background: none;
    border: none;
    padding: 10px 16px;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: color var(--dur-fast), border-color var(--dur-fast);
    white-space: nowrap;
  }
  .tab-btn:hover { color: var(--ink-soft); }
  .tab-btn.active {
    color: var(--pulse);
    border-bottom-color: var(--pulse);
  }
  .tabs-panel { padding: 20px; }
</style>
