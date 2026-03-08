<script lang="ts">
  import Button from '../ui/Button.svelte';
  import Input from '../ui/Input.svelte';

  interface Props {
    properties: Record<string, unknown>;
    onSave: (props: Record<string, unknown>) => void;
  }

  let { properties, onSave }: Props = $props();
  let jsonText = $state(JSON.stringify(properties, null, 2));
  let parseError = $state('');

  function handleSave() {
    try {
      const parsed = JSON.parse(jsonText);
      parseError = '';
      onSave(parsed);
    } catch {
      parseError = 'Invalid JSON';
    }
  }
</script>

<div class="prop-editor">
  <textarea
    class="prop-textarea"
    bind:value={jsonText}
    rows={10}
    spellcheck="false"
  ></textarea>
  {#if parseError}
    <span class="prop-error">{parseError}</span>
  {/if}
  <div class="prop-actions">
    <Button size="sm" onclick={handleSave}>Save Properties</Button>
  </div>
</div>

<style>
  .prop-editor {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .prop-textarea {
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
  .prop-textarea:focus {
    border-color: var(--pulse);
    box-shadow: 0 0 0 2px var(--pulse-ring);
  }
  .prop-error {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--fault);
  }
  .prop-actions {
    display: flex;
    gap: 8px;
  }
</style>
