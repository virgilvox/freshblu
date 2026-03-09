<script lang="ts">
  import Modal from '../ui/Modal.svelte';
  import Input from '../ui/Input.svelte';
  import Button from '../ui/Button.svelte';
  import IconPicker from './IconPicker.svelte';

  interface Props {
    open: boolean;
    /** UUID of the active/primary device that will claim ownership, if any */
    ownerUuid?: string;
    /** Display name for the owner device */
    ownerLabel?: string;
    onregister: (properties: Record<string, unknown>, claimOwnership: boolean) => void;
    onclose?: () => void;
  }

  let { open = $bindable(false), ownerUuid, ownerLabel, onregister, onclose }: Props = $props();

  let name = $state('');
  let type = $state('');
  let icon = $state('');
  let claimOwnership = $state(true);

  // --- Name generator ---
  const adjectives = [
    'azure', 'crimson', 'silent', 'swift', 'lunar', 'solar', 'neon', 'frost',
    'amber', 'cobalt', 'iron', 'coral', 'onyx', 'jade', 'chrome', 'pixel',
    'pulse', 'drift', 'arc', 'flux', 'zen', 'nova', 'echo', 'void',
  ];
  const nouns = [
    'sensor', 'relay', 'beacon', 'node', 'probe', 'hub', 'gate', 'link',
    'core', 'cell', 'mesh', 'shard', 'spark', 'bolt', 'wave', 'prism',
    'nexus', 'lens', 'drone', 'matrix', 'coil', 'fuse', 'grid', 'chip',
  ];

  function generateName() {
    const adj = adjectives[Math.floor(Math.random() * adjectives.length)];
    const noun = nouns[Math.floor(Math.random() * nouns.length)];
    const suffix = Math.floor(Math.random() * 100).toString().padStart(2, '0');
    name = `${adj}-${noun}-${suffix}`;
  }

  const typePresets = ['sensor', 'actuator', 'gateway', 'controller', 'display', 'camera', 'thermostat'];

  function handleSubmit() {
    const props: Record<string, unknown> = {};
    if (name.trim()) props.name = name.trim();
    if (type.trim()) props.type = type.trim();
    if (icon) props.icon = icon;
    onregister(props, claimOwnership && !!ownerUuid);
    // Reset form
    name = '';
    type = '';
    icon = '';
    claimOwnership = true;
  }

  function handleClose() {
    open = false;
    onclose?.();
  }
</script>

<Modal bind:open title="Register Device" onclose={handleClose}>
  <div class="register-form">
    <div class="name-row">
      <Input label="Name" bind:value={name} placeholder="e.g. living-room-sensor" />
      <button class="generate-btn" onclick={generateName} title="Generate random name" type="button">
        <i class="fa-solid fa-dice"></i>
      </button>
    </div>

    <div class="type-field">
      <Input label="Type" bind:value={type} placeholder="e.g. sensor" />
      <div class="type-presets">
        {#each typePresets as preset}
          <button
            class="preset-btn"
            class:active={type === preset}
            onclick={() => type = preset}
            type="button"
          >{preset}</button>
        {/each}
      </div>
    </div>

    <div class="icon-field">
      <span class="field-label">Icon</span>
      <IconPicker selected={icon} onSelect={(i) => icon = i} />
    </div>

    {#if ownerUuid}
      <label class="ownership-row">
        <input type="checkbox" bind:checked={claimOwnership} />
        <span class="ownership-text">
          Owned by <strong>{ownerLabel || ownerUuid.substring(0, 8)}</strong>
        </span>
      </label>
    {/if}

    <div class="form-actions">
      <Button size="sm" variant="muted" onclick={handleClose}>Cancel</Button>
      <Button size="sm" onclick={handleSubmit}>
        <i class="fa-solid fa-plus"></i>
        Register
      </Button>
    </div>
  </div>
</Modal>

<style>
  .register-form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .name-row {
    display: flex;
    gap: 8px;
    align-items: flex-end;
  }
  .name-row :global(.field) {
    flex: 1;
  }
  .generate-btn {
    background: var(--void);
    border: 1px solid var(--border-strong);
    color: var(--ink-muted);
    padding: 10px 12px;
    cursor: pointer;
    font-size: var(--text-sm);
    transition: color var(--dur-fast), border-color var(--dur-fast);
    flex-shrink: 0;
  }
  .generate-btn:hover {
    color: var(--pulse);
    border-color: var(--pulse);
  }
  .type-field {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .type-presets {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .preset-btn {
    font-family: var(--font-ui);
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    background: var(--void);
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 3px 8px;
    cursor: pointer;
    transition: color var(--dur-fast), border-color var(--dur-fast), background var(--dur-fast);
  }
  .preset-btn:hover {
    color: var(--pulse);
    border-color: var(--pulse);
  }
  .preset-btn.active {
    color: var(--pulse);
    border-color: var(--pulse);
    background: var(--void-lift);
  }
  .icon-field {
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
  .ownership-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--void);
    border: 1px solid var(--border);
    cursor: pointer;
  }
  .ownership-row input[type="checkbox"] {
    accent-color: var(--pulse);
  }
  .ownership-text {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-soft);
    letter-spacing: 0.04em;
  }
  .ownership-text strong {
    color: var(--pulse);
  }
  .form-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }
</style>
