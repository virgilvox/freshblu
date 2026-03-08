<script lang="ts">
  interface Props {
    code: string;
    lang?: string;
  }

  let { code, lang = '' }: Props = $props();
  let copied = $state(false);

  function copyCode() {
    navigator.clipboard.writeText(code);
    copied = true;
    setTimeout(() => copied = false, 2000);
  }
</script>

<div class="code-block-wrap">
  {#if lang}
    <div class="code-lang">{lang}</div>
  {/if}
  <button class="code-copy" onclick={copyCode} title="Copy">
    <i class="fa-solid {copied ? 'fa-check' : 'fa-copy'}"></i>
  </button>
  <pre class="code-block"><code>{code}</code></pre>
</div>

<style>
  .code-block-wrap {
    position: relative;
  }
  .code-block {
    background: var(--void);
    border: 1px solid var(--border);
    border-left: 3px solid var(--pulse);
    padding: 20px 24px;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--ink-soft);
    line-height: var(--leading-relaxed);
    overflow-x: auto;
    white-space: pre;
  }
  .code-lang {
    position: absolute;
    top: 0;
    left: 3px;
    font-family: var(--font-ui);
    font-size: 9px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-muted);
    background: var(--void);
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-top: none;
    border-left: none;
  }
  .code-copy {
    position: absolute;
    top: 8px;
    right: 8px;
    background: var(--void-lift);
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 4px 8px;
    cursor: pointer;
    font-size: var(--text-xs);
    transition: color var(--dur-fast), border-color var(--dur-fast);
  }
  .code-copy:hover {
    color: var(--pulse);
    border-color: var(--pulse);
  }
</style>
