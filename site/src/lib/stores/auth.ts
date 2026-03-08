import { writable } from 'svelte/store';
import { browser } from '$app/environment';

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
}

export function clearCredentials() {
  uuid.set('');
  token.set('');
  authenticated.set(false);
}
