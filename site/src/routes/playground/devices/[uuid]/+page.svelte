<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/state';
  import Tabs from '$lib/components/ui/Tabs.svelte';
  import CredentialsPanel from '$lib/components/playground/CredentialsPanel.svelte';
  import PropertyEditor from '$lib/components/playground/PropertyEditor.svelte';
  import WhitelistEditor from '$lib/components/playground/WhitelistEditor.svelte';
  import WebhookEditor from '$lib/components/playground/WebhookEditor.svelte';
  import IconPicker from '$lib/components/playground/IconPicker.svelte';
  import StatusDot from '$lib/components/ui/StatusDot.svelte';
  import { api } from '$lib/api/client';
  import { uuid as authUuid, token as authToken } from '$lib/stores/auth';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { vaultDevices } from '$lib/stores/vault';
  import type { Device, Whitelists, Forwarders } from '$lib/api/types';
  import type { VaultDevice } from '$lib/stores/vault';

  let vault: VaultDevice[] = $state([]);
  const unsubVault = vaultDevices.subscribe(v => vault = v);

  onDestroy(unsubVault);

  function isInVault(deviceUuid: string): boolean {
    return vault.some(d => d.uuid === deviceUuid);
  }

  const tabs = ['Properties', 'Credentials', 'Permissions', 'Webhooks'];
  let activeTab = $state(tabs[0]);
  let device: Device | null = $state(null);
  let loading = $state(true);

  const deviceUuid = page.params.uuid;

  onMount(async () => {
    let u = '', t = '';
    authUuid.subscribe(v => u = v)();
    authToken.subscribe(v => t = v)();
    api.setCredentials(u, t);
    try {
      device = await api.getDevice(deviceUuid);
    } catch {
      device = null;
    }
    loading = false;
  });

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
  <p class="error-text">Device not found.</p>
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
      {#if device.type}
        <span class="detail-type">{device.type}</span>
      {/if}
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
        <CredentialsPanel uuid={device.uuid} />
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
  .loading-text, .error-text {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    color: var(--ink-muted);
  }
</style>
