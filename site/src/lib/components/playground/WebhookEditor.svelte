<script lang="ts">
  import type { Forwarders, Forwarder } from '$lib/api/types';
  import Button from '../ui/Button.svelte';
  import Input from '../ui/Input.svelte';

  interface Props {
    forwarders: Forwarders;
    onSave: (forwarders: Forwarders) => void;
  }

  let { forwarders, onSave }: Props = $props();

  const eventTypes = ['broadcast', 'configure', 'message', 'unregister'] as const;
  const subTypes = ['received', 'sent'] as const;

  let editData = $state(JSON.parse(JSON.stringify(forwarders || {})) as Forwarders);
  let newUrl = $state('');
  let newMethod = $state('POST');
  let selectedEvent = $state<string>('message');
  let selectedSub = $state<string>('received');

  function getForwarders(event: string, sub: string): Forwarder[] {
    return (editData as Record<string, Record<string, Forwarder[]>>)[event]?.[sub] || [];
  }

  function addForwarder() {
    if (!newUrl.trim()) return;
    const fwd: Forwarder = { url: newUrl, method: newMethod, type: 'webhook' };
    const key = selectedEvent;
    const sub = selectedSub;
    if (!(editData as Record<string, Record<string, Forwarder[]>>)[key]) {
      (editData as Record<string, Record<string, Forwarder[]>>)[key] = {};
    }
    if (!(editData as Record<string, Record<string, Forwarder[]>>)[key][sub]) {
      (editData as Record<string, Record<string, Forwarder[]>>)[key][sub] = [];
    }
    (editData as Record<string, Record<string, Forwarder[]>>)[key][sub].push(fwd);
    editData = { ...editData };
    newUrl = '';
  }

  function removeForwarder(event: string, sub: string, idx: number) {
    const list = getForwarders(event, sub);
    list.splice(idx, 1);
    (editData as Record<string, Record<string, Forwarder[]>>)[event][sub] = list;
    editData = { ...editData };
  }
</script>

<div class="webhook-editor">
  {#each eventTypes as event}
    {#each subTypes as sub}
      {@const list = getForwarders(event, sub)}
      {#if list.length > 0}
        <div class="wh-group">
          <span class="wh-label">{event}.{sub}</span>
          {#each list as fwd, idx}
            <div class="wh-entry">
              <span class="wh-method">{fwd.method}</span>
              <code class="wh-url">{fwd.url}</code>
              <button class="wh-remove" onclick={() => removeForwarder(event, sub, idx)}>
                <i class="fa-solid fa-xmark"></i>
              </button>
            </div>
          {/each}
        </div>
      {/if}
    {/each}
  {/each}

  <div class="wh-add">
    <div class="wh-add-row">
      <select class="wh-select" bind:value={selectedEvent}>
        {#each eventTypes as e}
          <option value={e}>{e}</option>
        {/each}
      </select>
      <select class="wh-select" bind:value={selectedSub}>
        {#each subTypes as s}
          <option value={s}>{s}</option>
        {/each}
      </select>
      <select class="wh-select" bind:value={newMethod}>
        <option>POST</option>
        <option>PUT</option>
        <option>GET</option>
      </select>
    </div>
    <Input placeholder="https://example.com/webhook" bind:value={newUrl} />
    <Button size="sm" variant="ghost" onclick={addForwarder}>Add Webhook</Button>
  </div>

  <div class="wh-actions">
    <Button size="sm" onclick={() => onSave(editData)}>Save Webhooks</Button>
  </div>
</div>

<style>
  .webhook-editor {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .wh-group { margin-bottom: 8px; }
  .wh-label {
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--pulse);
    display: block;
    margin-bottom: 4px;
  }
  .wh-entry {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: var(--void);
    border: 1px solid var(--border);
    margin-bottom: 4px;
  }
  .wh-method {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--signal);
    font-weight: 500;
  }
  .wh-url {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-soft);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .wh-remove {
    background: none;
    border: none;
    color: var(--fault);
    cursor: pointer;
    padding: 2px;
  }
  .wh-add {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .wh-add-row {
    display: flex;
    gap: 8px;
  }
  .wh-select {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    background: var(--void-high);
    border: 1px solid var(--border-strong);
    color: var(--ink);
    padding: 6px 8px;
    outline: none;
  }
  .wh-actions { margin-top: 4px; }
</style>
