import { writable } from 'svelte/store';
import type { Device } from '$lib/api/types';

export const devices = writable<Device[]>([]);
export const selectedDevice = writable<Device | null>(null);
