<script lang="ts">
  import { onDestroy } from 'svelte';
  import { vaultDevices, activeUuid, primaryUuid, removeFromVault, clearVault, setActiveDevice } from '$lib/stores/vault';
  import { uuid, token } from '$lib/stores/auth';
  import { api, syncApiBaseUrl } from '$lib/api/client';
  import type { VaultDevice } from '$lib/stores/vault';

  let devices: VaultDevice[] = $state([]);
  let active: string = $state('');
  let primary: string = $state('');
  let open = $state(false);

  const unsubDevices = vaultDevices.subscribe((v) => (devices = v));
  const unsubActive = activeUuid.subscribe((v) => (active = v));
  const unsubPrimary = primaryUuid.subscribe((v) => (primary = v));

  onDestroy(() => {
    unsubDevices();
    unsubActive();
    unsubPrimary();
  });

  function switchDevice(device: VaultDevice) {
    setActiveDevice(device.uuid);
    uuid.set(device.uuid);
    token.set(device.token);
    syncApiBaseUrl();
    api.setCredentials(device.uuid, device.token);
    open = false;
  }

  async function remove(e: Event, deviceUuid: string) {
    e.stopPropagation();
    if (deviceUuid === primary) {
      if (!confirm('This is your PRIMARY KEY. Removing it will require recovery to restore your vault. Continue?')) return;
    }
    await removeFromVault(deviceUuid);
  }

  async function handleClearAll() {
    if (!confirm('Remove all devices from vault?')) return;
    await clearVault();
    open = false;
  }

  function truncate(id: string): string {
    return id.substring(0, 8);
  }
</script>

<div class="vault-switcher">
  <button class="vault-trigger" onclick={() => (open = !open)}>
    <i class="fa-solid fa-lock"></i>
    {#if active && devices.length > 0}
      {@const activeDevice = devices.find(d => d.uuid === active)}
      <span class="vault-active">{activeDevice?.name || activeDevice?.label || activeDevice?.type || truncate(active)}</span>
    {:else}
      <span class="vault-empty">No device</span>
    {/if}
    <i class="fa-solid fa-chevron-down vault-chevron" class:open></i>
  </button>

  {#if open}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="vault-backdrop" onclick={() => (open = false)} onkeydown={() => {}}></div>
    <div class="vault-dropdown">
      <div class="vault-header">
        <i class="fa-solid fa-lock"></i>
        <span>Stored locally</span>
        <span class="vault-count">{devices.length}</span>
      </div>
      {#if devices.length === 0}
        <div class="vault-empty-msg">No devices in vault. Register one to get started.</div>
      {:else}
        <div class="vault-list">
          {#each devices as device (device.uuid)}
            <div
              class="vault-item"
              class:active={device.uuid === active}
              role="button"
              tabindex="0"
              onclick={() => switchDevice(device)}
              onkeydown={(e) => { if (e.key === 'Enter') switchDevice(device); }}
            >
              <span class="vault-dot" class:active={device.uuid === active}></span>
              <span class="vault-id">
                <span class="vault-name">{device.name || device.label || device.type || truncate(device.uuid)}</span>
                {#if device.label || device.type}
                  <span class="vault-uuid-sub">{truncate(device.uuid)}</span>
                {/if}
              </span>
              {#if device.uuid === primary}
                <span class="vault-primary"><i class="fa-solid fa-key"></i> PK</span>
              {/if}
              <button class="vault-remove" onclick={(e) => remove(e, device.uuid)} title="Remove">
                <i class="fa-solid fa-xmark"></i>
              </button>
            </div>
          {/each}
        </div>
        <button class="vault-clear" onclick={handleClearAll}>
          <i class="fa-solid fa-trash"></i>
          Clear All
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .vault-switcher {
    position: relative;
  }
  .vault-trigger {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--void);
    border: 1px solid var(--border);
    color: var(--ink-soft);
    padding: 4px 10px;
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    transition: border-color var(--dur-fast);
  }
  .vault-trigger:hover {
    border-color: var(--signal);
  }
  .vault-active {
    color: var(--signal);
    font-family: var(--font-body);
  }
  .vault-empty {
    color: var(--ink-muted);
  }
  .vault-chevron {
    font-size: 8px;
    transition: transform var(--dur-fast);
  }
  .vault-chevron.open {
    transform: rotate(180deg);
  }
  .vault-backdrop {
    position: fixed;
    inset: 0;
    z-index: 90;
  }
  .vault-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    min-width: 240px;
    background: var(--void-lift);
    border: 1px solid var(--border);
    z-index: 100;
    box-shadow: var(--shadow-sharp);
  }
  .vault-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .vault-count {
    margin-left: auto;
    color: var(--signal);
  }
  .vault-list {
    max-height: 200px;
    overflow-y: auto;
  }
  .vault-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--ink-soft);
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    cursor: pointer;
    text-align: left;
    transition: background var(--dur-fast);
  }
  .vault-item:hover {
    background: var(--void);
  }
  .vault-item.active {
    background: var(--void);
  }
  .vault-dot {
    width: 6px;
    height: 6px;
    background: var(--ink-muted);
    flex-shrink: 0;
  }
  .vault-dot.active {
    background: var(--signal);
    animation: blink 2.4s ease-in-out infinite;
  }
  .vault-id {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .vault-name {
    font-family: var(--font-body);
    color: var(--pulse);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .vault-uuid-sub {
    font-family: var(--font-body);
    font-size: 9px;
    color: var(--ink-ghost);
    letter-spacing: 0.05em;
  }
  .vault-primary {
    color: var(--warn);
    font-size: 9px;
    flex-shrink: 0;
  }
  .vault-remove {
    background: none;
    border: none;
    color: var(--ink-ghost);
    cursor: pointer;
    padding: 2px 4px;
    margin-left: auto;
    font-size: var(--text-xs);
    transition: color var(--dur-fast);
  }
  .vault-remove:hover {
    color: var(--fault);
  }
  .vault-empty-msg {
    padding: 16px 12px;
    color: var(--ink-muted);
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    text-align: center;
  }
  .vault-clear {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 8px 12px;
    background: none;
    border: none;
    color: var(--ink-muted);
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    cursor: pointer;
    transition: color var(--dur-fast);
  }
  .vault-clear:hover {
    color: var(--fault);
  }
  @keyframes blink {
    0%, 80%, 100% { opacity: 1; }
    40% { opacity: 0.3; }
  }
</style>
