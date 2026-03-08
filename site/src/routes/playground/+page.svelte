<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Toast from '$lib/components/ui/Toast.svelte';
  import { FreshBluClient, api, syncApiBaseUrl } from '$lib/api/client';
  import { PUBLIC_API_URL } from '$env/static/public';
  import { addToVault, setActiveDevice, vaultDevices } from '$lib/stores/vault';
  import { uuid as authUuid, token as authToken } from '$lib/stores/auth';
  import type { VaultDevice } from '$lib/stores/vault';

  const defaultUrl = PUBLIC_API_URL || 'http://localhost:3000';
  let serverUrl = $state(defaultUrl);
  let pingStatus = $state('');
  let pinging = $state(false);
  let registering = $state(false);
  let regType = $state('');
  let regName = $state('');
  let lastRegistered: { uuid: string; token: string } | null = $state(null);
  let copied = $state('');
  let toast: Toast;

  let vault: VaultDevice[] = $state([]);
  const unsubVault = vaultDevices.subscribe(v => vault = v);

  onMount(() => {
    const stored = localStorage.getItem('freshblu_server_url');
    if (stored) serverUrl = stored;
    return unsubVault;
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

  async function handleRegister() {
    registering = true;
    lastRegistered = null;
    try {
      localStorage.setItem('freshblu_server_url', serverUrl);
      const client = new FreshBluClient(serverUrl);
      const params: Record<string, unknown> = {};
      if (regType) params.type = regType;
      if (regName) params.name = regName;
      const res = await client.register(Object.keys(params).length > 0 ? params : undefined);
      lastRegistered = { uuid: res.uuid, token: res.token };
      await addToVault({ uuid: res.uuid, token: res.token, addedAt: Date.now() });
      setActiveDevice(res.uuid);
      authUuid.set(res.uuid);
      authToken.set(res.token);
      syncApiBaseUrl();
      api.setCredentials(res.uuid, res.token);
      toast.show('Device registered and added to vault', 'success');
    } catch (e) {
      toast.show((e as Error).message, 'error');
    }
    registering = false;
  }

  async function copyToClipboard(text: string, label: string) {
    await navigator.clipboard.writeText(text);
    copied = label;
    setTimeout(() => copied = '', 1500);
  }
</script>

<svelte:head>
  <title>Playground - FreshBlu</title>
</svelte:head>

<div class="start-page">
  <!-- Server Connection -->
  <div class="server-bar">
    <span class="bar-label">Server</span>
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

  <!-- Quick Register -->
  <section class="section">
    <h2 class="section-title">Register a Device</h2>
    <p class="section-desc">Create a new device identity on the mesh. You'll get a UUID and token for authentication.</p>
    <div class="register-form">
      <input class="reg-field" placeholder="Type (optional)" bind:value={regType} aria-label="Device type" />
      <input class="reg-field" placeholder="Name (optional)" bind:value={regName} aria-label="Device name" />
      <Button size="sm" onclick={handleRegister} disabled={registering}>
        <i class="fa-solid fa-plus"></i>
        {registering ? 'Registering...' : 'Register'}
      </Button>
    </div>

    {#if lastRegistered}
      <div class="credentials-result">
        <div class="cred-row">
          <span class="cred-label">UUID</span>
          <code class="cred-value">{lastRegistered.uuid}</code>
          <button class="copy-btn" onclick={() => copyToClipboard(lastRegistered!.uuid, 'uuid')} title="Copy UUID">
            <i class="fa-solid {copied === 'uuid' ? 'fa-check' : 'fa-copy'}"></i>
          </button>
        </div>
        <div class="cred-row">
          <span class="cred-label">Token</span>
          <code class="cred-value">{lastRegistered.token}</code>
          <button class="copy-btn" onclick={() => copyToClipboard(lastRegistered!.token, 'token')} title="Copy Token">
            <i class="fa-solid {copied === 'token' ? 'fa-check' : 'fa-copy'}"></i>
          </button>
        </div>
      </div>
    {/if}
  </section>

  <!-- What's Next -->
  <section class="section">
    <h2 class="section-title">What's Next</h2>
    <div class="cards-grid">
      <a href="/playground/session" class="next-card">
        <i class="fa-solid fa-terminal card-icon"></i>
        <span class="card-title">Open a Session</span>
        <span class="card-desc">Connect via WebSocket, send messages, subscribe to events — all in one live panel.</span>
      </a>
      <a href="/playground/api" class="next-card">
        <i class="fa-solid fa-code card-icon"></i>
        <span class="card-title">Explore the API</span>
        <span class="card-desc">Try every HTTP endpoint with your vault credentials. See requests and responses live.</span>
      </a>
      <a href="/playground/visualizer" class="next-card">
        <i class="fa-solid fa-diagram-project card-icon"></i>
        <span class="card-title">Visualize the Mesh</span>
        <span class="card-desc">See your devices as nodes and watch messages flow between them in real time.</span>
      </a>
    </div>
  </section>

  <!-- Vault Status -->
  {#if vault.length > 0}
    <section class="section vault-status">
      <i class="fa-solid fa-lock"></i>
      <span>{vault.length} device{vault.length === 1 ? '' : 's'} in vault.</span>
      <a href="/playground/devices" class="vault-link">Manage devices</a>
    </section>
  {/if}
</div>

<Toast bind:this={toast} />

<style>
  .start-page {
    display: flex;
    flex-direction: column;
    gap: 32px;
    max-width: 800px;
  }
  .server-bar {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .bar-label {
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

  .section { display: flex; flex-direction: column; gap: 12px; }
  .section-title {
    font-family: var(--font-display);
    font-size: var(--text-lg);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .section-desc {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    color: var(--ink-soft);
    line-height: var(--leading-normal);
  }

  .register-form {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  .reg-field {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    background: var(--void);
    border: 1px solid var(--border-strong);
    color: var(--ink);
    padding: 6px 12px;
    outline: none;
    min-width: 140px;
    flex: 1;
    transition: border-color var(--dur-fast);
  }
  .reg-field:focus { border-color: var(--pulse); }
  .reg-field::placeholder { color: var(--ink-ghost); }

  .credentials-result {
    background: var(--void-lift);
    border: 1px solid var(--border);
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .cred-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .cred-label {
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--ink-muted);
    min-width: 44px;
  }
  .cred-value {
    font-family: var(--font-body);
    font-size: var(--text-xs);
    color: var(--pulse);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .copy-btn {
    background: none;
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 2px 6px;
    cursor: pointer;
    font-size: var(--text-xs);
    transition: color var(--dur-fast), border-color var(--dur-fast);
    flex-shrink: 0;
  }
  .copy-btn:hover { color: var(--pulse); border-color: var(--pulse); }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 12px;
  }
  .next-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 20px;
    border: 1px solid var(--border);
    background: var(--void-lift);
    text-decoration: none;
    transition: border-color var(--dur-fast), transform var(--dur-fast);
  }
  .next-card:hover {
    border-color: var(--pulse);
    transform: translateY(-2px);
  }
  .card-icon {
    font-size: var(--text-lg);
    color: var(--pulse);
  }
  .card-title {
    font-family: var(--font-display);
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink);
  }
  .card-desc {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-muted);
    line-height: var(--leading-normal);
  }

  .vault-status {
    flex-direction: row;
    align-items: center;
    gap: 8px;
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    color: var(--ink-muted);
    padding: 10px 16px;
    border: 1px solid var(--border);
    background: var(--void-lift);
  }
  .vault-link {
    margin-left: auto;
    color: var(--signal);
    text-decoration: none;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .vault-link:hover { color: var(--ink); }

  @media (max-width: 600px) {
    .server-bar { flex-wrap: wrap; }
    .register-form { flex-direction: column; }
    .reg-field { min-width: 0; width: 100%; }
  }
</style>
