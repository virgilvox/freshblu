<script lang="ts">
  interface Props {
    status?: 'online' | 'warn' | 'fault' | 'pending';
  }

  let { status = 'online' }: Props = $props();
</script>

<span class="status-sq sq-{status}"></span>

<style>
  .status-sq {
    display: inline-block;
    width: 8px;
    height: 8px;
    flex-shrink: 0;
  }
  .sq-online  { background: var(--online);  animation: blink 2.4s ease-in-out infinite; }
  .sq-warn    { background: var(--warn); }
  .sq-fault   { background: var(--fault); }
  .sq-pending { background: var(--pending); opacity: 0.7; }

  @keyframes blink {
    0%, 80%, 100% { opacity: 1; }
    40% { opacity: 0.3; }
  }
</style>
