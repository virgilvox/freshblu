import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';
import { addToVault, setActiveDevice, initVault } from './vault';

function stored<T>(key: string, initial: T) {
  const value = browser ? localStorage.getItem(key) : null;
  const store = writable<T>(value ? JSON.parse(value) : initial);
  if (browser) {
    store.subscribe((v) => {
      localStorage.setItem(key, JSON.stringify(v));
    });
  }
  return store;
}

export const uuid = stored<string>('freshblu_uuid', '');
export const token = stored<string>('freshblu_token', '');
export const authenticated = writable(false);

export function setCredentials(u: string, t: string) {
  uuid.set(u);
  token.set(t);
  authenticated.set(true);
  addToVault({ uuid: u, token: t, addedAt: Date.now() });
  setActiveDevice(u);
}

export function clearCredentials() {
  uuid.set('');
  token.set('');
  authenticated.set(false);
}

/** Migrate existing localStorage credentials into vault on first load */
export async function migrateToVault() {
  if (!browser) return;
  await initVault();
  const u = get(uuid);
  const t = get(token);
  if (u && t) {
    await addToVault({ uuid: u, token: t, addedAt: Date.now() });
    setActiveDevice(u);
  }
}
