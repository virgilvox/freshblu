import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';

export interface VaultDevice {
  uuid: string;
  token: string;
  label?: string;
  name?: string;
  type?: string;
  addedAt: number;
}

const DB_NAME = 'freshblu-vault';
const STORE_NAME = 'devices';
const ACTIVE_KEY = 'freshblu_active_uuid';
const PRIMARY_KEY = 'freshblu_primary_uuid';

export const vaultDevices = writable<VaultDevice[]>([]);
export const activeUuid = writable<string>('');
export const primaryUuid = writable<string>('');

let dbPromise: Promise<import('idb').IDBPDatabase> | null = null;

async function getDb() {
  if (!dbPromise) {
    const { openDB } = await import('idb');
    dbPromise = openDB(DB_NAME, 1, {
      upgrade(db) {
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          db.createObjectStore(STORE_NAME, { keyPath: 'uuid' });
        }
      },
    });
  }
  return dbPromise;
}

async function loadAll(): Promise<VaultDevice[]> {
  const db = await getDb();
  return db.getAll(STORE_NAME);
}

export async function initVault() {
  if (!browser) return;
  const devices = await loadAll();
  vaultDevices.set(devices);
  const stored = localStorage.getItem(ACTIVE_KEY);
  if (stored) activeUuid.set(stored);
  const primary = localStorage.getItem(PRIMARY_KEY);
  if (primary) primaryUuid.set(primary);
}

export async function addToVault(device: VaultDevice) {
  if (!browser) return;
  const db = await getDb();
  await db.put(STORE_NAME, device);
  vaultDevices.set(await loadAll());
}

export async function removeFromVault(uuid: string) {
  if (!browser) return;
  const db = await getDb();
  await db.delete(STORE_NAME, uuid);
  vaultDevices.set(await loadAll());
  if (get(activeUuid) === uuid) {
    activeUuid.set('');
    localStorage.removeItem(ACTIVE_KEY);
  }
  if (get(primaryUuid) === uuid) {
    primaryUuid.set('');
    localStorage.removeItem(PRIMARY_KEY);
    localStorage.removeItem('freshblu_primary_saved');
  }
}

export async function clearVault() {
  if (!browser) return;
  const db = await getDb();
  await db.clear(STORE_NAME);
  vaultDevices.set([]);
  activeUuid.set('');
  localStorage.removeItem(ACTIVE_KEY);
  primaryUuid.set('');
  localStorage.removeItem(PRIMARY_KEY);
  localStorage.removeItem('freshblu_primary_saved');
}

export function setActiveDevice(uuid: string) {
  if (!browser) return;
  activeUuid.set(uuid);
  localStorage.setItem(ACTIVE_KEY, uuid);
}

export function getActiveCredentials(): { uuid: string; token: string } | null {
  const devices = get(vaultDevices);
  const active = get(activeUuid);
  const device = devices.find((d) => d.uuid === active);
  if (!device) return null;
  return { uuid: device.uuid, token: device.token };
}

// -- Primary device helpers --

export function setPrimaryDevice(uuid: string) {
  if (!browser) return;
  primaryUuid.set(uuid);
  localStorage.setItem(PRIMARY_KEY, uuid);
}

export function getPrimaryDevice(): string | null {
  if (!browser) return null;
  return localStorage.getItem(PRIMARY_KEY);
}

export function clearPrimaryDevice() {
  if (!browser) return;
  primaryUuid.set('');
  localStorage.removeItem(PRIMARY_KEY);
  localStorage.removeItem('freshblu_primary_saved');
}

export function hasPrimaryDevice(): boolean {
  if (!browser) return false;
  return !!localStorage.getItem(PRIMARY_KEY);
}

export function getPrimaryCredentials(): { uuid: string; token: string } | null {
  const primary = getPrimaryDevice();
  if (!primary) return null;
  const devices = get(vaultDevices);
  const device = devices.find((d) => d.uuid === primary);
  if (!device) return null;
  return { uuid: device.uuid, token: device.token };
}
