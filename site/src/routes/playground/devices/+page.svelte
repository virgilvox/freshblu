<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Toast from '$lib/components/ui/Toast.svelte';
  import DeviceCard from '$lib/components/playground/DeviceCard.svelte';
  import { FreshBluClient, api } from '$lib/api/client';
  import { uuid, token } from '$lib/stores/auth';
  import { devices } from '$lib/stores/devices';
  import { vaultDevices, addToVault, removeFromVault, setActiveDevice } from '$lib/stores/vault';
  import { goto } from '$app/navigation';
  import type { Device } from '$lib/api/types';
  import type { VaultDevice } from '$lib/stores/vault';

  let loading = $state(true);
  let deviceList: Device[] = $state([]);
  let vault: VaultDevice[] = $state([]);
  let toast: Toast;

  const unsubDevices = devices.subscribe(v => deviceList = v);
  const unsubVault = vaultDevices.subscribe(v => vault = v);

  onDestroy(() => {
    unsubDevices();
    unsubVault();
  });

  function isInVault(deviceUuid: string): boolean {
    return vault.some(d => d.uuid === deviceUuid);
  }

  onMount(async () => {
    let u = '', t = '';
    uuid.subscribe(v => u = v)();
    token.subscribe(v => t = v)();
    if (u && t) {
      api.setCredentials(u, t);
      try {
        const me = await api.whoami();
        const mine = await api.myDevices();
        devices.set([me, ...mine.filter(d => d.uuid !== me.uuid)]);
      } catch {
        devices.set([]);
      }
    }
    loading = false;
  });

  async function registerAndVault() {
    const serverUrl = localStorage.getItem('freshblu_server_url') || 'http://localhost:3000';
    const client = new FreshBluClient(serverUrl);
    const res = await client.register();
    const newDevice: Device = {
      uuid: res.uuid,
      online: res.online,
      meshblu: res.meshblu,
    };
    devices.set([...deviceList, newDevice]);
    await addToVault({ uuid: res.uuid, token: res.token, addedAt: Date.now() });
    setActiveDevice(res.uuid);
    uuid.set(res.uuid);
    token.set(res.token);
    api.setCredentials(res.uuid, res.token);
    toast.show('Device registered and added to vault', 'success');
  }

  async function handleRemoveFromVault(deviceUuid: string) {
    await removeFromVault(deviceUuid);
  }

  async function handleAddToVault(device: Device) {
    // We don't have the token for arbitrary API devices, but we can add a stub
    await addToVault({ uuid: device.uuid, token: '', addedAt: Date.now() });
  }

  async function handleDelete(deviceUuid: string) {
    try {
      await api.unregister(deviceUuid);
      devices.set(deviceList.filter(d => d.uuid !== deviceUuid));
      await removeFromVault(deviceUuid);
    } catch { /* ignore */ }
  }

  async function handleGenerateToken(deviceUuid: string) {
    try {
      const res = await api.generateToken(deviceUuid);
      await addToVault({ uuid: res.uuid, token: res.token, addedAt: Date.now() });
      toast.show(`Token generated and added to vault`, 'success');
    } catch (e) {
      toast.show((e as Error).message, 'error');
    }
  }
</script>

<svelte:head>
  <title>Devices - Playground - FreshBlu</title>
</svelte:head>

<div class="devices-page">
  <div class="devices-header">
    <h1 class="page-title">Devices</h1>
    <Button size="sm" onclick={registerAndVault}>
      <i class="fa-solid fa-plus"></i>
      Register & Add to Vault
    </Button>
  </div>

  {#if loading}
    <p class="loading-text">Loading devices...</p>
  {:else if deviceList.length === 0}
    <p class="empty-text">No devices found. Register one to get started.</p>
  {:else}
    <div class="device-grid">
      {#each deviceList as device (device.uuid)}
        <div class="device-card-wrap">
          <div class="card-badges">
            {#if isInVault(device.uuid)}
              <Badge variant="pulse"><i class="fa-solid fa-lock"></i> Vault</Badge>
            {/if}
            <Badge variant={device.online ? 'online' : 'muted'}>{device.online ? 'Online' : 'Offline'}</Badge>
          </div>
          <DeviceCard {device} onclick={() => goto(`/playground/devices/${device.uuid}`)} />
          <div class="card-actions">
            <button class="action-btn" onclick={() => goto(`/playground/devices/${device.uuid}`)} title="View">
              <i class="fa-solid fa-eye"></i>
            </button>
            <button class="action-btn" onclick={() => handleGenerateToken(device.uuid)} title="Generate Token">
              <i class="fa-solid fa-key"></i>
            </button>
            {#if isInVault(device.uuid)}
              <button class="action-btn" onclick={() => handleRemoveFromVault(device.uuid)} title="Remove from Vault">
                <i class="fa-solid fa-lock-open"></i>
              </button>
            {:else}
              <button class="action-btn" onclick={() => handleAddToVault(device)} title="Add to Vault">
                <i class="fa-solid fa-lock"></i>
              </button>
            {/if}
            <button class="action-btn action-delete" onclick={() => handleDelete(device.uuid)} title="Delete">
              <i class="fa-solid fa-trash"></i>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<Toast bind:this={toast} />

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
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
  }
  .device-card-wrap {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  .card-badges {
    display: flex;
    gap: 6px;
    margin-bottom: 6px;
  }
  .card-actions {
    display: flex;
    gap: 4px;
    margin-top: 4px;
  }
  .action-btn {
    flex: 1;
    background: var(--void-lift);
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 4px;
    cursor: pointer;
    font-size: var(--text-xs);
    transition: color var(--dur-fast), border-color var(--dur-fast);
  }
  .action-btn:hover {
    color: var(--pulse);
    border-color: var(--pulse);
  }
  .action-delete:hover {
    color: var(--fault);
    border-color: var(--fault);
  }
  .loading-text, .empty-text {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    color: var(--ink-muted);
  }
</style>
