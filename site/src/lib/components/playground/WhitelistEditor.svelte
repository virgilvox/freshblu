<script lang="ts">
  import type { Whitelists, WhitelistEntry } from '$lib/api/types';
  import Button from '../ui/Button.svelte';
  import Input from '../ui/Input.svelte';

  interface Props {
    whitelists: Whitelists;
    onSave: (whitelists: Whitelists) => void;
  }

  let { whitelists, onSave }: Props = $props();

  const categories = [
    { key: 'discover', fields: ['view', 'as'] },
    { key: 'configure', fields: ['update', 'sent', 'received', 'as'] },
    { key: 'message', fields: ['from', 'sent', 'received', 'as'] },
    { key: 'broadcast', fields: ['sent', 'received', 'as'] },
  ] as const;

  let editData = $state(JSON.parse(JSON.stringify(whitelists)) as Whitelists);
  let newUuid = $state('');

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
  }

  function removeUuid(category: string, field: string, uuid: string) {
    setEntries(category, field, getEntries(category, field).filter(e => e.uuid !== uuid));
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
              <div class="wl-entry">
                <code>{entry.uuid.substring(0, 8)}...</code>
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
    <Input placeholder="UUID to add" bind:value={newUuid} />
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
  }
  .wl-actions {
    margin-top: 4px;
  }
</style>
