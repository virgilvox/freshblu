<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open: boolean;
    title?: string;
    onclose?: () => void;
    children: Snippet;
  }

  let { open = $bindable(false), title, onclose, children }: Props = $props();

  function handleClose() {
    open = false;
    onclose?.();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }

  function trapFocus(node: HTMLElement) {
    const focusable = () =>
      node.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );

    function handleTrap(e: KeyboardEvent) {
      if (e.key !== 'Tab') return;
      const els = focusable();
      if (els.length === 0) return;
      const first = els[0];
      const last = els[els.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }

    node.addEventListener('keydown', handleTrap);
    // Auto-focus first focusable element
    requestAnimationFrame(() => {
      const els = focusable();
      if (els.length > 0) els[0].focus();
    });

    return {
      destroy() {
        node.removeEventListener('keydown', handleTrap);
      }
    };
  }
</script>

{#if open}
  <div class="modal-overlay" onkeydown={handleKeydown}>
    <button class="modal-backdrop" onclick={handleClose} tabindex="-1" aria-label="Close"></button>
    <div class="modal-panel" role="dialog" aria-modal="true" aria-labelledby="modal-title" use:trapFocus>
      {#if title}
        <div class="modal-header">
          <span class="modal-title" id="modal-title">{title}</span>
          <button class="modal-close" onclick={handleClose} aria-label="Close">
            <i class="fa-solid fa-xmark"></i>
          </button>
        </div>
      {/if}
      <div class="modal-body">
        {@render children()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .modal-backdrop {
    position: absolute;
    inset: 0;
    background: rgba(6,8,16,0.85);
    border: none;
    cursor: default;
  }
  .modal-panel {
    position: relative;
    background: var(--void-high);
    border: 1px solid var(--border-strong);
    min-width: min(400px, 90vw);
    max-width: 90vw;
    max-height: 90vh;
    overflow-y: auto;
  }
  .modal-header {
    padding: 14px 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .modal-title {
    font-family: var(--font-display);
    font-size: var(--text-base);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .modal-close {
    background: none;
    border: none;
    color: var(--ink-muted);
    cursor: pointer;
    padding: 4px;
    font-size: var(--text-md);
    transition: color var(--dur-fast);
  }
  .modal-close:hover { color: var(--ink); }
  .modal-body { padding: 20px; }
</style>
