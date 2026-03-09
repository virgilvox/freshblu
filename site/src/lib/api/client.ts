import { FreshBlu, FreshBluHttp } from 'freshblu';
export { FreshBlu as FreshBluClient, FreshBluHttp };
export type {
  Device,
  RegisterResponse,
  Message,
  Subscription,
  SubscriptionType,
  StatusResponse,
  Whitelists,
  WhitelistEntry,
  Forwarder,
  Forwarders,
} from 'freshblu';

const defaultUrl = 'https://api.freshblu.org';

export const api = new FreshBluHttp(defaultUrl);

/** Sync the singleton api client with the user's localStorage serverUrl (only if user explicitly customized it) */
export function syncApiBaseUrl() {
  if (typeof localStorage !== 'undefined') {
    const url = localStorage.getItem('freshblu_server_url');
    if (url) api.setBaseUrl(url);
  }
}

/** Get the effective server URL: localStorage override or PUBLIC_API_URL */
export function getServerUrl(): string {
  if (typeof localStorage !== 'undefined') {
    const stored = localStorage.getItem('freshblu_server_url');
    if (stored) return stored;
  }
  return defaultUrl;
}

/** Save server URL to localStorage only if it differs from the build-time default */
export function saveServerUrl(url: string) {
  if (url === defaultUrl) {
    localStorage.removeItem('freshblu_server_url');
  } else {
    localStorage.setItem('freshblu_server_url', url);
  }
  api.setBaseUrl(url);
}
