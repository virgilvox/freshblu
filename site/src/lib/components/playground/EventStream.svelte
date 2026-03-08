<script lang="ts">
  import Badge from '../ui/Badge.svelte';
  import { events, type EventItem } from '$lib/stores/events';

  let items: EventItem[] = $state([]);
  events.subscribe(v => items = v);

  function formatTime(d: Date): string {
    return d.toLocaleTimeString('en-US', { hour12: false, fractionalSecondDigits: 3 });
  }

  function eventVariant(type: string): 'online' | 'pulse' | 'warn' | 'fault' | 'pending' | 'muted' {
    switch (type) {
      case 'message': return 'pulse';
      case 'broadcast': return 'pending';
      case 'config': return 'warn';
      case 'ready': return 'online';
      case 'notReady': return 'fault';
      case 'unregistered': return 'fault';
      default: return 'muted';
    }
  }
</script>

<div class="event-stream">
  <div class="stream-header">
    <span class="stream-title">Event Stream</span>
    <span class="stream-count">{items.length} events</span>
    <button class="stream-clear" onclick={() => events.clear()}>Clear</button>
  </div>
  <div class="stream-body">
    {#each items.toReversed() as item (item.id)}
      <div class="stream-event">
        <span class="event-time">{formatTime(item.timestamp)}</span>
        <Badge variant={eventVariant(item.type)}>{item.type}</Badge>
        {#if item.fromUuid}
          <span class="event-from">{item.fromUuid.substring(0, 8)}</span>
        {/if}
        <code class="event-data">{JSON.stringify(item.data)}</code>
      </div>
    {/each}
    {#if items.length === 0}
      <div class="stream-empty">No events yet. Connect via WebSocket to see live events.</div>
    {/if}
  </div>
</div>

<style>
  .event-stream {
    border: 1px solid var(--border);
  }
  .stream-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    background: var(--void-lift);
    border-bottom: 1px solid var(--border);
  }
  .stream-title {
    font-family: var(--font-display);
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
  }
  .stream-count {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    color: var(--ink-muted);
    margin-left: auto;
  }
  .stream-clear {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    background: none;
    border: 1px solid var(--border);
    color: var(--ink-muted);
    padding: 3px 8px;
    cursor: pointer;
    transition: color var(--dur-fast), border-color var(--dur-fast);
  }
  .stream-clear:hover { color: var(--fault); border-color: var(--fault); }
  .stream-body {
    max-height: 400px;
    overflow-y: auto;
  }
  .stream-event {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 16px;
    border-bottom: 1px solid var(--border);
    font-size: var(--text-xs);
  }
  .stream-event:last-child { border-bottom: none; }
  .event-time {
    font-family: var(--font-ui);
    color: var(--ink-muted);
    flex-shrink: 0;
  }
  .event-from {
    font-family: var(--font-ui);
    color: var(--signal);
  }
  .event-data {
    font-family: var(--font-body);
    color: var(--ink-soft);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }
  .stream-empty {
    padding: 24px 16px;
    text-align: center;
    color: var(--ink-muted);
    font-family: var(--font-ui);
    font-size: var(--text-xs);
  }
</style>
