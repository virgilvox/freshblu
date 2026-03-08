<script lang="ts">
  import { onMount } from 'svelte';
  import hljs from 'highlight.js/lib/core';
  import bash from 'highlight.js/lib/languages/bash';
  import json from 'highlight.js/lib/languages/json';
  import javascript from 'highlight.js/lib/languages/javascript';
  import typescript from 'highlight.js/lib/languages/typescript';
  import rust from 'highlight.js/lib/languages/rust';

  hljs.registerLanguage('bash', bash);
  hljs.registerLanguage('json', json);
  hljs.registerLanguage('javascript', javascript);
  hljs.registerLanguage('typescript', typescript);
  hljs.registerLanguage('rust', rust);

  interface Props {
    code: string;
    lang?: string;
  }

  let { code, lang = '' }: Props = $props();
  let copied = $state(false);
  let highlighted = $state('');

  onMount(() => {
    if (lang && hljs.getLanguage(lang)) {
      highlighted = hljs.highlight(code, { language: lang }).value;
    }
  });

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
  {#if highlighted}
    <pre class="code-block"><code class="hljs">{@html highlighted}</code></pre>
  {:else}
    <pre class="code-block"><code>{code}</code></pre>
  {/if}
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

  /* highlight.js theme - matched to void/pulse palette */
  :global(.hljs) {
    color: var(--ink-soft);
  }
  :global(.hljs-keyword),
  :global(.hljs-selector-tag),
  :global(.hljs-built_in),
  :global(.hljs-name) {
    color: var(--pulse);
  }
  :global(.hljs-string),
  :global(.hljs-attr) {
    color: var(--signal);
  }
  :global(.hljs-number),
  :global(.hljs-literal) {
    color: #d19a66;
  }
  :global(.hljs-comment) {
    color: var(--ink-muted);
    font-style: italic;
  }
  :global(.hljs-variable),
  :global(.hljs-template-variable),
  :global(.hljs-params) {
    color: #e5c07b;
  }
  :global(.hljs-type),
  :global(.hljs-title) {
    color: #61afef;
  }
  :global(.hljs-function) {
    color: #61afef;
  }
  :global(.hljs-meta) {
    color: var(--ink-muted);
  }
  :global(.hljs-punctuation),
  :global(.hljs-operator) {
    color: var(--ink-soft);
  }
  :global(.hljs-property) {
    color: #e06c75;
  }
  :global(.hljs-section) {
    color: var(--pulse);
  }
</style>
