<script lang="ts">
  import { page } from '$app/state';

  interface NavSection {
    title: string;
    items: { href: string; label: string }[];
  }

  interface Props {
    sections: NavSection[];
  }

  let { sections }: Props = $props();
</script>

<aside class="sidebar">
  {#each sections as section}
    <div class="sidebar-section">
      <span class="sidebar-heading">{section.title}</span>
      <ul class="sidebar-list">
        {#each section.items as item}
          <li>
            <a
              href={item.href}
              class="sidebar-link"
              class:active={page.url.pathname === item.href}
            >
              {item.label}
            </a>
          </li>
        {/each}
      </ul>
    </div>
  {/each}
</aside>

<style>
  .sidebar {
    width: 240px;
    flex-shrink: 0;
    padding: 24px 0;
    border-right: 1px solid var(--border);
    position: sticky;
    top: 56px;
    height: calc(100vh - 56px);
    overflow-y: auto;
  }
  .sidebar-section { margin-bottom: 24px; }
  .sidebar-heading {
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--pulse);
    padding: 0 20px;
    display: block;
    margin-bottom: 8px;
  }
  .sidebar-list {
    list-style: none;
  }
  .sidebar-link {
    display: block;
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-muted);
    padding: 5px 20px;
    text-decoration: none;
    transition: color var(--dur-fast), background var(--dur-fast);
    border-left: 2px solid transparent;
  }
  .sidebar-link:hover {
    color: var(--ink-soft);
    background: var(--void-lift);
  }
  .sidebar-link.active {
    color: var(--pulse);
    border-left-color: var(--pulse);
    background: var(--pulse-glow);
  }

  @media (max-width: 900px) {
    .sidebar {
      width: 100%;
      position: static;
      height: auto;
      border-right: none;
      border-bottom: 1px solid var(--border);
    }
    .sidebar-list {
      display: flex;
      flex-wrap: wrap;
      gap: 0;
      padding: 0 12px;
    }
    .sidebar-link {
      border-left: none;
      border-bottom: 2px solid transparent;
      padding: 5px 10px;
    }
    .sidebar-link.active {
      border-left-color: transparent;
      border-bottom-color: var(--pulse);
    }
  }
</style>
