import { writable } from 'svelte/store';

export interface EventItem {
  id: number;
  timestamp: Date;
  type: string;
  fromUuid?: string;
  data: Record<string, unknown>;
}

let nextId = 0;

export function createEventStore() {
  const { subscribe, update } = writable<EventItem[]>([]);

  return {
    subscribe,
    push(type: string, data: Record<string, unknown>, fromUuid?: string) {
      update((items) => {
        const next = [...items, { id: nextId++, timestamp: new Date(), type, fromUuid, data }];
        if (next.length > 200) next.shift();
        return next;
      });
    },
    clear() {
      update(() => []);
    }
  };
}

export const events = createEventStore();
