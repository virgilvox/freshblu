<script lang="ts">
  import Button from '../ui/Button.svelte';
  import { api } from '$lib/api/client';

  interface Props {
    uuid: string;
    token?: string;
  }

  let { uuid, token }: Props = $props();
  let showToken = $state(false);
  let generatedToken = $state('');
  let copiedField = $state('');

  async function copy(text: string, field: string) {
    await navigator.clipboard.writeText(text);
    copiedField = field;
    setTimeout(() => copiedField = '', 2000);
  }

  async function generateToken() {
    const res = await api.generateToken(uuid);
    generatedToken = res.token;
  }

  async function resetToken() {
    if (!confirm('This will invalidate all existing tokens. Continue?')) return;
    const res = await api.resetToken(uuid);
    generatedToken = res.token;
  }
</script>

<div class="creds-panel">
  <div class="cred-row">
    <span class="cred-label">UUID</span>
    <code class="cred-value">{uuid}</code>
    <button class="cred-btn" onclick={() => copy(uuid, 'uuid')}>
      <i class="fa-solid {copiedField === 'uuid' ? 'fa-check' : 'fa-copy'}"></i>
    </button>
  </div>

  {#if token}
    <div class="cred-row">
      <span class="cred-label">Token</span>
      <code class="cred-value">{showToken ? token : '\u2022'.repeat(24)}</code>
      <button class="cred-btn" onclick={() => showToken = !showToken}>
        <i class="fa-solid {showToken ? 'fa-eye-slash' : 'fa-eye'}"></i>
      </button>
      <button class="cred-btn" onclick={() => copy(token!, 'token')}>
        <i class="fa-solid {copiedField === 'token' ? 'fa-check' : 'fa-copy'}"></i>
      </button>
    </div>
  {/if}

  {#if generatedToken}
    <div class="cred-row generated">
      <span class="cred-label">New Token</span>
      <code class="cred-value">{generatedToken}</code>
      <button class="cred-btn" onclick={() => copy(generatedToken, 'gen')}>
        <i class="fa-solid {copiedField === 'gen' ? 'fa-check' : 'fa-copy'}"></i>
      </button>
    </div>
  {/if}

  <div class="cred-actions">
    <Button size="sm" variant="ghost" onclick={generateToken}>Generate Token</Button>
    <Button size="sm" variant="muted" onclick={resetToken}>Reset Token</Button>
  </div>
</div>

<style>
  .creds-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .cred-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--void);
    border: 1px solid var(--border);
  }
  .cred-row.generated {
    border-color: var(--online);
  }
  .cred-label {
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-muted);
    width: 70px;
    flex-shrink: 0;
  }
  .cred-value {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--pulse);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cred-btn {
    background: none;
    border: none;
    color: var(--ink-muted);
    cursor: pointer;
    padding: 4px;
    font-size: var(--text-xs);
    transition: color var(--dur-fast);
  }
  .cred-btn:hover { color: var(--ink); }
  .cred-actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }
</style>
