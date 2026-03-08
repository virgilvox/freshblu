<script lang="ts">
  import { onMount } from 'svelte';
  import DevicePanel from '$lib/components/playground/DevicePanel.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { FreshBluClient } from '$lib/api/client';

  let serverUrl = $state('http://localhost:3000');
  let pingStatus = $state('');
  let connectionCount = $state<number | null>(null);
  let pinging = $state(false);

  onMount(() => {
    const stored = localStorage.getItem('freshblu_server_url');
    if (stored) serverUrl = stored;
  });

  async function handlePing() {
    pinging = true;
    pingStatus = '';
    localStorage.setItem('freshblu_server_url', serverUrl);
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
  <title>Device Tester - Playground - FreshBlu</title>
</svelte:head>

<div class="tester-page">
  <div class="tester-toolbar">
    <div class="server-input">
      <span class="toolbar-label">Server</span>
      <input class="server-field" bind:value={serverUrl} placeholder="http://localhost:3000" />
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
  </div>

  <div class="tester-panels">
    <DevicePanel id="a" label="Device A" {serverUrl} accent="var(--pulse)" />
    <DevicePanel id="b" label="Device B" {serverUrl} accent="var(--signal)" />
  </div>
</div>

<style>
  .tester-page {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  .tester-toolbar {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .server-input {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
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
  .server-field:focus {
    border-color: var(--pulse);
  }
  .server-field::placeholder {
    color: var(--ink-ghost);
  }
  .tester-panels {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
  }

  @media (max-width: 900px) {
    .tester-panels { grid-template-columns: 1fr; }
    .server-input { flex-wrap: wrap; }
  }
</style>
