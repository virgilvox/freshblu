<script lang="ts">
  import Button from '../ui/Button.svelte';
  import Input from '../ui/Input.svelte';

  interface Props {
    onSend: (params: { devices: string[]; topic?: string; payload?: unknown }) => void;
  }

  let { onSend }: Props = $props();

  let targetUuid = $state('');
  let topic = $state('');
  let payloadText = $state('{}');
  let isBroadcast = $state(false);
  let parseError = $state('');

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
</script>

<div class="composer">
  <div class="composer-mode">
    <label class="composer-toggle">
      <input type="checkbox" bind:checked={isBroadcast} />
      <span>Broadcast</span>
    </label>
  </div>

  {#if !isBroadcast}
    <Input label="Target UUID" placeholder="Device UUID" bind:value={targetUuid} />
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
