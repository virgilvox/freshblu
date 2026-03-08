import { PUBLIC_API_URL } from '$env/static/public';
import type {
  Device,
  RegisterResponse,
  Message,
  Subscription,
  SubscriptionType,
  StatusResponse,
} from './types';

class FreshBluClient {
  private baseUrl: string;
  private uuid = '';
  private token = '';

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || PUBLIC_API_URL || 'http://localhost:3000';
  }

  setBaseUrl(url: string) {
    this.baseUrl = url;
  }

  setCredentials(uuid: string, token: string) {
    this.uuid = uuid;
    this.token = token;
  }

  private headers(): Record<string, string> {
    const h: Record<string, string> = { 'Content-Type': 'application/json' };
    if (this.uuid && this.token) {
      h['Authorization'] = 'Basic ' + btoa(this.uuid + ':' + this.token);
    }
    return h;
  }

  private async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const res = await fetch(this.baseUrl + path, {
      method,
      headers: this.headers(),
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(err.error || res.statusText);
    }
    if (res.status === 204) return undefined as T;
    return res.json();
  }

  // Status
  status(): Promise<StatusResponse> {
    return this.request('GET', '/status');
  }

  // Auth
  authenticate(): Promise<Device> {
    return this.request('POST', '/authenticate', { uuid: this.uuid, token: this.token });
  }

  // Devices
  register(params?: Record<string, unknown>): Promise<RegisterResponse> {
    return this.request('POST', '/devices', params || {});
  }

  getDevice(uuid: string): Promise<Device> {
    return this.request('GET', `/devices/${uuid}`);
  }

  updateDevice(uuid: string, properties: Record<string, unknown>): Promise<Device> {
    return this.request('PUT', `/devices/${uuid}`, properties);
  }

  unregister(uuid: string): Promise<void> {
    return this.request('DELETE', `/devices/${uuid}`);
  }

  whoami(): Promise<Device> {
    return this.request('GET', '/whoami');
  }

  myDevices(): Promise<Device[]> {
    return this.request('GET', '/mydevices');
  }

  searchDevices(query?: Record<string, unknown>): Promise<Device[]> {
    return this.request('POST', '/devices/search', query || {});
  }

  claimDevice(uuid: string): Promise<Device> {
    return this.request('POST', `/claimdevice/${uuid}`);
  }

  // Messages
  sendMessage(params: { devices: string[]; topic?: string; payload?: unknown }): Promise<void> {
    return this.request('POST', '/messages', params);
  }

  broadcast(params: { topic?: string; payload?: unknown }): Promise<void> {
    return this.request('POST', '/broadcasts', params);
  }

  // Subscriptions
  createSubscription(uuid: string, emitterUuid: string, type: SubscriptionType): Promise<Subscription> {
    return this.request('POST', `/devices/${uuid}/subscriptions`, { emitterUuid, type });
  }

  listSubscriptions(uuid: string): Promise<Subscription[]> {
    return this.request('GET', `/devices/${uuid}/subscriptions`);
  }

  deleteSubscription(uuid: string, emitterUuid: string, type: SubscriptionType): Promise<void> {
    return this.request('DELETE', `/devices/${uuid}/subscriptions/${emitterUuid}/${type}`);
  }

  // Tokens
  generateToken(uuid: string): Promise<{ uuid: string; token: string }> {
    return this.request('POST', `/devices/${uuid}/tokens`);
  }

  revokeToken(uuid: string, token: string): Promise<void> {
    return this.request('DELETE', `/devices/${uuid}/tokens/${token}`);
  }

  resetToken(uuid: string): Promise<{ uuid: string; token: string }> {
    return this.request('POST', `/devices/${uuid}/token`);
  }
}

export const api = new FreshBluClient();
export { FreshBluClient };

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
  return PUBLIC_API_URL || 'http://localhost:3000';
}

/** Save server URL to localStorage only if it differs from the build-time default */
export function saveServerUrl(url: string) {
  const defaultUrl = PUBLIC_API_URL || 'http://localhost:3000';
  if (url === defaultUrl) {
    localStorage.removeItem('freshblu_server_url');
  } else {
    localStorage.setItem('freshblu_server_url', url);
  }
  api.setBaseUrl(url);
}
