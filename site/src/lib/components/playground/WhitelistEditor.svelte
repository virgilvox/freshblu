<script lang="ts">
  import type { Whitelists, WhitelistEntry } from '$lib/api/client';
  import type { VaultDevice } from '$lib/stores/vault';
  import Button from '../ui/Button.svelte';
  import Input from '../ui/Input.svelte';

  interface Props {
    whitelists: Whitelists;
    onSave: (whitelists: Whitelists) => void;
    vaultDevices?: VaultDevice[];
    primaryUuid?: string;
  }

  let { whitelists, onSave, vaultDevices = [], primaryUuid = '' }: Props = $props();

  const categories = [
    { key: 'discover', fields: ['view', 'as'] },
    { key: 'configure', fields: ['update', 'sent', 'received', 'as'] },
    { key: 'message', fields: ['from', 'sent', 'received', 'as'] },
    { key: 'broadcast', fields: ['sent', 'received', 'as'] },
  ] as const;

  let editData = $state(JSON.parse(JSON.stringify(whitelists)) as Whitelists);
  let newUuid = $state('');
  let showDropdown = $state(false);

  function getEntries(category: string, field: string): WhitelistEntry[] {
    return (editData as Record<string, Record<string, WhitelistEntry[]>>)[category]?.[field] || [];
  }

  function setEntries(category: string, field: string, entries: WhitelistEntry[]) {
    (editData as Record<string, Record<string, WhitelistEntry[]>>)[category][field] = entries;
    editData = { ...editData };
  }

  function hasWildcard(category: string, field: string): boolean {
    return getEntries(category, field).some(e => e.uuid === '*');
  }

  function toggleWildcard(category: string, field: string) {
    const entries = getEntries(category, field);
    if (hasWildcard(category, field)) {
      setEntries(category, field, entries.filter(e => e.uuid !== '*'));
    } else {
      setEntries(category, field, [...entries, { uuid: '*' }]);
    }
  }

  function addUuid(category: string, field: string) {
    if (!newUuid.trim()) return;
    const entries = getEntries(category, field);
    if (!entries.some(e => e.uuid === newUuid)) {
      setEntries(category, field, [...entries, { uuid: newUuid }]);
    }
    newUuid = '';
    showDropdown = false;
  }

  function removeUuid(category: string, field: string, uuid: string) {
    if (uuid === primaryUuid) {
      if (!confirm('This is your PRIMARY KEY device. Removing it from this whitelist may break vault recovery. Continue?')) return;
    }
    setEntries(category, field, getEntries(category, field).filter(e => e.uuid !== uuid));
  }

  function deviceLabel(uuid: string): string {
    if (uuid === '*') return '*';
    const vd = vaultDevices.find(d => d.uuid === uuid);
    const name = vd?.name || vd?.label || vd?.type;
    const short = uuid.substring(0, 8);
    if (uuid === primaryUuid) {
      return name ? `${name} (primary)` : `${short}... (primary)`;
    }
    return name ? `${name} (${short})` : `${short}...`;
  }

  function selectVaultDevice(uuid: string) {
    newUuid = uuid;
    showDropdown = false;
  }
</script>

<div class="whitelist-editor">
  {#each categories as cat}
    <div class="wl-category">
      <span class="wl-category-name">{cat.key}</span>
      {#each cat.fields as field}
        <div class="wl-field">
          <div class="wl-field-header">
            <span class="wl-field-name">{field}</span>
            <button
              class="wl-wildcard"
              class:active={hasWildcard(cat.key, field)}
              onclick={() => toggleWildcard(cat.key, field)}
            >*</button>
          </div>
          <div class="wl-entries">
            {#each getEntries(cat.key, field).filter(e => e.uuid !== '*') as entry}
              <div class="wl-entry" class:is-primary={entry.uuid === primaryUuid}>
                {#if entry.uuid === primaryUuid}<i class="fa-solid fa-key wl-key-icon"></i>{/if}
                <code>{deviceLabel(entry.uuid)}</code>
                <button class="wl-remove" onclick={() => removeUuid(cat.key, field, entry.uuid)}>
                  <i class="fa-solid fa-xmark"></i>
                </button>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/each}

  <div class="wl-add-row">
    <div class="wl-add-input">
      <Input placeholder="UUID to add" bind:value={newUuid} />
      {#if vaultDevices.length > 0}
        <button class="wl-vault-btn" onclick={() => showDropdown = !showDropdown} type="button" title="Pick from vault">
          <i class="fa-solid fa-lock"></i>
        </button>
      {/if}
    </div>
    {#if showDropdown && vaultDevices.length > 0}
      <div class="wl-vault-dropdown">
        {#each vaultDevices as vd (vd.uuid)}
          <button class="wl-vault-item" onclick={() => selectVaultDevice(vd.uuid)} type="button">
            {#if vd.uuid === primaryUuid}<i class="fa-solid fa-key wl-key-icon"></i>{/if}
            <span class="wl-vault-name">{vd.name || vd.type || vd.uuid.substring(0, 8)}</span>
            <span class="wl-vault-uuid">{vd.uuid.substring(0, 8)}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <div class="wl-actions">
    <Button size="sm" onclick={() => onSave(editData)}>Save Permissions</Button>
  </div>
</div>

<style>
  .whitelist-editor {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .wl-category {
    border: 1px solid var(--border);
    padding: 12px;
  }
  .wl-category-name {
    font-family: var(--font-display);
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--pulse);
    display: block;
    margin-bottom: 8px;
  }
  .wl-field {
    margin-bottom: 8px;
  }
  .wl-field-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }
  .wl-field-name {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .wl-wildcard {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    background: none;
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 1px 6px;
    cursor: pointer;
    transition: all var(--dur-fast);
  }
  .wl-wildcard.active {
    background: var(--online);
    border-color: var(--online);
    color: var(--void);
  }
  .wl-entries {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .wl-entry {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    background: var(--void);
    border: 1px solid var(--border);
    padding: 2px 6px;
    color: var(--ink-soft);
  }
  .wl-entry.is-primary {
    border-color: var(--warn);
  }
  .wl-key-icon {
    color: var(--warn);
    font-size: 9px;
  }
  .wl-remove {
    background: none;
    border: none;
    color: var(--fault);
    cursor: pointer;
    font-size: 9px;
    padding: 0 2px;
  }
  .wl-add-row {
    max-width: 300px;
    position: relative;
  }
  .wl-add-input {
    display: flex;
    gap: 4px;
    align-items: flex-end;
  }
  .wl-add-input :global(.field) { flex: 1; }
  .wl-vault-btn {
    background: var(--void);
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 8px 10px;
    cursor: pointer;
    font-size: var(--text-xs);
    transition: color var(--dur-fast), border-color var(--dur-fast);
    flex-shrink: 0;
  }
  .wl-vault-btn:hover {
    color: var(--pulse);
    border-color: var(--pulse);
  }
  .wl-vault-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--void-lift);
    border: 1px solid var(--border);
    z-index: 10;
    max-height: 160px;
    overflow-y: auto;
  }
  .wl-vault-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 10px;
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
  .wl-vault-item:hover { background: var(--void); }
  .wl-vault-name { color: var(--pulse); }
  .wl-vault-uuid { color: var(--ink-ghost); font-size: 9px; margin-left: auto; }
  .wl-actions {
    margin-top: 4px;
  }
</style>
