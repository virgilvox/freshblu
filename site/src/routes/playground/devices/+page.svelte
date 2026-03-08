<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Toast from '$lib/components/ui/Toast.svelte';
  import DeviceCard from '$lib/components/playground/DeviceCard.svelte';
  import { FreshBluClient, api, syncApiBaseUrl, getServerUrl } from '$lib/api/client';
  import { uuid, token } from '$lib/stores/auth';
  import { vaultDevices, addToVault, removeFromVault, setActiveDevice } from '$lib/stores/vault';
  import { goto } from '$app/navigation';
  import type { Device } from '$lib/api/types';
  import type { VaultDevice } from '$lib/stores/vault';

  let loading = $state(true);
  let vault: VaultDevice[] = $state([]);
  let liveStatus: Map<string, { online: boolean; device?: Device }> = $state(new Map());
  let toast: Toast;

  const unsubVault = vaultDevices.subscribe(v => vault = v);

  onDestroy(() => {
    unsubVault();
  });

  onMount(async () => {
    // Fetch live status for each vault device
    let u = '', t = '';
    uuid.subscribe(v => u = v)();
    token.subscribe(v => t = v)();
    if (u && t) {
      syncApiBaseUrl();
      api.setCredentials(u, t);
    }
    await refreshStatus();
    loading = false;
  });

  async function refreshStatus() {
    const statuses = new Map<string, { online: boolean; device?: Device }>();
    for (const vd of vault) {
      if (!vd.token) {
        statuses.set(vd.uuid, { online: false });
        continue;
      }
      try {
        const client = new FreshBluClient(getServerUrl());
        client.setCredentials(vd.uuid, vd.token);
        const device = await client.getDevice(vd.uuid);
        statuses.set(vd.uuid, { online: device.online, device });
      } catch {
        statuses.set(vd.uuid, { online: false });
      }
    }
    liveStatus = statuses;
  }

  function toDevice(vd: VaultDevice): Device {
    const live = liveStatus.get(vd.uuid);
    if (live?.device) return live.device;
    return {
      uuid: vd.uuid,
      online: live?.online ?? false,
      type: vd.type,
      name: vd.name,
      meshblu: { version: '2.0.0', createdAt: '', hash: '', whitelists: { discover: { view: [], as: [] }, configure: { update: [], sent: [], received: [], as: [] }, message: { from: [], sent: [], received: [], as: [] }, broadcast: { sent: [], received: [], as: [] } } },
    };
  }

  async function registerAndVault() {
    const serverUrl = getServerUrl();
    const client = new FreshBluClient(serverUrl);
    try {
      const res = await client.register();
      await addToVault({ uuid: res.uuid, token: res.token, addedAt: Date.now() });
      setActiveDevice(res.uuid);
      uuid.set(res.uuid);
      token.set(res.token);
      syncApiBaseUrl();
      api.setCredentials(res.uuid, res.token);
      liveStatus.set(res.uuid, { online: res.online, device: { uuid: res.uuid, online: res.online, meshblu: res.meshblu } });
      liveStatus = new Map(liveStatus);
      toast.show('Device registered and added to vault', 'success');
    } catch (e) {
      toast.show((e as Error).message, 'error');
    }
  }

  async function handleRemoveFromVault(deviceUuid: string) {
    await removeFromVault(deviceUuid);
    liveStatus.delete(deviceUuid);
    liveStatus = new Map(liveStatus);
  }

  async function handleDelete(deviceUuid: string) {
    try {
      await api.unregister(deviceUuid);
      await removeFromVault(deviceUuid);
      liveStatus.delete(deviceUuid);
      liveStatus = new Map(liveStatus);
      toast.show('Device deleted', 'success');
    } catch (e) {
      toast.show((e as Error).message, 'error');
    }
  }

  async function handleGenerateToken(deviceUuid: string) {
    try {
      const res = await api.generateToken(deviceUuid);
      await addToVault({ uuid: res.uuid, token: res.token, addedAt: Date.now() });
      toast.show('Token generated and saved to vault', 'success');
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
      Register New Device
    </Button>
  </div>

  {#if loading}
    <p class="loading-text">Loading devices...</p>
  {:else if vault.length === 0}
    <p class="empty-text">No devices in vault. Register one to get started.</p>
  {:else}
    <div class="device-grid">
      {#each vault as vd (vd.uuid)}
        {@const device = toDevice(vd)}
        <div class="device-card-wrap">
          <div class="card-badges">
            <Badge variant="pulse"><i class="fa-solid fa-lock"></i> Vault</Badge>
            {#if !vd.token}
              <Badge variant="warn">No Token</Badge>
            {:else}
              <Badge variant={device.online ? 'online' : 'muted'}>{device.online ? 'Online' : 'Offline'}</Badge>
            {/if}
          </div>
          <DeviceCard {device} onclick={() => goto(`/playground/devices/${device.uuid}`)} />
          <div class="card-actions">
            <button class="action-btn" onclick={() => goto(`/playground/devices/${device.uuid}`)} title="View">
              <i class="fa-solid fa-eye"></i>
            </button>
            <button class="action-btn" onclick={() => handleGenerateToken(device.uuid)} title="Generate Token">
              <i class="fa-solid fa-key"></i>
            </button>
            <button class="action-btn" onclick={() => handleRemoveFromVault(device.uuid)} title="Remove from Vault">
              <i class="fa-solid fa-lock-open"></i>
            </button>
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
