import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { devices, selectedDevice } from './devices';

describe('devices store', () => {
  it('starts empty', () => {
    expect(get(devices)).toEqual([]);
    expect(get(selectedDevice)).toBeNull();
  });

  it('updates device list', () => {
    devices.set([{ uuid: 'a', online: true } as any]);
    expect(get(devices)).toHaveLength(1);
    devices.set([]);
  });

  it('sets selected device', () => {
    const dev = { uuid: 'b', name: 'test' } as any;
    selectedDevice.set(dev);
    expect(get(selectedDevice)?.uuid).toBe('b');
    selectedDevice.set(null);
  });
});
