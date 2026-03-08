<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  interface Props extends HTMLInputAttributes {
    label?: string;
    note?: string;
    value?: string;
  }

  let { label, note, value = $bindable(''), ...rest }: Props = $props();
</script>

<div class="field">
  {#if label}
    <label class="field-label">{label}</label>
  {/if}
  <input class="field-input" bind:value {...rest} />
  {#if note}
    <span class="field-note">{note}</span>
  {/if}
</div>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field-label {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .field-input {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    background: var(--void-high);
    border: 1px solid var(--border-strong);
    color: var(--ink);
    padding: 10px 12px;
    border-radius: var(--r-none);
    outline: none;
    transition: border-color var(--dur-fast), box-shadow var(--dur-fast);
  }
  .field-input::placeholder { color: var(--ink-ghost); }
  .field-input:focus {
    border-color: var(--pulse);
    box-shadow: 0 0 0 2px var(--pulse-ring);
  }
  .field-note {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-muted);
  }
</style>
