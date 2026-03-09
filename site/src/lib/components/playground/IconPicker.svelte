<script lang="ts">
  interface Props {
    selected: string;
    onSelect: (icon: string) => void;
  }

  let { selected, onSelect }: Props = $props();
  let open = $state(false);

  const icons = [
    { key: 'microchip', icon: 'fa-microchip' },
    { key: 'lightbulb', icon: 'fa-lightbulb' },
    { key: 'temperature', icon: 'fa-temperature-half' },
    { key: 'fan', icon: 'fa-fan' },
    { key: 'lock', icon: 'fa-lock' },
    { key: 'camera', icon: 'fa-camera' },
    { key: 'wifi', icon: 'fa-wifi' },
    { key: 'satellite', icon: 'fa-satellite-dish' },
    { key: 'gauge', icon: 'fa-gauge' },
    { key: 'robot', icon: 'fa-robot' },
    { key: 'plug', icon: 'fa-plug' },
    { key: 'server', icon: 'fa-server' },
  ];

  function getIconClass(key: string): string {
    return icons.find(i => i.key === key)?.icon || 'fa-microchip';
  }

  function handleSelect(key: string) {
    onSelect(key);
    open = false;
  }
</script>

<div class="icon-picker">
  <button class="icon-trigger" onclick={() => open = !open} type="button">
    <i class="fa-solid {getIconClass(selected)} trigger-icon"></i>
    <span class="trigger-label">{selected || 'Select icon'}</span>
    <i class="fa-solid fa-chevron-down trigger-chevron" class:open></i>
  </button>
  {#if open}
    <div class="icon-grid">
      {#each icons as { key, icon }}
        <button
          class="icon-btn"
          class:active={selected === key}
          onclick={() => handleSelect(key)}
          title={key}
          type="button"
        >
          <i class="fa-solid {icon}"></i>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .icon-picker {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .icon-trigger {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--void);
    border: 1px solid var(--border);
    color: var(--ink-soft);
    padding: 6px 12px;
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    transition: border-color var(--dur-fast);
    width: fit-content;
  }
  .icon-trigger:hover { border-color: var(--pulse); }
  .trigger-icon { color: var(--pulse); font-size: var(--text-sm); }
  .trigger-label { text-transform: uppercase; color: var(--ink-muted); }
  .trigger-chevron {
    font-size: 8px;
    margin-left: 4px;
    transition: transform var(--dur-fast);
  }
  .trigger-chevron.open { transform: rotate(180deg); }
  .icon-grid {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 4px;
  }
  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    background: var(--void);
    border: 1px solid var(--border);
    color: var(--ink-muted);
    cursor: pointer;
    font-size: var(--text-md);
    transition: all var(--dur-fast);
  }
  .icon-btn:hover {
    color: var(--pulse);
    border-color: var(--pulse);
  }
  .icon-btn.active {
    color: var(--pulse);
    border-color: var(--pulse);
    background: var(--pulse-glow);
  }
</style>
