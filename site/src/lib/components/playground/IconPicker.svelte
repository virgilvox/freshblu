<script lang="ts">
  interface Props {
    selected: string;
    onSelect: (icon: string) => void;
  }

  let { selected, onSelect }: Props = $props();

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
</script>

<div class="icon-picker">
  {#each icons as { key, icon }}
    <button
      class="icon-btn"
      class:active={selected === key}
      onclick={() => onSelect(key)}
      title={key}
    >
      <i class="fa-solid {icon}"></i>
    </button>
  {/each}
</div>

<style>
  .icon-picker {
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
