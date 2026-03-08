<script lang="ts">
  import Logo from './Logo.svelte';

  interface Props {
    size?: number;
  }

  let { size = 32 }: Props = $props();
</script>

<a href="/" class="logo-full">
  <Logo {size} />
  <span class="logo-text">FRESH<span class="logo-accent">BLU</span></span>
</a>

<style>
  .logo-full {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    text-decoration: none;
    color: var(--ink);
  }
  .logo-text {
    font-family: var(--font-display);
    font-size: var(--text-lg);
    font-weight: 700;
    letter-spacing: 0.08em;
  }
  .logo-accent { color: var(--pulse); }
</style>
