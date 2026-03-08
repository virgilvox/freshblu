<script lang="ts">
  import type { Snippet } from 'svelte';
  import { page } from '$app/state';

  let { children }: { children: Snippet } = $props();

  const navItems = [
    { href: '/playground', label: 'Connect', icon: 'fa-plug' },
    { href: '/playground/devices', label: 'Devices', icon: 'fa-microchip' },
    { href: '/playground/messages', label: 'Messages', icon: 'fa-paper-plane' },
  ];
</script>

<div class="playground-layout">
  <div class="playground-nav">
    <div class="playground-nav-inner">
      <span class="playground-title">
        <i class="fa-solid fa-terminal"></i>
        Playground
      </span>
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
    max-width: 1200px;
    margin: 0 auto;
  }
  .playground-nav {
    border-bottom: 1px solid var(--border);
    background: var(--void-lift);
  }
  .playground-nav-inner {
    display: flex;
    align-items: center;
    gap: 24px;
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
  }
  .playground-links {
    display: flex;
    gap: 16px;
    margin-left: auto;
  }
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
  }
  .playground-link:hover { color: var(--ink-soft); }
  .playground-link.active {
    color: var(--signal);
    border-bottom-color: var(--signal);
  }
  .playground-content {
    padding: 40px;
  }

  @media (max-width: 600px) {
    .playground-nav-inner { padding: 0 16px; }
    .playground-content { padding: 24px 16px; }
  }
</style>
