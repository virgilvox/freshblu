<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import Tabs from '$lib/components/ui/Tabs.svelte';
  import CredentialsPanel from '$lib/components/playground/CredentialsPanel.svelte';
  import PropertyEditor from '$lib/components/playground/PropertyEditor.svelte';
  import WhitelistEditor from '$lib/components/playground/WhitelistEditor.svelte';
  import WebhookEditor from '$lib/components/playground/WebhookEditor.svelte';
  import IconPicker from '$lib/components/playground/IconPicker.svelte';
  import StatusDot from '$lib/components/ui/StatusDot.svelte';
  import { api, syncApiBaseUrl } from '$lib/api/client';
  import { uuid as authUuid, token as authToken } from '$lib/stores/auth';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { vaultDevices } from '$lib/stores/vault';
  import type { Device, Whitelists, Forwarders } from '$lib/api/client';
  import type { VaultDevice } from '$lib/stores/vault';

  let vault: VaultDevice[] = $state([]);
  let currentAuthUuid = $state('');
  const unsubVault = vaultDevices.subscribe(v => vault = v);
  const unsubAuth = authUuid.subscribe(v => currentAuthUuid = v);

  onDestroy(() => {
    unsubVault();
    unsubAuth();
  });

  function isInVault(deviceUuid: string): boolean {
    return vault.some(d => d.uuid === deviceUuid);
  }

  function getVaultToken(deviceUuid: string): string | undefined {
    return vault.find(d => d.uuid === deviceUuid)?.token;
  }

  const tabs = ['Properties', 'Credentials', 'Permissions', 'Webhooks'];
  let activeTab = $state(tabs[0]);
  let device: Device | null = $state(null);
  let loading = $state(true);
  let errorMessage = $state('');

  const deviceUuid = page.params.uuid;

  async function loadDevice() {
    loading = true;
    errorMessage = '';
    let u = '', t = '';
    authUuid.subscribe(v => u = v)();
    authToken.subscribe(v => t = v)();
    syncApiBaseUrl();
    api.setCredentials(u, t);
    try {
      device = await api.getDevice(deviceUuid);
    } catch (e) {
      device = null;
      const msg = (e as Error).message || 'Unknown error';
      if (msg.includes('404') || msg.toLowerCase().includes('not found')) {
        errorMessage = 'Device not found. It may have been deleted.';
      } else if (msg.includes('403') || msg.toLowerCase().includes('permission') || msg.toLowerCase().includes('forbidden')) {
        errorMessage = 'Permission denied. You do not have access to view this device.';
      } else {
        errorMessage = msg;
      }
    }
    loading = false;
  }

  onMount(loadDevice);

  async function saveProperties(props: Record<string, unknown>) {
    device = await api.updateDevice(deviceUuid, props);
  }

  async function saveWhitelists(whitelists: Whitelists) {
    device = await api.updateDevice(deviceUuid, { meshblu: { whitelists } });
  }

  async function saveForwarders(forwarders: Forwarders) {
    device = await api.updateDevice(deviceUuid, { meshblu: { forwarders } });
  }

  async function setIcon(icon: string) {
    device = await api.updateDevice(deviceUuid, { icon });
  }
</script>

<svelte:head>
  <title>{deviceUuid.substring(0, 8)}... - Playground - FreshBlu</title>
</svelte:head>

{#if loading}
  <p class="loading-text">Loading device...</p>
{:else if !device}
  <div class="error-state">
    <p class="error-text">{errorMessage || 'Device not found.'}</p>
    {#if isInVault(deviceUuid)}
      {@const vt = getVaultToken(deviceUuid)}
      {#if vt}
        <p class="error-hint">This device is in your vault. Try re-authenticating with its token.</p>
      {/if}
    {/if}
    <div class="error-actions">
      <Button size="sm" variant="ghost" onclick={loadDevice}>Retry</Button>
      <Button size="sm" variant="muted" onclick={() => goto('/playground/devices')}>Back to Devices</Button>
    </div>
  </div>
{:else}
  <div class="device-detail">
    <div class="detail-header">
      <div class="detail-info">
        <StatusDot status={device.online ? 'online' : 'fault'} />
        <h1 class="detail-uuid">{device.uuid}</h1>
        {#if isInVault(device.uuid)}
          <Badge variant="pulse"><i class="fa-solid fa-lock"></i> In Vault</Badge>
        {/if}
      </div>
      <div class="detail-meta">
        {#if device.type}
          <span class="detail-type">{device.type}</span>
        {/if}
        {#if device.name}
          <span class="detail-name">{device.name}</span>
        {/if}
        {#if device.meshblu?.owner}
          {@const ownerUuid = device.meshblu.owner}
          <Badge variant={ownerUuid === currentAuthUuid ? 'online' : 'muted'}>
            {ownerUuid === currentAuthUuid ? 'Owned by you' : `Owner: ${ownerUuid.substring(0, 8)}...`}
          </Badge>
        {/if}
      </div>
    </div>

    <div class="icon-section">
      <span class="section-label">Device Icon</span>
      <IconPicker selected={device.icon as string || ''} onSelect={setIcon} />
    </div>

    <Tabs {tabs} bind:active={activeTab}>
      {#if activeTab === 'Properties'}
        <PropertyEditor
          properties={Object.fromEntries(
            Object.entries(device).filter(([k]) => !['uuid', 'online', 'meshblu', 'token'].includes(k))
          )}
          onSave={saveProperties}
        />
      {:else if activeTab === 'Credentials'}
        {@const vaultToken = vault.find(d => d.uuid === device!.uuid)?.token}
        <CredentialsPanel uuid={device.uuid} token={vaultToken} />
      {:else if activeTab === 'Permissions'}
        <WhitelistEditor
          whitelists={device.meshblu.whitelists}
          onSave={saveWhitelists}
        />
      {:else if activeTab === 'Webhooks'}
        <WebhookEditor
          forwarders={device.meshblu.forwarders || {}}
          onSave={saveForwarders}
        />
      {/if}
    </Tabs>
  </div>
{/if}

<style>
  .device-detail {
    max-width: 700px;
  }
  .detail-header {
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }
  .detail-info {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 4px;
  }
  .detail-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }
  .detail-uuid {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    font-weight: 500;
    letter-spacing: 0.05em;
    color: var(--pulse);
  }
  .detail-type {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .detail-name {
    font-family: var(--font-display);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--ink-soft);
  }
  .icon-section {
    margin-bottom: 24px;
  }
  .section-label {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--ink-muted);
    display: block;
    margin-bottom: 8px;
  }
  .loading-text {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    color: var(--ink-muted);
  }
  .error-state {
    max-width: 500px;
  }
  .error-text {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    color: var(--fault);
    margin-bottom: 8px;
  }
  .error-hint {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-muted);
    margin-bottom: 12px;
  }
  .error-actions {
    display: flex;
    gap: 8px;
  }
</style>
