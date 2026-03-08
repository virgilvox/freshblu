<script lang="ts">
  import type { Snippet } from 'svelte';
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import VaultSwitcher from '$lib/components/playground/VaultSwitcher.svelte';
  import { migrateToVault } from '$lib/stores/auth';

  let { children }: { children: Snippet } = $props();

  onMount(() => {
    migrateToVault();
  });

  const navItems = [
    { href: '/playground', label: 'Tester', icon: 'fa-flask-vial' },
    { href: '/playground/devices', label: 'Devices', icon: 'fa-microchip' },
    { href: '/playground/api', label: 'API Explorer', icon: 'fa-code' },
    { href: '/playground/visualizer', label: 'Visualizer', icon: 'fa-diagram-project' },
  ];
</script>

<div class="playground-layout">
  <div class="playground-nav">
    <div class="playground-nav-inner">
      <span class="playground-title">
        <i class="fa-solid fa-terminal"></i>
        Playground
      </span>
      <VaultSwitcher />
      <div class="playground-links">
        {#each navItems as item}
          <a
            href={item.href}
            class="playground-link"
            class:active={page.url.pathname === item.href || (item.href !== '/playground' && page.url.pathname.startsWith(item.href))}
          >
            <i class="fa-solid {item.icon}"></i>
            {item.label}
          </a>
        {/each}
      </div>
    </div>
  </div>
  <div class="playground-content">
    {@render children()}
  </div>
</div>

<style>
  .playground-layout {
    max-width: 1400px;
    margin: 0 auto;
  }
  .playground-nav {
    border-bottom: 1px solid var(--border);
    background: var(--void-lift);
  }
  .playground-nav-inner {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 0 40px;
    height: 44px;
  }
  .playground-title {
    font-family: var(--font-display);
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--signal);
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .playground-links {
    display: flex;
    gap: 16px;
    margin-left: auto;
    overflow-x: auto;
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
  .playground-links::-webkit-scrollbar { display: none; }
  .playground-link {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-muted);
    text-decoration: none;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 0;
    border-bottom: 2px solid transparent;
    transition: color var(--dur-fast), border-color var(--dur-fast);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .playground-link:hover { color: var(--ink-soft); }
  .playground-link.active {
    color: var(--signal);
    border-bottom-color: var(--signal);
  }
  .playground-content {
    padding: 40px;
  }

  @media (max-width: 768px) {
    .playground-nav-inner {
      padding: 0 16px;
      flex-wrap: wrap;
      height: auto;
      padding-top: 8px;
      padding-bottom: 8px;
      gap: 8px;
    }
    .playground-links {
      width: 100%;
      margin-left: 0;
      gap: 12px;
    }
    .playground-content { padding: 24px 16px; }
  }
</style>
