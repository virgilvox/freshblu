<script lang="ts">
  import CodeBlock from './CodeBlock.svelte';

  interface Tab {
    label: string;
    lang: string;
    code: string;
  }

  interface Props {
    tabs: Tab[];
  }

  let { tabs }: Props = $props();
  let active = $state(0);
</script>

<div class="lang-tabs">
  <div class="tab-row">
    {#each tabs as tab, i}
      <button
        class="tab-btn"
        class:active={active === i}
        onclick={() => active = i}
      >
        {tab.label}
      </button>
    {/each}
  </div>
  <CodeBlock lang={tabs[active].lang} code={tabs[active].code} />
</div>

<style>
  .lang-tabs {
    display: flex;
    flex-direction: column;
  }
  .tab-row {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--border);
  }
  .tab-btn {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-muted);
    background: none;
    border: none;
    padding: 8px 16px;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: color var(--dur-fast), border-color var(--dur-fast);
  }
  .tab-btn:hover {
    color: var(--ink-soft);
  }
  .tab-btn.active {
    color: var(--pulse);
    border-bottom-color: var(--pulse);
  }
</style>
