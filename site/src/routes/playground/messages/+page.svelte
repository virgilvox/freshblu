<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import MessageComposer from '$lib/components/playground/MessageComposer.svelte';
  import EventStream from '$lib/components/playground/EventStream.svelte';
  import { api } from '$lib/api/client';
  import { FreshBluWs } from '$lib/api/ws';
  import { uuid as authUuid, token as authToken } from '$lib/stores/auth';
  import { events } from '$lib/stores/events';
  import { goto } from '$app/navigation';

  let ws: FreshBluWs | null = $state(null);
  let connected = $state(false);
  let connectError = $state('');

  onMount(async () => {
    let u = '', t = '';
    authUuid.subscribe(v => u = v)();
    authToken.subscribe(v => t = v)();
    if (!u || !t) {
      goto('/playground');
      return;
    }
    api.setCredentials(u, t);

    ws = new FreshBluWs(u, t);
    ws.on('*', (event) => {
      events.push(event.event, event as Record<string, unknown>, (event as Record<string, unknown>).fromUuid as string | undefined);
    });
    ws.on('ready', () => { connected = true; });
    ws.on('close', () => { connected = false; });

    try {
      await ws.connect();
    } catch (e) {
      connectError = (e as Error).message;
    }
  });

  onDestroy(() => {
    ws?.close();
  });

  function handleSend(params: { devices: string[]; topic?: string; payload?: unknown }) {
    if (ws?.connected) {
      ws.sendMessage(params.devices, params.payload, params.topic);
    } else {
      api.sendMessage(params);
    }
  }
</script>

<svelte:head>
  <title>Messages - Playground - FreshBlu</title>
</svelte:head>

<div class="messages-page">
  <div class="messages-header">
    <h1 class="page-title">Messages</h1>
    <span class="ws-status" class:connected>
      <span class="ws-dot"></span>
      {connected ? 'WebSocket Connected' : 'Disconnected'}
    </span>
  </div>

  {#if connectError}
    <div class="error-msg">
      <i class="fa-solid fa-circle-exclamation"></i>
      {connectError}
    </div>
  {/if}

  <div class="messages-layout">
    <div class="composer-panel">
      <MessageComposer onSend={handleSend} />
    </div>
    <div class="stream-panel">
      <EventStream />
    </div>
  </div>
</div>

<style>
  .messages-header {
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
  .ws-status {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--fault);
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .ws-status.connected { color: var(--online); }
  .ws-dot {
    width: 6px;
    height: 6px;
    background: currentColor;
    display: inline-block;
  }
  .ws-status.connected .ws-dot {
    animation: blink 2.4s ease-in-out infinite;
  }
  .messages-layout {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
  }
  .error-msg {
    margin-bottom: 24px;
    padding: 10px 16px;
    border: 1px solid var(--fault);
    color: var(--fault);
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  @media (max-width: 900px) {
    .messages-layout { grid-template-columns: 1fr; }
  }

  @keyframes blink {
    0%, 80%, 100% { opacity: 1; }
    40% { opacity: 0.3; }
  }
</style>
