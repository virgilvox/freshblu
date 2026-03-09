<script lang="ts">
  import { onDestroy } from 'svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { FreshBluClient, type SubscriptionType } from '$lib/api/client';
  import { createEventStore, type EventItem } from '$lib/stores/events';
  import { addToVault, vaultDevices, hasPrimaryDevice, getPrimaryCredentials } from '$lib/stores/vault';
  import type { VaultDevice } from '$lib/stores/vault';

  interface Props {
    id: string;
    label: string;
    serverUrl: string;
    accent?: string;
  }

  let { id, label, serverUrl, accent = 'var(--pulse)' }: Props = $props();

  let panelUuid = $state('');
  let panelToken = $state('');
  let deviceType = $state('');
  let deviceName = $state('');
  let connected = $state(false);
  let statusText = $state('Disconnected');
  let loading = $state('');

  // Send fields
  let sendTo = $state('');
  let sendTopic = $state('');
  let sendPayload = $state('{}');

  // Subscribe fields
  let subEmitter = $state('');
  let subType: SubscriptionType = $state('message.received');

  const subTypes: SubscriptionType[] = [
    'broadcast.sent', 'broadcast.received',
    'configure.sent', 'configure.received',
    'message.sent', 'message.received',
    'unregister.sent', 'unregister.received',
  ];

  const eventStore = createEventStore();
  let eventItems: EventItem[] = $state([]);
  const unsubEvents = eventStore.subscribe((v) => (eventItems = v));

  let vault: VaultDevice[] = $state([]);
  const unsubVault = vaultDevices.subscribe(v => vault = v);

  function loadFromVault(deviceUuid: string) {
    const vd = vault.find(d => d.uuid === deviceUuid);
    if (vd) {
      panelUuid = vd.uuid;
      panelToken = vd.token;
    }
  }

  let client: FreshBluClient | null = null;
  let ws: FreshBluClient | null = null;

  function getClient(): FreshBluClient {
    if (!client) client = new FreshBluClient(serverUrl);
    client.setCredentials(panelUuid, panelToken);
    return client;
  }

  async function handleRegister() {
    loading = 'register';
    try {
      const c = new FreshBluClient(serverUrl);
      const res = await c.register(
        deviceType || deviceName ? { type: deviceType || undefined, name: deviceName || undefined } : undefined
      );
      panelUuid = res.uuid;
      panelToken = res.token;
      eventStore.push('registered', { uuid: res.uuid });
      await addToVault({ uuid: res.uuid, token: res.token, label: `${label}`, addedAt: Date.now() });

      // Auto-claim with primary if available
      if (hasPrimaryDevice()) {
        const primaryCreds = getPrimaryCredentials();
        if (primaryCreds) {
          try {
            const primaryClient = new FreshBluClient(serverUrl);
            primaryClient.setCredentials(primaryCreds.uuid, primaryCreds.token);
            await primaryClient.claimDevice(res.uuid);
            eventStore.push('claimed', { uuid: res.uuid, owner: primaryCreds.uuid });
          } catch {
            eventStore.push('warn', { message: 'Claim failed — device not recoverable' });
          }
        }
      }
    } catch (e) {
      eventStore.push('error', { message: (e as Error).message });
    }
    loading = '';
  }

  async function handleConnect() {
    if (!panelUuid || !panelToken) return;
    loading = 'connect';
    statusText = 'Connecting...';
    try {
      ws?.close();
      ws = new FreshBluClient(serverUrl);
      ws.setCredentials(panelUuid, panelToken);
      ws.on('*', (event: Record<string, unknown>) => {
        eventStore.push(event.event as string, event, event.fromUuid as string | undefined);
      });
      ws.on('ready', () => {
        connected = true;
        statusText = 'Connected';
      });
      ws.on('close', () => {
        connected = false;
        statusText = 'Disconnected';
      });
      await ws.connect();
    } catch (e) {
      statusText = 'Failed';
      eventStore.push('error', { message: (e as Error).message });
    }
    loading = '';
  }

  function handleDisconnect() {
    ws?.close();
    ws = null;
    connected = false;
    statusText = 'Disconnected';
  }

  async function handleWhoami() {
    loading = 'whoami';
    try {
      const c = getClient();
      const res = await c.whoami();
      eventStore.push('whoami', res as unknown as Record<string, unknown>);
    } catch (e) {
      eventStore.push('error', { message: (e as Error).message });
    }
    loading = '';
  }

  async function handleSubscribe() {
    if (!subEmitter) return;
    loading = 'subscribe';
    try {
      const c = getClient();
      await c.createSubscription({ subscriberUuid: panelUuid, emitterUuid: subEmitter, type: subType });
      eventStore.push('subscribed', { emitterUuid: subEmitter, type: subType });
    } catch (e) {
      eventStore.push('error', { message: (e as Error).message });
    }
    loading = '';
  }

  async function handleSend() {
    loading = 'send';
    try {
      let payload: unknown;
      try {
        payload = JSON.parse(sendPayload);
      } catch {
        eventStore.push('error', { message: 'Invalid JSON payload' });
        loading = '';
        return;
      }
      if (sendTo === '*') {
        if (ws?.connected) {
          ws.send({ event: 'broadcast', payload, topic: sendTopic || undefined });
        } else {
          const c = getClient();
          await c.broadcast({ payload, topic: sendTopic || undefined });
        }
      } else {
        if (ws?.connected) {
          ws.sendMessage(sendTo ? [sendTo] : [], payload, sendTopic || undefined);
        } else {
          const c = getClient();
          await c.message({ devices: [sendTo], payload, topic: sendTopic || undefined });
        }
      }
      eventStore.push('sent', { to: sendTo || '*', topic: sendTopic, payload });
    } catch (e) {
      eventStore.push('error', { message: (e as Error).message });
    }
    loading = '';
  }

  function formatTime(d: Date): string {
    return d.toLocaleTimeString('en-US', { hour12: false, fractionalSecondDigits: 3 });
  }

  function eventVariant(type: string): 'online' | 'pulse' | 'warn' | 'fault' | 'pending' | 'muted' {
    switch (type) {
      case 'message': case 'sent': return 'pulse';
      case 'broadcast': return 'pending';
      case 'config': case 'subscribed': return 'warn';
      case 'ready': case 'registered': case 'whoami': return 'online';
      case 'notReady': case 'error': case 'unregistered': return 'fault';
      default: return 'muted';
    }
  }

  onDestroy(() => {
    ws?.close();
    unsubEvents();
    unsubVault();
  });
</script>

<div class="panel" style="--accent: {accent}">
  <div class="panel-header">
    <span class="panel-label">{label}</span>
    <span class="panel-status" class:connected>
      <span class="panel-dot"></span>
      {statusText}
    </span>
  </div>

  <div class="panel-section">
    <div class="section-label">Credentials</div>
    {#if vault.length > 0}
      <div class="field-row">
        <select class="field-sm select" aria-label="Load from vault" onchange={(e) => { const v = (e.target as HTMLSelectElement).value; if (v) loadFromVault(v); }}>
          <option value="">Load from vault...</option>
          {#each vault as vd (vd.uuid)}
            <option value={vd.uuid}>{vd.uuid.substring(0, 8)}...{vd.label ? ` (${vd.label})` : ''}</option>
          {/each}
        </select>
      </div>
    {/if}
    <div class="field-row">
      <input class="field-sm" placeholder="UUID" bind:value={panelUuid} aria-label="Device UUID" />
      <input class="field-sm" type="password" placeholder="Token" bind:value={panelToken} aria-label="Device Token" />
    </div>
  </div>

  <div class="panel-section">
    <div class="section-label">Identity</div>
    <div class="field-row">
      <input class="field-sm" placeholder="Type (optional)" bind:value={deviceType} aria-label="Device Type" />
      <input class="field-sm" placeholder="Name (optional)" bind:value={deviceName} aria-label="Device Name" />
    </div>
  </div>

  <div class="panel-section">
    <div class="section-label">Actions</div>
    <div class="action-row">
      <Button size="sm" onclick={handleRegister} disabled={loading === 'register'}>
        {loading === 'register' ? '...' : 'Register'}
      </Button>
      <Button size="sm" variant="signal" onclick={handleConnect} disabled={loading === 'connect' || !panelUuid || !panelToken}>
        {loading === 'connect' ? '...' : 'Connect'}
      </Button>
      <Button size="sm" variant="muted" onclick={handleDisconnect} disabled={!connected}>
        Disconnect
      </Button>
      <Button size="sm" variant="ghost" onclick={handleWhoami} disabled={!panelUuid || !panelToken || loading === 'whoami'}>
        Whoami
      </Button>
    </div>
  </div>

  <div class="panel-section">
    <div class="section-label">Subscribe</div>
    <div class="field-row">
      <input class="field-sm" placeholder="Emitter UUID" bind:value={subEmitter} style="flex:2" aria-label="Emitter UUID" />
      <select class="field-sm select" bind:value={subType} aria-label="Subscription Type">
        {#each subTypes as st}
          <option value={st}>{st}</option>
        {/each}
      </select>
      <Button size="sm" variant="ghost" onclick={handleSubscribe} disabled={!subEmitter || !panelUuid || loading === 'subscribe'}>
        Sub
      </Button>
    </div>
  </div>

  <div class="panel-section">
    <div class="section-label">Send</div>
    <div class="field-row">
      <input class="field-sm" placeholder="To (uuid or *)" bind:value={sendTo} aria-label="Recipient UUID" />
      <input class="field-sm" placeholder="Topic" bind:value={sendTopic} aria-label="Message Topic" />
    </div>
    <textarea class="payload-area" bind:value={sendPayload} rows={3} spellcheck="false" aria-label="Message Payload"></textarea>
    <Button size="sm" onclick={handleSend} disabled={(!sendTo) || !panelUuid || loading === 'send'}>
      <i class="fa-solid fa-paper-plane"></i>
      Send
    </Button>
  </div>

  <div class="panel-section events-section">
    <div class="section-label">
      Event Log
      <span class="event-count">{eventItems.length}</span>
      <button class="clear-btn" onclick={() => eventStore.clear()}>Clear</button>
    </div>
    <div class="event-log">
      {#each eventItems.toReversed() as item (item.id)}
        <div class="event-row">
          <span class="event-time">{formatTime(item.timestamp)}</span>
          <Badge variant={eventVariant(item.type)}>{item.type}</Badge>
          {#if item.fromUuid}
            <span class="event-from">{item.fromUuid.substring(0, 8)}</span>
          {/if}
          <code class="event-data">{JSON.stringify(item.data)}</code>
        </div>
      {/each}
      {#if eventItems.length === 0}
        <div class="event-empty">No events yet.</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .panel {
    border: 1px solid var(--border);
    border-left: 3px solid var(--accent);
    display: flex;
    flex-direction: column;
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    background: var(--void-lift);
    border-bottom: 1px solid var(--border);
  }
  .panel-label {
    font-family: var(--font-display);
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--accent);
  }
  .panel-status {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--fault);
  }
  .panel-status.connected {
    color: var(--online);
  }
  .panel-dot {
    width: 6px;
    height: 6px;
    background: currentColor;
  }
  .panel-status.connected .panel-dot {
    animation: blink 2.4s ease-in-out infinite;
  }
  .panel-section {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .panel-section:last-child {
    border-bottom: none;
  }
  .section-label {
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--ink-muted);
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .field-row {
    display: flex;
    gap: 8px;
  }
  .field-sm {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    background: var(--void);
    border: 1px solid var(--border-strong);
    color: var(--ink);
    padding: 6px 10px;
    outline: none;
    flex: 1;
    min-width: 0;
    transition: border-color var(--dur-fast);
  }
  .field-sm::placeholder {
    color: var(--ink-ghost);
  }
  .field-sm:focus {
    border-color: var(--pulse);
  }
  .select {
    background: var(--void);
    cursor: pointer;
    flex: 1.5;
  }
  .action-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .payload-area {
    font-family: var(--font-body);
    font-size: var(--text-xs);
    background: var(--void);
    border: 1px solid var(--border-strong);
    color: var(--ink);
    padding: 8px 10px;
    outline: none;
    resize: vertical;
    line-height: var(--leading-relaxed);
  }
  .payload-area:focus {
    border-color: var(--pulse);
  }
  .events-section {
    flex: 1;
    min-height: 0;
  }
  .event-count {
    color: var(--signal);
    font-size: 9px;
  }
  .clear-btn {
    margin-left: auto;
    background: none;
    border: 1px solid var(--border);
    color: var(--ink-muted);
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    padding: 2px 6px;
    cursor: pointer;
    transition: color var(--dur-fast), border-color var(--dur-fast);
  }
  .clear-btn:hover {
    color: var(--fault);
    border-color: var(--fault);
  }
  .event-log {
    max-height: 240px;
    overflow-y: auto;
    border: 1px solid var(--border);
  }
  .event-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-bottom: 1px solid var(--border);
    font-size: 10px;
  }
  .event-row:last-child {
    border-bottom: none;
  }
  .event-time {
    font-family: var(--font-ui);
    color: var(--ink-muted);
    flex-shrink: 0;
  }
  .event-from {
    font-family: var(--font-ui);
    color: var(--signal);
  }
  .event-data {
    font-family: var(--font-body);
    color: var(--ink-soft);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }
  .event-empty {
    padding: 16px;
    text-align: center;
    color: var(--ink-muted);
    font-family: var(--font-ui);
    font-size: var(--text-xs);
  }
  @keyframes blink {
    0%, 80%, 100% { opacity: 1; }
    40% { opacity: 0.3; }
  }
</style>
