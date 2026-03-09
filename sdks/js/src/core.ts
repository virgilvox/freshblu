/**
 * FreshBlu TypeScript SDK
 *
 * Drop-in replacement for meshblu-http and meshblu (socket.io) clients.
 * Works in browser (fetch + WebSocket) and Node.js.
 *
 * Usage:
 *   import { FreshBlu } from 'freshblu';
 *
 *   const client = new FreshBlu('https://api.freshblu.org');
 *   const device = await client.register({ type: 'sensor' });
 *   await client.connect();
 */

export interface FreshBluOptions {
  hostname?: string;
  port?: number;
  secure?: boolean;
  uuid?: string;
  token?: string;
  /** Auto-resolve SRV record (legacy Octoblu compat) */
  resolveSrv?: boolean;
}

export interface Forwarder {
  url: string;
  method: string;
  type: string;
}

export interface Forwarders {
  broadcast?: { received?: Forwarder[]; sent?: Forwarder[] };
  configure?: { received?: Forwarder[]; sent?: Forwarder[] };
  message?: { received?: Forwarder[]; sent?: Forwarder[] };
  unregister?: { received?: Forwarder[]; sent?: Forwarder[] };
}

export interface Device {
  uuid: string;
  online: boolean;
  type?: string;
  meshblu: {
    version: string;
    createdAt: string;
    updatedAt?: string;
    hash: string;
    whitelists: Whitelists;
    forwarders?: Forwarders;
    publicKey?: string;
    owner?: string;
  };
  [key: string]: unknown;
}

export interface RegisterResponse extends Device {
  /** Plaintext token - only returned once, save it */
  token: string;
}

export interface Whitelists {
  discover?: { view?: WhitelistEntry[]; as?: WhitelistEntry[] };
  configure?: {
    update?: WhitelistEntry[];
    sent?: WhitelistEntry[];
    received?: WhitelistEntry[];
    as?: WhitelistEntry[];
  };
  message?: {
    from?: WhitelistEntry[];
    sent?: WhitelistEntry[];
    received?: WhitelistEntry[];
    as?: WhitelistEntry[];
  };
  broadcast?: {
    sent?: WhitelistEntry[];
    received?: WhitelistEntry[];
    as?: WhitelistEntry[];
  };
}

export interface WhitelistEntry {
  uuid: string;
}

export interface Message {
  devices: string[];
  fromUuid?: string;
  topic?: string;
  payload?: unknown;
  metadata?: { route: RouteHop[] };
  [key: string]: unknown;
}

export interface RouteHop {
  from: string;
  to: string;
  type: string;
}

export type SubscriptionType =
  | 'broadcast.sent'
  | 'broadcast.received'
  | 'configure.sent'
  | 'configure.received'
  | 'message.sent'
  | 'message.received'
  | 'unregister.sent'
  | 'unregister.received';

export interface Subscription {
  emitterUuid: string;
  subscriberUuid: string;
  type: SubscriptionType;
}

export interface TokenRecord {
  uuid: string;
  createdAt: string;
  expiresOn?: number;
  tag?: string;
}

export interface GenerateTokenOptions {
  expiresOn?: number;
  tag?: string;
}

export interface StatusResponse {
  meshblu: boolean;
  sky?: string;
  version?: string;
  connections?: number;
}

// ----- Event types for WebSocket client -----

export type DeviceEventMap = {
  ready: { uuid: string; fromUuid: string; meshblu: unknown };
  notReady: { reason: string };
  message: Message;
  broadcast: Message;
  config: { device: Device };
  unregistered: { uuid: string };
  pong: {};
};

type EventListener<T> = (event: T) => void;

// ---- Helper: parse base URL or options ----

function resolveBaseUrl(optionsOrUrl: FreshBluOptions | string): { baseUrl: string; uuid?: string; token?: string } {
  if (typeof optionsOrUrl === 'string') {
    // Strip trailing slash
    const baseUrl = optionsOrUrl.replace(/\/+$/, '');
    return { baseUrl };
  }
  const {
    hostname = 'localhost',
    port = 3000,
    secure = false,
    uuid,
    token,
  } = optionsOrUrl;
  const scheme = secure ? 'https' : 'http';
  return { baseUrl: `${scheme}://${hostname}:${port}`, uuid, token };
}

// ---- HTTP Client ----

export class FreshBluHttp {
  private baseUrl: string;
  protected uuid?: string;
  protected token?: string;

  constructor(optionsOrUrl: FreshBluOptions | string = {}) {
    const resolved = resolveBaseUrl(optionsOrUrl);
    this.baseUrl = resolved.baseUrl;
    this.uuid = resolved.uuid;
    this.token = resolved.token;
  }

  /** Change the base URL */
  setBaseUrl(url: string): void {
    this.baseUrl = url.replace(/\/+$/, '');
  }

  private authHeader(): string | undefined {
    if (!this.uuid || !this.token) return undefined;
    const creds = btoa(`${this.uuid}:${this.token}`);
    return `Basic ${creds}`;
  }

  private headers(extra?: Record<string, string>): Record<string, string> {
    const h: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    const auth = this.authHeader();
    if (auth) h['Authorization'] = auth;
    return { ...h, ...extra };
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    extraHeaders?: Record<string, string>
  ): Promise<T> {
    const resp = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers: this.headers(extraHeaders),
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });

    if (!resp.ok) {
      const err = await resp.json().catch(() => ({ error: resp.statusText }));
      throw new Error(err.error || `HTTP ${resp.status}`);
    }

    if (resp.status === 204) return undefined as T;
    return resp.json();
  }

  /** Authenticate with current credentials */
  async authenticate(): Promise<Device> {
    return this.request<Device>('POST', '/authenticate', { uuid: this.uuid, token: this.token });
  }

  /** Register a new device */
  async register(properties: Record<string, unknown> = {}): Promise<RegisterResponse> {
    return this.request<RegisterResponse>('POST', '/devices', properties);
  }

  /** Get authenticated device info */
  async whoami(): Promise<Device> {
    return this.request<Device>('GET', '/whoami');
  }

  /** Get devices owned by authenticated device */
  async myDevices(): Promise<Device[]> {
    return this.request<Device[]>('GET', '/mydevices');
  }

  /** Get a device by UUID */
  async getDevice(uuid: string, asUuid?: string): Promise<Device> {
    const headers = asUuid ? { 'x-meshblu-as': asUuid } : undefined;
    return this.request<Device>('GET', `/devices/${uuid}`, undefined, headers);
  }

  /** Update a device */
  async updateDevice(uuid: string, properties: Record<string, unknown>): Promise<Device> {
    return this.request<Device>('PUT', `/devices/${uuid}`, properties);
  }

  /** Unregister a device */
  async unregister(uuid: string): Promise<void> {
    await this.request('DELETE', `/devices/${uuid}`);
  }

  /** Search for devices */
  async search(query: Record<string, unknown> = {}): Promise<Device[]> {
    return this.request<Device[]>('POST', '/devices/search', query);
  }

  /** Claim a device by UUID */
  async claimDevice(uuid: string): Promise<Device> {
    return this.request<Device>('POST', `/claimdevice/${uuid}`);
  }

  /** Send a message */
  async message(msg: {
    devices: string[];
    topic?: string;
    payload?: unknown;
    [key: string]: unknown;
  }): Promise<{ sent: boolean }> {
    return this.request('POST', '/messages', msg);
  }

  /** Broadcast a message */
  async broadcast(msg: { topic?: string; payload?: unknown; [key: string]: unknown }): Promise<void> {
    await this.request('POST', '/broadcasts', msg);
  }

  /** Create a subscription */
  async createSubscription(params: Subscription): Promise<Subscription> {
    return this.request<Subscription>(
      'POST',
      `/devices/${params.subscriberUuid}/subscriptions`,
      params
    );
  }

  /** Delete a subscription */
  async deleteSubscription(
    subscriberUuid: string,
    emitterUuid: string,
    type: SubscriptionType
  ): Promise<void> {
    await this.request(
      'DELETE',
      `/devices/${subscriberUuid}/subscriptions/${emitterUuid}/${type.replace('.', '-')}`
    );
  }

  /** List subscriptions for a device */
  async subscriptions(subscriberUuid: string): Promise<Subscription[]> {
    return this.request<Subscription[]>('GET', `/devices/${subscriberUuid}/subscriptions`);
  }

  /** Generate a new token for a device */
  async generateToken(uuid: string, opts: GenerateTokenOptions = {}): Promise<TokenRecord & { token: string }> {
    return this.request('POST', `/devices/${uuid}/tokens`, opts);
  }

  /** Revoke a token */
  async revokeToken(uuid: string, token: string): Promise<void> {
    await this.request('DELETE', `/devices/${uuid}/tokens/${token}`);
  }

  /** Reset token for a device (revokes all existing, returns new one) */
  async resetToken(uuid: string): Promise<{ uuid: string; token: string }> {
    return this.request('POST', `/devices/${uuid}/token`);
  }

  /** Get server status */
  async status(): Promise<StatusResponse> {
    const resp = await fetch(`${this.baseUrl}/status`);
    return resp.json();
  }

  /** Set credentials (after registration) */
  setCredentials(uuid: string, token: string): void {
    this.uuid = uuid;
    this.token = token;
  }
}

// ---- WebSocket Client ----

export class FreshBlu extends FreshBluHttp {
  private ws?: WebSocket;
  private wsBaseUrl: string;
  private listeners: Map<string, EventListener<unknown>[]> = new Map();
  private _connected = false;

  constructor(optionsOrUrl: FreshBluOptions | string = {}) {
    super(optionsOrUrl);
    if (typeof optionsOrUrl === 'string') {
      // Derive WS URL from HTTP URL
      this.wsBaseUrl = optionsOrUrl.replace(/\/+$/, '').replace(/^http/, 'ws');
    } else {
      const {
        hostname = 'localhost',
        port = 3000,
        secure = false,
      } = optionsOrUrl;
      const scheme = secure ? 'wss' : 'ws';
      this.wsBaseUrl = `${scheme}://${hostname}:${port}`;
    }
  }

  /** Whether the WebSocket is currently connected and authenticated */
  get connected(): boolean {
    return this._connected;
  }

  /** Connect to the WebSocket server and authenticate. Resolves on 'ready', rejects on 'notReady' or error. */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const wsUrl = `${this.wsBaseUrl}/ws`;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        // Read credentials via a direct property access workaround:
        // We send identity using the credentials set on the parent FreshBluHttp
        this.ws!.send(JSON.stringify({
          event: 'identity',
          ...this.getCredentials(),
        }));
      };

      this.ws.onmessage = (evt) => {
        try {
          const data = JSON.parse(evt.data);
          const event = data.event;
          if (event) {
            this.emit(event, data);
            if (event === 'ready') {
              this._connected = true;
              resolve();
            } else if (event === 'notReady') {
              reject(new Error(data.reason || 'Authentication failed'));
              this.disconnect();
              return;
            }
          }
        } catch (e) {
          // ignore parse errors
        }
      };

      this.ws.onclose = () => {
        this._connected = false;
        this.emit('close', { event: 'close' });
      };

      this.ws.onerror = () => {
        reject(new Error('WebSocket connection failed'));
      };
    });
  }

  /** Listen for events. Use '*' to listen for all events. */
  on(event: keyof DeviceEventMap | '*' | string, listener: EventListener<any>): this {
    const list = this.listeners.get(event as string) || [];
    list.push(listener as EventListener<unknown>);
    this.listeners.set(event as string, list);
    return this;
  }

  /** Remove a listener */
  off(event: keyof DeviceEventMap | '*' | string, listener: Function): this {
    const list = this.listeners.get(event as string) || [];
    this.listeners.set(
      event as string,
      list.filter((l) => l !== listener)
    );
    return this;
  }

  private emit(event: string, data: unknown): void {
    const list = this.listeners.get(event) || [];
    for (const listener of list) {
      try {
        listener(data);
      } catch (e) {
        console.error('FreshBlu listener error:', e);
      }
    }
    // Wildcard listeners
    const allList = this.listeners.get('*');
    if (allList) {
      for (const listener of allList) {
        try {
          listener(data);
        } catch (e) {
          console.error('FreshBlu listener error:', e);
        }
      }
    }
  }

  /** Send a WebSocket command */
  send(data: Record<string, unknown>): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    } else {
      throw new Error('Not connected');
    }
  }

  /** Send a message over WebSocket */
  sendMessage(devices: string[], payload?: unknown, topic?: string): void;
  sendMessage(msg: { devices: string[]; topic?: string; payload?: unknown; [key: string]: unknown }): void;
  sendMessage(
    devicesOrMsg: string[] | { devices: string[]; topic?: string; payload?: unknown; [key: string]: unknown },
    payload?: unknown,
    topic?: string,
  ): void {
    if (Array.isArray(devicesOrMsg)) {
      this.send({ event: 'message', devices: devicesOrMsg, payload, topic });
    } else {
      this.send({ event: 'message', ...devicesOrMsg });
    }
  }

  /** Subscribe to events over WebSocket */
  subscribeWs(emitterUuid: string, type: SubscriptionType): void {
    this.send({ event: 'subscribe', emitterUuid, type });
  }

  /** Update device over WebSocket */
  updateWs(properties: Record<string, unknown>): void {
    this.send({ event: 'update', ...properties });
  }

  /** Request whoami over WebSocket */
  whoamiWs(): void {
    this.send({ event: 'whoami' });
  }

  /** Disconnect */
  disconnect(): void {
    this.ws?.close();
    this.ws = undefined;
    this._connected = false;
  }

  /** Alias for disconnect (compat with site's FreshBluWs.close) */
  close(): void {
    this.disconnect();
  }

  /** Get current credentials (used internally for WS identity) */
  private getCredentials(): { uuid?: string; token?: string } {
    return { uuid: this.uuid, token: this.token };
  }
}

// ---- Default export ----

export default FreshBlu;

// ---- Convenience factory ----

export function createClient(options: FreshBluOptions | string): FreshBlu {
  return new FreshBlu(options);
}

export function createHttpClient(options: FreshBluOptions | string): FreshBluHttp {
  return new FreshBluHttp(options);
}
