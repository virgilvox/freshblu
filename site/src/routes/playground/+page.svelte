<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Toast from '$lib/components/ui/Toast.svelte';
  import { FreshBluHttp, api, syncApiBaseUrl, getServerUrl, saveServerUrl } from '$lib/api/client';
  import { addToVault, setActiveDevice, vaultDevices, setPrimaryDevice, clearPrimaryDevice, hasPrimaryDevice, getPrimaryCredentials, getPrimaryDevice, primaryUuid } from '$lib/stores/vault';
  import { uuid as authUuid, token as authToken } from '$lib/stores/auth';
  import type { VaultDevice } from '$lib/stores/vault';

  let serverUrl = $state('');
  let pingStatus = $state('');
  let pinging = $state(false);
  let registering = $state(false);
  let regType = $state('');
  let regName = $state('');
  let lastRegistered: { uuid: string; token: string } | null = $state(null);
  let copied = $state('');
  let toast: Toast;
  let showPrimarySave = $state(false);
  let primarySaved = $state(false);
  let primarySaveCreds: { uuid: string; token: string } | null = $state(null);

  // Recovery state
  let recoverUuid = $state('');
  let recoverToken = $state('');
  let recovering = $state(false);

  let vault: VaultDevice[] = $state([]);
  const unsubVault = vaultDevices.subscribe(v => vault = v);

  let currentPrimary = $state('');
  const unsubPrimary = primaryUuid.subscribe(v => currentPrimary = v);

  onMount(() => {
    serverUrl = getServerUrl();
    primarySaved = localStorage.getItem('freshblu_primary_saved') === '1';
    return () => { unsubVault(); unsubPrimary(); };
  });

  async function handlePing() {
    pinging = true;
    pingStatus = '';
    saveServerUrl(serverUrl);
    try {
      const client = new FreshBluHttp(serverUrl);
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
      saveServerUrl(serverUrl);
      const client = new FreshBluHttp(serverUrl);
      const params: Record<string, unknown> = {};
      if (regType) params.type = regType;
      if (regName) params.name = regName;
      const res = await client.register(Object.keys(params).length > 0 ? params : undefined);
      lastRegistered = { uuid: res.uuid, token: res.token };

      await addToVault({
        uuid: res.uuid,
        token: res.token,
        name: regName || undefined,
        type: regType || undefined,
        addedAt: Date.now(),
      });

      setActiveDevice(res.uuid);
      authUuid.set(res.uuid);
      authToken.set(res.token);
      syncApiBaseUrl();
      api.setCredentials(res.uuid, res.token);

      if (!hasPrimaryDevice() || !getPrimaryCredentials()) {
        // First device or orphaned primary — clean up and set as primary
        clearPrimaryDevice();
        setPrimaryDevice(res.uuid);
        primarySaveCreds = { uuid: res.uuid, token: res.token };
        showPrimarySave = true;
        primarySaved = false;
        toast.show('Primary device created. Save your master key!', 'success');
      } else {
        // Auto-claim with primary credentials
        const primaryCreds = getPrimaryCredentials();
        if (primaryCreds) {
          try {
            const primaryClient = new FreshBluHttp(serverUrl);
            primaryClient.setCredentials(primaryCreds.uuid, primaryCreds.token);
            await primaryClient.claimDevice(res.uuid);
            toast.show('Device registered and claimed by primary', 'success');
          } catch {
            toast.show('Device registered but claim failed — not recoverable', 'warn');
          }
        } else {
          toast.show('Device registered (primary credentials not in vault)', 'success');
        }
      }
    } catch (e) {
      toast.show((e as Error).message, 'error');
    }
    registering = false;
  }

  async function handleRecover() {
    recovering = true;
    try {
      saveServerUrl(serverUrl);
      const client = new FreshBluHttp(serverUrl);
      client.setCredentials(recoverUuid, recoverToken);

      // Verify credentials
      await client.whoami();

      // Add primary to vault
      await addToVault({ uuid: recoverUuid, token: recoverToken, addedAt: Date.now() });
      setPrimaryDevice(recoverUuid);
      setActiveDevice(recoverUuid);
      authUuid.set(recoverUuid);
      authToken.set(recoverToken);
      syncApiBaseUrl();
      api.setCredentials(recoverUuid, recoverToken);

      // Fetch owned devices and generate fresh tokens
      const owned = await client.myDevices();
      let recovered = 0;
      for (const device of owned) {
        if (device.uuid === recoverUuid) continue;
        try {
          const tokenRes = await client.generateToken(device.uuid);
          await addToVault({ uuid: device.uuid, token: tokenRes.token, addedAt: Date.now() });
          recovered++;
        } catch {
          // Skip devices we can't generate tokens for
        }
      }

      toast.show(`Vault restored: ${recovered + 1} device${recovered > 0 ? 's' : ''} recovered`, 'success');
      recoverUuid = '';
      recoverToken = '';
    } catch (e) {
      toast.show(`Recovery failed: ${(e as Error).message}`, 'error');
    }
    recovering = false;
  }

  function handlePrimarySaved() {
    primarySaved = true;
    showPrimarySave = false;
    localStorage.setItem('freshblu_primary_saved', '1');
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
    <input class="server-field" bind:value={serverUrl} placeholder="https://api.freshblu.org" aria-label="Server URL" />
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

  <!-- Welcome (first visit) -->
  {#if vault.length === 0 && !currentPrimary}
    <section class="section welcome-section">
      <h2 class="section-title">Welcome to the Mesh</h2>
      <p class="section-desc">The playground lets you interact with a live FreshBlu mesh — register devices, send messages, and watch data flow in real time.</p>
      <div class="welcome-steps">
        <div class="welcome-step">
          <span class="step-number">1</span>
          <span class="step-text">Register your first device to get a UUID and token</span>
        </div>
        <div class="welcome-step">
          <span class="step-number">2</span>
          <span class="step-text">Save your master key so you can recover your vault later</span>
        </div>
        <div class="welcome-step">
          <span class="step-number">3</span>
          <span class="step-text">Open a session, explore the API, or visualize the mesh</span>
        </div>
      </div>
    </section>
  {/if}

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

    {#if showPrimarySave && primarySaveCreds}
      <div class="master-key-banner">
        <div class="master-key-header">
          <i class="fa-solid fa-key"></i>
          <span>Save Your Master Key</span>
        </div>
        <p class="master-key-desc">This is your primary device. Save these credentials somewhere safe — they're needed to recover your vault on a new machine.</p>
        <div class="credentials-result">
          <div class="cred-row">
            <span class="cred-label">UUID</span>
            <code class="cred-value">{primarySaveCreds.uuid}</code>
            <button class="copy-btn" onclick={() => copyToClipboard(primarySaveCreds!.uuid, 'pk-uuid')} title="Copy UUID">
              <i class="fa-solid {copied === 'pk-uuid' ? 'fa-check' : 'fa-copy'}"></i>
            </button>
          </div>
          <div class="cred-row">
            <span class="cred-label">Token</span>
            <code class="cred-value">{primarySaveCreds.token}</code>
            <button class="copy-btn" onclick={() => copyToClipboard(primarySaveCreds!.token, 'pk-token')} title="Copy Token">
              <i class="fa-solid {copied === 'pk-token' ? 'fa-check' : 'fa-copy'}"></i>
            </button>
          </div>
        </div>
        <div class="master-key-actions">
          <Button size="sm" onclick={() => copyToClipboard(`${primarySaveCreds!.uuid}:${primarySaveCreds!.token}`, 'pk-both')}>
            <i class="fa-solid {copied === 'pk-both' ? 'fa-check' : 'fa-copy'}"></i>
            {copied === 'pk-both' ? 'Copied!' : 'Copy Both'}
          </Button>
          <Button size="sm" variant="ghost" onclick={handlePrimarySaved}>
            I have saved my credentials
          </Button>
        </div>
      </div>
    {/if}
  </section>

  <!-- Recover Vault -->
  {#if vault.length === 0}
    <section class="section">
      <h2 class="section-title">Recover Vault</h2>
      <p class="section-desc">Enter your primary device credentials to restore all owned devices.</p>
      <div class="register-form">
        <input class="reg-field" placeholder="Primary UUID" bind:value={recoverUuid} aria-label="Recovery UUID" />
        <input class="reg-field" type="password" placeholder="Primary Token" bind:value={recoverToken} aria-label="Recovery Token" />
        <Button size="sm" variant="signal" onclick={handleRecover} disabled={recovering || !recoverUuid || !recoverToken}>
          <i class="fa-solid fa-rotate"></i>
          {recovering ? 'Recovering...' : 'Recover'}
        </Button>
      </div>
    </section>
  {/if}

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
    {@const primaryDevice = currentPrimary ? vault.find(d => d.uuid === currentPrimary) : null}
    <section class="section vault-status">
      <i class="fa-solid fa-lock"></i>
      <span>{vault.length} device{vault.length === 1 ? '' : 's'} in vault.</span>
      {#if primaryDevice}
        <span class="vault-primary">
          <i class="fa-solid fa-key"></i>
          {primaryDevice.name || primaryDevice.type || primaryDevice.uuid.slice(0, 8)}
        </span>
        {#if !primarySaved}
          <button class="vault-save-link" onclick={() => { showPrimarySave = true; primarySaveCreds = { uuid: primaryDevice.uuid, token: primaryDevice.token }; }}>
            Save master key
          </button>
        {/if}
      {:else if currentPrimary}
        <span class="vault-orphan">
          <i class="fa-solid fa-triangle-exclamation"></i>
          Primary missing from vault
        </span>
      {/if}
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

  .master-key-banner {
    border: 1px solid var(--warn);
    background: var(--void-lift);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .master-key-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-display);
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--warn);
  }
  .master-key-desc {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-soft);
    line-height: var(--leading-normal);
  }
  .master-key-actions {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }

  .welcome-section {
    border: 1px solid var(--border);
    background: var(--void-lift);
    padding: 20px;
  }
  .welcome-steps {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .welcome-step {
    display: flex;
    align-items: center;
    gap: 12px;
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    color: var(--ink-soft);
  }
  .step-number {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: 1px solid var(--pulse);
    color: var(--pulse);
    font-family: var(--font-display);
    font-size: var(--text-xs);
    font-weight: 700;
    flex-shrink: 0;
  }
  .vault-primary {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--pulse);
  }
  .vault-orphan {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--warn);
  }
  .vault-save-link {
    background: none;
    border: none;
    color: var(--warn);
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .vault-save-link:hover { color: var(--ink); }

  @media (max-width: 600px) {
    .server-bar { flex-wrap: wrap; }
    .register-form { flex-direction: column; }
    .reg-field { min-width: 0; width: 100%; }
  }
</style>
