<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Toast from '$lib/components/ui/Toast.svelte';
  import DeviceCard from '$lib/components/playground/DeviceCard.svelte';
  import RegisterDeviceModal from '$lib/components/playground/RegisterDeviceModal.svelte';
  import { FreshBluHttp, api, syncApiBaseUrl, getServerUrl } from '$lib/api/client';
  import { uuid, token } from '$lib/stores/auth';
  import { vaultDevices, addToVault, removeFromVault, setActiveDevice } from '$lib/stores/vault';
  import { goto } from '$app/navigation';
  import type { Device } from '$lib/api/client';
  import type { VaultDevice } from '$lib/stores/vault';

  let loading = $state(true);
  let vault: VaultDevice[] = $state([]);
  let liveStatus: Map<string, { online: boolean; device?: Device }> = $state(new Map());
  let toast: Toast;
  let showMyDevices = $state(false);
  let myDevicesList: Device[] = $state([]);
  let loadingMyDevices = $state(false);
  let showRegisterModal = $state(false);

  // Active device credentials (reactive)
  let activeUuid = $state('');
  let activeToken = $state('');
  const unsubUuid = uuid.subscribe(v => activeUuid = v);
  const unsubToken = token.subscribe(v => activeToken = v);
  const unsubVault = vaultDevices.subscribe(v => vault = v);

  onDestroy(() => {
    unsubVault();
    unsubUuid();
    unsubToken();
  });

  /** Find label for the active device */
  function activeDeviceLabel(): string | undefined {
    const vd = vault.find(d => d.uuid === activeUuid);
    return vd?.name || vd?.label || vd?.type;
  }

  onMount(async () => {
    if (activeUuid && activeToken) {
      syncApiBaseUrl();
      api.setCredentials(activeUuid, activeToken);
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
        const client = new FreshBluHttp(getServerUrl());
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

  async function handleRegister(properties: Record<string, unknown>, claimOwnership: boolean) {
    const serverUrl = getServerUrl();
    const client = new FreshBluHttp(serverUrl);
    try {
      const res = await client.register(Object.keys(properties).length > 0 ? properties : undefined);

      // Store in vault with name/type for VaultSwitcher display
      await addToVault({
        uuid: res.uuid,
        token: res.token,
        name: properties.name as string | undefined,
        type: properties.type as string | undefined,
        addedAt: Date.now(),
      });

      // If this is the first device, make it the primary
      if (vault.length === 0 || !activeUuid) {
        setActiveDevice(res.uuid);
        uuid.set(res.uuid);
        token.set(res.token);
        syncApiBaseUrl();
        api.setCredentials(res.uuid, res.token);
      }

      // Claim ownership: the active device claims the new device
      if (claimOwnership && activeUuid && activeUuid !== res.uuid) {
        try {
          await api.claimDevice(res.uuid);
        } catch {
          // Claiming may fail if permissions aren't set up -- not fatal
        }
      }

      liveStatus.set(res.uuid, { online: res.online, device: { uuid: res.uuid, online: res.online, meshblu: res.meshblu, ...properties } as Device });
      liveStatus = new Map(liveStatus);
      showRegisterModal = false;
      toast.show(`Registered ${properties.name || res.uuid.substring(0, 8)}`, 'success');
    } catch (e) {
      toast.show((e as Error).message, 'error');
    }
  }

  async function handleRemoveFromVault(deviceUuid: string) {
    if (!confirm('Remove this device from vault? You will lose the stored token.')) return;
    await removeFromVault(deviceUuid);
    liveStatus.delete(deviceUuid);
    liveStatus = new Map(liveStatus);
  }

  async function handleDelete(deviceUuid: string) {
    if (!confirm('Delete this device permanently? This cannot be undone.')) return;
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

  async function toggleMyDevices() {
    showMyDevices = !showMyDevices;
    if (showMyDevices && myDevicesList.length === 0) {
      loadingMyDevices = true;
      try {
        myDevicesList = await api.myDevices();
      } catch (e) {
        toast.show((e as Error).message, 'error');
        showMyDevices = false;
      }
      loadingMyDevices = false;
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
    <div class="header-actions">
      <Button size="sm" variant={showMyDevices ? 'pulse' : 'ghost'} onclick={toggleMyDevices}>
        <i class="fa-solid fa-user"></i>
        My Devices
      </Button>
      <Button size="sm" onclick={() => showRegisterModal = true}>
        <i class="fa-solid fa-plus"></i>
        Register New Device
      </Button>
    </div>
  </div>

  {#if showMyDevices}
    <div class="my-devices-section">
      <h2 class="section-title">Server-Owned Devices</h2>
      {#if loadingMyDevices}
        <p class="loading-text">Loading owned devices...</p>
      {:else if myDevicesList.length === 0}
        <p class="empty-text">No owned devices found. Claim devices or register with ownership enabled.</p>
      {:else}
        <div class="device-grid">
          {#each myDevicesList as device (device.uuid)}
            <div class="device-card-wrap">
              <div class="card-badges">
                <Badge variant={device.online ? 'online' : 'muted'}>{device.online ? 'Online' : 'Offline'}</Badge>
                {#if vault.some(v => v.uuid === device.uuid)}
                  <Badge variant="pulse"><i class="fa-solid fa-lock"></i> Vault</Badge>
                {/if}
              </div>
              <DeviceCard {device} onclick={() => goto(`/playground/devices/${device.uuid}`)} />
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  {#if loading}
    <p class="loading-text">Loading devices...</p>
  {:else if vault.length === 0}
    <div class="empty-state">
      <p class="empty-text">No devices in vault. Register one to get started.</p>
      <Button size="sm" onclick={() => showRegisterModal = true}>
        <i class="fa-solid fa-plus"></i>
        Register Your First Device
      </Button>
    </div>
  {:else}
    <div class="device-grid">
      {#each vault as vd (vd.uuid)}
        {@const device = toDevice(vd)}
        {@const isPrimary = vd.uuid === activeUuid}
        <div class="device-card-wrap">
          <div class="card-badges">
            {#if isPrimary}
              <Badge variant="pulse"><i class="fa-solid fa-star"></i> Primary</Badge>
            {:else}
              <Badge variant="pulse"><i class="fa-solid fa-lock"></i> Vault</Badge>
            {/if}
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

<RegisterDeviceModal
  bind:open={showRegisterModal}
  ownerUuid={activeUuid || undefined}
  ownerLabel={activeDeviceLabel()}
  onregister={handleRegister}
/>

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
  .header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .my-devices-section {
    margin-bottom: 32px;
    padding-bottom: 24px;
    border-bottom: 1px solid var(--border);
  }
  .section-title {
    font-family: var(--font-display);
    font-size: var(--text-lg);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    margin-bottom: 16px;
  }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }
  .loading-text, .empty-text {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    color: var(--ink-muted);
  }
</style>
