<script lang="ts">
  import StatusDot from '../ui/StatusDot.svelte';
  import type { Device } from '$lib/api/types';

  interface Props {
    device: Device;
    onclick?: () => void;
  }

  let { device, onclick }: Props = $props();

  const icons: Record<string, string> = {
    microchip: 'fa-microchip',
    lightbulb: 'fa-lightbulb',
    temperature: 'fa-temperature-half',
    fan: 'fa-fan',
    lock: 'fa-lock',
    camera: 'fa-camera',
    wifi: 'fa-wifi',
    satellite: 'fa-satellite-dish',
    gauge: 'fa-gauge',
    robot: 'fa-robot',
    plug: 'fa-plug',
    server: 'fa-server',
  };

  function getIcon(d: Device): string {
    const icon = d.icon as string | undefined;
    return icons[icon || ''] || 'fa-microchip';
  }

  function truncateUuid(uuid: string): string {
    return uuid.substring(0, 8) + '...';
  }
</script>

<button class="device-card" {onclick}>
  <div class="device-header">
    <i class="fa-solid {getIcon(device)} device-icon"></i>
    <StatusDot status={device.online ? 'online' : 'fault'} />
  </div>
  <div class="device-uuid">{truncateUuid(device.uuid)}</div>
  <div class="device-name">{device.name || 'Unnamed Device'}</div>
  {#if device.type}
    <div class="device-type">{device.type}</div>
  {/if}
</button>

<style>
  .device-card {
    background: var(--void-high);
    border: 1px solid var(--border);
    padding: 16px;
    cursor: pointer;
    text-align: left;
    transition: transform var(--dur-med) var(--ease-snap), border-color var(--dur-med);
    width: 100%;
  }
  .device-card:hover {
    transform: translateY(-2px);
    border-color: var(--border-pulse);
  }
  .device-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }
  .device-icon {
    font-size: var(--text-xl);
    color: var(--pulse);
  }
  .device-uuid {
    font-family: var(--font-ui);
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--pulse);
    margin-bottom: 8px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .device-name {
    font-family: var(--font-display);
    font-size: var(--text-md);
    font-weight: 700;
    letter-spacing: 0.04em;
    margin-bottom: 4px;
    color: var(--ink);
  }
  .device-type {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
</style>
