<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import DeviceCard from '$lib/components/playground/DeviceCard.svelte';
  import { api } from '$lib/api/client';
  import { uuid, token } from '$lib/stores/auth';
  import { devices } from '$lib/stores/devices';
  import { goto } from '$app/navigation';
  import type { Device } from '$lib/api/types';

  let loading = $state(true);
  let deviceList: Device[] = $state([]);

  devices.subscribe(v => deviceList = v);

  onMount(async () => {
    let u = '', t = '';
    uuid.subscribe(v => u = v)();
    token.subscribe(v => t = v)();
    if (!u || !t) {
      goto('/playground');
      return;
    }
    api.setCredentials(u, t);
    try {
      const me = await api.whoami();
      const mine = await api.myDevices();
      devices.set([me, ...mine.filter(d => d.uuid !== me.uuid)]);
    } catch {
      devices.set([]);
    }
    loading = false;
  });

  async function registerNew() {
    const res = await api.register();
    const newDevice: Device = {
      uuid: res.uuid,
      online: res.online,
      meshblu: res.meshblu,
    };
    devices.set([...deviceList, newDevice]);
  }
</script>

<svelte:head>
  <title>Devices - Playground - FreshBlu</title>
</svelte:head>

<div class="devices-page">
  <div class="devices-header">
    <h1 class="page-title">Devices</h1>
    <Button size="sm" onclick={registerNew}>
      <i class="fa-solid fa-plus"></i>
      Register Device
    </Button>
  </div>

  {#if loading}
    <p class="loading-text">Loading devices...</p>
  {:else if deviceList.length === 0}
    <p class="empty-text">No devices found. Register one to get started.</p>
  {:else}
    <div class="device-grid">
      {#each deviceList as device (device.uuid)}
        <DeviceCard {device} onclick={() => goto(`/playground/devices/${device.uuid}`)} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .devices-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 32px;
  }
  .page-title {
    font-family: var(--font-display);
    font-size: var(--text-2xl);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .device-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 12px;
  }
  .loading-text, .empty-text {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    color: var(--ink-muted);
  }
</style>
