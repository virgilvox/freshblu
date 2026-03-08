<script lang="ts">
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import { api } from '$lib/api/client';
  import { setCredentials } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let mode: 'register' | 'connect' = $state('register');
  let connectUuid = $state('');
  let connectToken = $state('');
  let error = $state('');
  let loading = $state(false);

  let registered: { uuid: string; token: string } | null = $state(null);
  let copiedField = $state('');

  async function copyText(text: string, field: string) {
    await navigator.clipboard.writeText(text);
    copiedField = field;
    setTimeout(() => copiedField = '', 2000);
  }

  async function handleRegister() {
    error = '';
    loading = true;
    try {
      const res = await api.register();
      api.setCredentials(res.uuid, res.token);
      setCredentials(res.uuid, res.token);
      registered = { uuid: res.uuid, token: res.token };
    } catch (e) {
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  }

  async function handleConnect() {
    error = '';
    loading = true;
    try {
      api.setCredentials(connectUuid, connectToken);
      await api.authenticate();
      setCredentials(connectUuid, connectToken);
      goto('/playground/devices');
    } catch (e) {
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Playground - FreshBlu</title>
</svelte:head>

<div class="connect-page">
  <h1 class="page-title">Connect to FreshBlu</h1>
  <p class="page-desc">Register a new device or connect with existing credentials.</p>

  {#if registered}
    <Card title="Device Registered">
      <p class="card-text">Save these credentials. The token is shown only once.</p>
      <div class="cred-grid">
        <div class="cred-row">
          <span class="cred-label">UUID</span>
          <code class="cred-value">{registered.uuid}</code>
          <button class="cred-copy" onclick={() => copyText(registered!.uuid, 'uuid')}>
            <i class="fa-solid {copiedField === 'uuid' ? 'fa-check' : 'fa-copy'}"></i>
          </button>
        </div>
        <div class="cred-row">
          <span class="cred-label">Token</span>
          <code class="cred-value">{registered.token}</code>
          <button class="cred-copy" onclick={() => copyText(registered!.token, 'token')}>
            <i class="fa-solid {copiedField === 'token' ? 'fa-check' : 'fa-copy'}"></i>
          </button>
        </div>
        <div class="cred-row">
          <span class="cred-label">JSON</span>
          <code class="cred-value">{JSON.stringify({ uuid: registered.uuid, token: registered.token })}</code>
          <button class="cred-copy" onclick={() => copyText(JSON.stringify({ uuid: registered!.uuid, token: registered!.token }, null, 2), 'json')}>
            <i class="fa-solid {copiedField === 'json' ? 'fa-check' : 'fa-copy'}"></i>
          </button>
        </div>
      </div>
      <div class="action-row">
        <Button onclick={() => goto('/playground/devices')}>Continue to Dashboard</Button>
      </div>
    </Card>
  {:else}
    <div class="mode-switch">
      <button class="mode-btn" class:active={mode === 'register'} onclick={() => mode = 'register'}>
        Register
      </button>
      <button class="mode-btn" class:active={mode === 'connect'} onclick={() => mode = 'connect'}>
        Connect
      </button>
    </div>

    {#if mode === 'register'}
      <Card title="Register New Device">
        <p class="card-text">Create a new device on the mesh. You will receive a UUID and token.</p>
        <div class="action-row">
          <Button onclick={handleRegister} disabled={loading}>
            {loading ? 'Registering...' : 'Register Device'}
          </Button>
        </div>
      </Card>
    {:else}
      <Card title="Connect Existing Device">
        <div class="connect-form">
          <Input label="UUID" placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" bind:value={connectUuid} />
          <Input label="Token" type="password" placeholder="Device token" bind:value={connectToken} />
          <div class="action-row">
            <Button onclick={handleConnect} disabled={loading || !connectUuid || !connectToken}>
              {loading ? 'Connecting...' : 'Connect'}
            </Button>
          </div>
        </div>
      </Card>
    {/if}
  {/if}

  {#if error}
    <div class="error-msg">
      <i class="fa-solid fa-circle-exclamation"></i>
      {error}
    </div>
  {/if}
</div>

<style>
  .connect-page {
    max-width: 480px;
  }
  .page-title {
    font-family: var(--font-display);
    font-size: var(--text-2xl);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    margin-bottom: 8px;
  }
  .page-desc {
    font-size: var(--text-sm);
    color: var(--ink-soft);
    margin-bottom: 24px;
  }
  .mode-switch {
    display: flex;
    gap: 0;
    margin-bottom: 24px;
  }
  .mode-btn {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-muted);
    background: var(--void-lift);
    border: 1px solid var(--border);
    padding: 8px 20px;
    cursor: pointer;
    transition: color var(--dur-fast), background var(--dur-fast), border-color var(--dur-fast);
  }
  .mode-btn:first-child { border-right: none; }
  .mode-btn.active {
    color: var(--signal);
    border-color: var(--signal);
    background: var(--signal-dim);
  }
  .card-text {
    font-size: var(--text-sm);
    color: var(--ink-soft);
    margin-bottom: 16px;
  }
  .connect-form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .action-row {
    margin-top: 8px;
  }
  .error-msg {
    margin-top: 16px;
    padding: 10px 16px;
    border: 1px solid var(--fault);
    color: var(--fault);
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .cred-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }
  .cred-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--void);
    border: 1px solid var(--border);
  }
  .cred-label {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-muted);
    min-width: 48px;
  }
  .cred-value {
    font-family: var(--font-body);
    font-size: var(--text-xs);
    color: var(--pulse);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cred-copy {
    background: none;
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 4px 8px;
    cursor: pointer;
    font-size: var(--text-xs);
    transition: color var(--dur-fast), border-color var(--dur-fast);
    flex-shrink: 0;
  }
  .cred-copy:hover {
    color: var(--pulse);
    border-color: var(--pulse);
  }
</style>
