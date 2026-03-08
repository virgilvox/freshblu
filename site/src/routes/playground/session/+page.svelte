<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import DevicePanel from '$lib/components/playground/DevicePanel.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { FreshBluClient, getServerUrl, saveServerUrl } from '$lib/api/client';

  let serverUrl = $state('');
  let pingStatus = $state('');
  let pinging = $state(false);

  onMount(() => {
    serverUrl = getServerUrl();
  });

  async function handlePing() {
    pinging = true;
    pingStatus = '';
    saveServerUrl(serverUrl);
    try {
      const client = new FreshBluClient(serverUrl);
      const res = await client.status();
      pingStatus = res.meshblu ? 'online' : 'offline';
    } catch {
      pingStatus = 'error';
    }
    pinging = false;
  }
</script>

<svelte:head>
  <title>Session - Playground - FreshBlu</title>
</svelte:head>

<div class="session-page">
  <div class="session-toolbar">
    <span class="toolbar-label">Server</span>
    <input class="server-field" bind:value={serverUrl} placeholder="http://localhost:3000" aria-label="Server URL" />
    <Button size="sm" variant="ghost" onclick={handlePing} disabled={pinging}>
      {pinging ? '...' : 'Ping'}
    </Button>
    {#if pingStatus === 'online'}
      <Badge variant="online">Online</Badge>
    {:else if pingStatus === 'error'}
      <Badge variant="fault">Unreachable</Badge>
    {:else if pingStatus === 'offline'}
      <Badge variant="warn">Offline</Badge>
    {/if}
  </div>

  <p class="session-hint">
    Register or enter credentials, then connect to start a live session.
    To test two devices talking, open a second browser tab.
  </p>

  <DevicePanel id="session" label="Session" {serverUrl} accent="var(--pulse)" />
</div>

<style>
  .session-page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-width: 700px;
  }
  .session-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .toolbar-label {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--ink-muted);
    flex-shrink: 0;
  }
  .server-field {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    background: var(--void);
    border: 1px solid var(--border-strong);
    color: var(--ink);
    padding: 6px 12px;
    outline: none;
    flex: 1;
    min-width: 200px;
    transition: border-color var(--dur-fast);
  }
  .server-field:focus { border-color: var(--pulse); }
  .server-field::placeholder { color: var(--ink-ghost); }
  .session-hint {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-muted);
    line-height: var(--leading-normal);
  }

  @media (max-width: 600px) {
    .session-toolbar { flex-wrap: wrap; }
  }
</style>
