<script lang="ts">
  interface ToastItem {
    id: number;
    message: string;
    variant: 'info' | 'success' | 'error' | 'warn';
  }

  let toasts = $state<ToastItem[]>([]);
  let nextId = 0;

  export function show(message: string, variant: ToastItem['variant'] = 'info', duration = 3000) {
    const id = nextId++;
    toasts.push({ id, message, variant });
    setTimeout(() => {
      toasts = toasts.filter(t => t.id !== id);
    }, duration);
  }
</script>

{#if toasts.length > 0}
  <div class="toast-container">
    {#each toasts as toast (toast.id)}
      <div class="toast toast-{toast.variant}" role="alert" aria-live="polite">
        <span>{toast.message}</span>
        <button class="toast-close" onclick={() => toasts = toasts.filter(t => t.id !== toast.id)}>
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    top: 16px;
    right: 16px;
    z-index: 2000;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .toast {
    font-family: var(--font-ui);
    font-size: var(--text-sm);
    padding: 10px 16px;
    border: 1px solid;
    background: var(--void-high);
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 280px;
    max-width: min(400px, calc(100vw - 32px));
    animation: slide-in var(--dur-med) var(--ease-snap);
  }
  .toast-info    { border-color: var(--pulse);   color: var(--pulse); }
  .toast-success { border-color: var(--online);  color: var(--online); }
  .toast-error   { border-color: var(--fault);   color: var(--fault); }
  .toast-warn    { border-color: var(--warn);    color: var(--warn); }
  .toast-close {
    margin-left: auto;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    opacity: 0.6;
  }
  .toast-close:hover { opacity: 1; }

  @keyframes slide-in {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
</style>
