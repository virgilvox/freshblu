import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { events } from './events';

describe('events store', () => {
  it('starts empty', () => {
    events.clear();
    expect(get(events)).toHaveLength(0);
  });

  it('pushes events', () => {
    events.clear();
    events.push('message', { payload: 'hello' }, 'sender-1');
    const items = get(events);
    expect(items).toHaveLength(1);
    expect(items[0].type).toBe('message');
    expect(items[0].fromUuid).toBe('sender-1');
    expect(items[0].data.payload).toBe('hello');
  });

  it('caps at 200 events', () => {
    events.clear();
    for (let i = 0; i < 210; i++) {
      events.push('message', { i });
    }
    expect(get(events)).toHaveLength(200);
  });

  it('clears events', () => {
    events.push('test', {});
    events.clear();
    expect(get(events)).toHaveLength(0);
  });
});
