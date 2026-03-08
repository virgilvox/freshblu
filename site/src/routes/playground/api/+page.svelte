<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
  import { uuid, token } from '$lib/stores/auth';
  import { activeUuid, vaultDevices } from '$lib/stores/vault';
  import type { VaultDevice } from '$lib/stores/vault';

  interface Endpoint {
    method: string;
    path: string;
    label: string;
    description: string;
    pathParams?: string[];
    exampleBody?: string;
  }

  const endpoints: Endpoint[] = [
    { method: 'GET', path: '/status', label: 'Status', description: 'Check server health' },
    { method: 'POST', path: '/authenticate', label: 'Authenticate', description: 'Verify device credentials', exampleBody: '{"uuid": "", "token": ""}' },
    { method: 'POST', path: '/devices', label: 'Register Device', description: 'Create a new device', exampleBody: '{"type": "sensor"}' },
    { method: 'GET', path: '/devices/:uuid', label: 'Get Device', description: 'Retrieve a device by UUID', pathParams: ['uuid'] },
    { method: 'PUT', path: '/devices/:uuid', label: 'Update Device', description: 'Update device properties', pathParams: ['uuid'], exampleBody: '{"name": "my-device"}' },
    { method: 'DELETE', path: '/devices/:uuid', label: 'Delete Device', description: 'Unregister a device', pathParams: ['uuid'] },
    { method: 'GET', path: '/whoami', label: 'Whoami', description: 'Get authenticated device info' },
    { method: 'GET', path: '/mydevices', label: 'My Devices', description: 'List devices owned by authenticated device' },
    { method: 'POST', path: '/devices/search', label: 'Search Devices', description: 'Search devices by properties', exampleBody: '{"type": "sensor"}' },
    { method: 'POST', path: '/claimdevice/:uuid', label: 'Claim Device', description: 'Claim ownership of a device', pathParams: ['uuid'] },
    { method: 'POST', path: '/messages', label: 'Send Message', description: 'Send message to devices', exampleBody: '{"devices": ["*"], "payload": {"hello": true}}' },
    { method: 'POST', path: '/broadcasts', label: 'Broadcast', description: 'Broadcast to subscribers', exampleBody: '{"payload": {"data": 1}}' },
    { method: 'GET', path: '/devices/:uuid/subscriptions', label: 'List Subscriptions', description: 'List device subscriptions', pathParams: ['uuid'] },
    { method: 'POST', path: '/devices/:uuid/subscriptions', label: 'Create Subscription', description: 'Subscribe to events', pathParams: ['uuid'], exampleBody: '{"emitterUuid": "", "type": "message.received"}' },
    { method: 'DELETE', path: '/devices/:uuid/subscriptions/:emitterUuid/:type', label: 'Delete Subscription', description: 'Remove a subscription', pathParams: ['uuid', 'emitterUuid', 'type'] },
    { method: 'POST', path: '/devices/:uuid/tokens', label: 'Generate Token', description: 'Generate a new device token', pathParams: ['uuid'] },
    { method: 'DELETE', path: '/devices/:uuid/tokens/:token', label: 'Revoke Token', description: 'Revoke a specific token', pathParams: ['uuid', 'token'] },
    { method: 'POST', path: '/devices/:uuid/token', label: 'Reset Token', description: 'Reset device token (invalidates old)', pathParams: ['uuid'] },
    { method: 'GET', path: '/v2/devices/:uuid', label: 'Get Device (v2)', description: 'V2 endpoint for device retrieval', pathParams: ['uuid'] },
    { method: 'PATCH', path: '/v2/devices/:uuid', label: 'Patch Device (v2)', description: 'V2 partial device update', pathParams: ['uuid'], exampleBody: '{"$set": {"name": "updated"}}' },
    { method: 'POST', path: '/v2/devices/:uuid/tokens', label: 'Generate Token (v2)', description: 'V2 token generation', pathParams: ['uuid'] },
    { method: 'DELETE', path: '/v2/devices/:uuid/tokens/:token', label: 'Revoke Token (v2)', description: 'V2 token revocation', pathParams: ['uuid', 'token'] },
    { method: 'GET', path: '/metrics', label: 'Metrics', description: 'Prometheus metrics endpoint' },
  ];

  let filter = $state('');
  let selected: Endpoint | null = $state(null);
  let paramValues: Record<string, string> = $state({});
  let bodyText = $state('');
  import { PUBLIC_API_URL } from '$env/static/public';

  const defaultUrl = PUBLIC_API_URL || 'http://localhost:3000';
  let serverUrl = $state(defaultUrl);
  let executing = $state(false);
  let responseStatus = $state<number | null>(null);
  let responseBody = $state('');
  let responseTime = $state<number | null>(null);
  let controller: AbortController | null = null;

  // Reactive auth credentials
  let vaultList: VaultDevice[] = $state([]);
  let active: string = $state('');
  let creds: { uuid: string; token: string } | null = $state(null);

  const unsubVault = vaultDevices.subscribe(v => vaultList = v);
  const unsubActive = activeUuid.subscribe(v => active = v);

  onDestroy(() => {
    unsubVault();
    unsubActive();
  });

  $effect(() => {
    const device = vaultList.find(d => d.uuid === active);
    creds = device ? { uuid: device.uuid, token: device.token } : null;
  });

  onMount(() => {
    const stored = localStorage.getItem('freshblu_server_url');
    if (stored) serverUrl = stored;
    selected = endpoints[0];
  });

  $effect(() => {
    if (selected) {
      bodyText = selected.exampleBody || '';
      paramValues = {};
      responseStatus = null;
      responseBody = '';
      responseTime = null;
    }
  });

  function methodVariant(method: string): 'online' | 'pulse' | 'warn' | 'fault' | 'muted' {
    switch (method) {
      case 'GET': return 'online';
      case 'POST': return 'pulse';
      case 'PUT': case 'PATCH': return 'warn';
      case 'DELETE': return 'fault';
      default: return 'muted';
    }
  }

  function buildPath(ep: Endpoint): string {
    let path = ep.path;
    for (const p of ep.pathParams || []) {
      path = path.replace(`:${p}`, paramValues[p] || `:${p}`);
    }
    return path;
  }

  function filteredEndpoints(): Endpoint[] {
    if (!filter) return endpoints;
    const f = filter.toLowerCase();
    return endpoints.filter(
      (ep) => ep.label.toLowerCase().includes(f) || ep.path.toLowerCase().includes(f) || ep.method.toLowerCase().includes(f)
    );
  }

  async function execute() {
    if (!selected) return;
    if (controller) controller.abort();
    controller = new AbortController();
    executing = true;
    const path = buildPath(selected);
    const url = serverUrl + path;

    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (creds) {
      headers['Authorization'] = 'Basic ' + btoa(creds.uuid + ':' + creds.token);
    }

    const start = performance.now();
    try {
      const res = await fetch(url, {
        method: selected.method,
        headers,
        body: ['POST', 'PUT', 'PATCH'].includes(selected.method) && bodyText ? bodyText : undefined,
        signal: controller.signal,
      });
      responseTime = Math.round(performance.now() - start);
      responseStatus = res.status;
      const text = await res.text();
      try {
        responseBody = JSON.stringify(JSON.parse(text), null, 2);
      } catch {
        responseBody = text;
      }
    } catch (e) {
      if ((e as Error).name === 'AbortError') {
        executing = false;
        return;
      }
      responseTime = Math.round(performance.now() - start);
      responseStatus = 0;
      responseBody = (e as Error).message;
    }
    executing = false;
  }
</script>

<svelte:head>
  <title>API Explorer - Playground - FreshBlu</title>
</svelte:head>

<div class="api-page">
  <div class="api-sidebar">
    <input class="api-filter" placeholder="Filter endpoints..." bind:value={filter} />
    <div class="endpoint-list">
      {#each filteredEndpoints() as ep}
        <button
          class="endpoint-item"
          class:active={selected === ep}
          onclick={() => (selected = ep)}
        >
          <Badge variant={methodVariant(ep.method)}>{ep.method}</Badge>
          <span class="endpoint-label">{ep.label}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="api-main">
    {#if selected}
      <div class="request-section">
        <div class="request-header">
          <Badge variant={methodVariant(selected.method)}>{selected.method}</Badge>
          <code class="request-path">{buildPath(selected)}</code>
        </div>
        <p class="request-desc">{selected.description}</p>

        {#if selected.pathParams && selected.pathParams.length > 0}
          <div class="param-section">
            <span class="param-title">Path Parameters</span>
            <div class="param-fields">
              {#each selected.pathParams as param}
                <div class="param-row">
                  <span class="param-name">:{param}</span>
                  <input class="param-input" placeholder={param} bind:value={paramValues[param]} />
                </div>
              {/each}
            </div>
          </div>
        {/if}

        {#if ['POST', 'PUT', 'PATCH'].includes(selected.method)}
          <div class="body-section">
            <span class="param-title">Request Body</span>
            <textarea class="body-editor" bind:value={bodyText} rows={6} spellcheck="false"></textarea>
          </div>
        {/if}

        <div class="auth-note">
          {#if creds}
            <i class="fa-solid fa-lock"></i> Using vault credentials
          {:else}
            <i class="fa-solid fa-lock-open"></i> No active credentials
          {/if}
        </div>

        <Button onclick={execute} disabled={executing}>
          <i class="fa-solid fa-play"></i>
          {executing ? 'Executing...' : 'Execute'}
        </Button>
      </div>

      {#if responseStatus !== null}
        <div class="response-section">
          <div class="response-header">
            <span class="response-label">Response</span>
            <Badge variant={responseStatus >= 200 && responseStatus < 300 ? 'online' : responseStatus === 0 ? 'fault' : 'warn'}>
              {responseStatus === 0 ? 'Error' : responseStatus}
            </Badge>
            {#if responseTime !== null}
              <span class="response-time">{responseTime}ms</span>
            {/if}
          </div>
          <CodeBlock code={responseBody} lang="json" />
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .api-page {
    display: grid;
    grid-template-columns: 280px 1fr;
    gap: 24px;
  }
  .api-sidebar {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .api-filter {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    background: var(--void);
    border: 1px solid var(--border-strong);
    color: var(--ink);
    padding: 8px 12px;
    outline: none;
  }
  .api-filter::placeholder { color: var(--ink-ghost); }
  .api-filter:focus { border-color: var(--pulse); }
  .endpoint-list {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    max-height: 600px;
    overflow-y: auto;
  }
  .endpoint-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
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
  .endpoint-item:last-child { border-bottom: none; }
  .endpoint-item:hover { background: var(--void-lift); }
  .endpoint-item.active { background: var(--void-lift); color: var(--ink); }
  .endpoint-label { white-space: nowrap; }
  .api-main {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  .request-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .request-header {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .request-path {
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--pulse);
  }
  .request-desc {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    color: var(--ink-soft);
  }
  .param-section, .body-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .param-title {
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.15em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .param-fields {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .param-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .param-name {
    font-family: var(--font-body);
    font-size: var(--text-xs);
    color: var(--signal);
    min-width: 100px;
  }
  .param-input {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    background: var(--void);
    border: 1px solid var(--border-strong);
    color: var(--ink);
    padding: 6px 10px;
    outline: none;
    flex: 1;
  }
  .param-input:focus { border-color: var(--pulse); }
  .param-input::placeholder { color: var(--ink-ghost); }
  .body-editor {
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
  .body-editor:focus { border-color: var(--pulse); }
  .auth-note {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-muted);
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .response-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .response-header {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .response-label {
    font-family: var(--font-display);
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .response-time {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-muted);
    margin-left: auto;
  }

  @media (max-width: 768px) {
    .api-page { grid-template-columns: 1fr; }
    .endpoint-list { max-height: 200px; }
  }
</style>
