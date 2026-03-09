<script lang="ts">
  import LogoFull from '../brand/LogoFull.svelte';
  import ThemeToggle from '../ui/ThemeToggle.svelte';
  import { page } from '$app/state';

  const links = [
    { href: '/docs', label: 'Docs' },
    { href: '/playground', label: 'Playground' },
  ];

  function isActive(href: string): boolean {
    return page.url.pathname.startsWith(href);
  }
</script>

<nav class="nav">
  <div class="nav-inner">
    <LogoFull size={28} />
    <div class="nav-links">
      {#each links as link}
        <a
          href={link.href}
          class="nav-link"
          class:active={isActive(link.href)}
        >
          {link.label}
        </a>
      {/each}
      <a
        href="https://github.com/virgilvox/freshblu"
        class="nav-link"
        target="_blank"
        rel="noopener"
      >
        <i class="fa-brands fa-github"></i>
        GitHub
      </a>
      <ThemeToggle />
    </div>
  </div>
</nav>

<style>
  .nav {
    background: var(--void-lift);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 100;
  }
  .nav-inner {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 40px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .nav-links {
    display: flex;
    align-items: center;
    gap: 24px;
  }
  .nav-link {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-muted);
    text-decoration: none;
    display: flex;
    align-items: center;
    gap: 6px;
    transition: color var(--dur-fast);
  }
  .nav-link:hover { color: var(--ink); }
  .nav-link.active { color: var(--pulse); }

  @media (max-width: 600px) {
    .nav-inner { padding: 0 16px; }
    .nav-links { gap: 16px; }
  }
</style>
