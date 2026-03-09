<script lang="ts">
  import { onMount } from 'svelte';

  let dark = $state(true);

  onMount(() => {
    dark = document.documentElement.getAttribute('data-theme') !== 'light';
  });

  function toggle() {
    dark = !dark;
    if (dark) {
      document.documentElement.removeAttribute('data-theme');
      localStorage.removeItem('freshblu_theme');
    } else {
      document.documentElement.setAttribute('data-theme', 'light');
      localStorage.setItem('freshblu_theme', 'light');
    }
  }
</script>

<button class="theme-toggle" onclick={toggle} title={dark ? 'Switch to light mode' : 'Switch to dark mode'}>
  {#if dark}
    <i class="fa-solid fa-sun"></i>
  {:else}
    <i class="fa-solid fa-moon"></i>
  {/if}
</button>

<style>
  .theme-toggle {
    background: none;
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 4px 8px;
    cursor: pointer;
    font-size: var(--text-sm);
    display: flex;
    align-items: center;
    transition: color var(--dur-fast), border-color var(--dur-fast);
  }
  .theme-toggle:hover {
    color: var(--pulse);
    border-color: var(--pulse);
  }
</style>
