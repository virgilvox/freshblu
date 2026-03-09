<script lang="ts">
  import Button from '../ui/Button.svelte';
  import Input from '../ui/Input.svelte';
  import type { VaultDevice } from '$lib/stores/vault';

  interface Props {
    onSend: (params: { devices: string[]; topic?: string; payload?: unknown }) => void;
    vaultDevices?: VaultDevice[];
    primaryUuid?: string;
  }

  let { onSend, vaultDevices = [], primaryUuid = '' }: Props = $props();

  let targetUuid = $state('');
  let topic = $state('');
  let payloadText = $state('{}');
  let isBroadcast = $state(false);
  let parseError = $state('');
  let showPicker = $state(false);

  function handleSend() {
    let payload: unknown;
    try {
      payload = JSON.parse(payloadText);
      parseError = '';
    } catch {
      parseError = 'Invalid JSON payload';
      return;
    }

    const devices = isBroadcast ? ['*'] : [targetUuid];
    onSend({
      devices,
      topic: topic || undefined,
      payload,
    });
  }

  function selectDevice(uuid: string) {
    targetUuid = uuid;
    showPicker = false;
  }

  function deviceLabel(vd: VaultDevice): string {
    const name = vd.name || vd.type || vd.uuid.substring(0, 8);
    return vd.uuid === primaryUuid ? `${name} (primary)` : name;
  }
</script>

<div class="composer">
  <div class="composer-mode">
    <label class="composer-toggle">
      <input type="checkbox" bind:checked={isBroadcast} />
      <span>Broadcast</span>
    </label>
  </div>

  {#if !isBroadcast}
    <div class="target-row">
      <div class="target-input">
        <Input label="Target UUID" placeholder="Device UUID" bind:value={targetUuid} />
      </div>
      {#if vaultDevices.length > 0}
        <button class="picker-btn" onclick={() => showPicker = !showPicker} type="button" title="Pick from vault">
          <i class="fa-solid fa-lock"></i>
        </button>
      {/if}
    </div>
    {#if showPicker && vaultDevices.length > 0}
      <div class="picker-dropdown">
        {#each vaultDevices as vd (vd.uuid)}
          <button class="picker-item" onclick={() => selectDevice(vd.uuid)} type="button">
            {#if vd.uuid === primaryUuid}<i class="fa-solid fa-key picker-key"></i>{/if}
            <span class="picker-name">{deviceLabel(vd)}</span>
            <span class="picker-uuid">{vd.uuid.substring(0, 8)}</span>
          </button>
        {/each}
      </div>
    {/if}
  {/if}

  <Input label="Topic" placeholder="Optional topic" bind:value={topic} />

  <div class="field">
    <label class="field-label">Payload</label>
    <textarea
      class="composer-payload"
      bind:value={payloadText}
      rows={5}
      spellcheck="false"
    ></textarea>
    {#if parseError}
      <span class="composer-error">{parseError}</span>
    {/if}
  </div>

  <Button onclick={handleSend} disabled={!isBroadcast && !targetUuid}>
    <i class="fa-solid fa-paper-plane"></i>
    Send Message
  </Button>
</div>

<style>
  .composer {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .composer-mode {
    display: flex;
  }
  .composer-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-muted);
    cursor: pointer;
  }
  .composer-toggle input {
    accent-color: var(--signal);
  }
  .target-row {
    display: flex;
    gap: 4px;
    align-items: flex-end;
  }
  .target-input { flex: 1; }
  .picker-btn {
    background: var(--void);
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 8px 10px;
    cursor: pointer;
    font-size: var(--text-xs);
    transition: color var(--dur-fast), border-color var(--dur-fast);
    flex-shrink: 0;
  }
  .picker-btn:hover {
    color: var(--pulse);
    border-color: var(--pulse);
  }
  .picker-dropdown {
    background: var(--void-lift);
    border: 1px solid var(--border);
    max-height: 160px;
    overflow-y: auto;
  }
  .picker-item {
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
  .picker-item:hover { background: var(--void); }
  .picker-key { color: var(--warn); font-size: 9px; }
  .picker-name { color: var(--pulse); }
  .picker-uuid { color: var(--ink-ghost); font-size: 9px; margin-left: auto; }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field-label {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .composer-payload {
    font-family: var(--font-body);
    font-size: var(--text-sm);
    background: var(--void);
    border: 1px solid var(--border-strong);
    color: var(--ink);
    padding: 12px;
    outline: none;
    resize: vertical;
    line-height: var(--leading-relaxed);
  }
  .composer-payload:focus {
    border-color: var(--pulse);
    box-shadow: 0 0 0 2px var(--pulse-ring);
  }
  .composer-error {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--fault);
  }
</style>
