/**
 * FreshBlu TypeScript SDK
 * 
 * Drop-in replacement for meshblu-http and meshblu (socket.io) clients.
 * Works in browser (fetch + WebSocket) and Node.js.
 * 
 * Usage:
 *   import { FreshBlu } from 'freshblu';
 *   
 *   const client = new FreshBlu({ hostname: 'localhost', port: 3000 });
 *   const device = await client.register({ type: 'sensor' });
 *   await client.message({ devices: ['*'], payload: { temp: 72.4 } });
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

// ---- HTTP Client ----

export class FreshBluHttp {
  private baseUrl: string;
  private uuid?: string;
  private token?: string;

  constructor(private options: FreshBluOptions = {}) {
    const {
      hostname = 'localhost',
      port = 3000,
      secure = false,
      uuid,
      token,
    } = options;
    const scheme = secure ? 'https' : 'http';
    this.baseUrl = `${scheme}://${hostname}:${port}`;
    this.uuid = uuid;
    this.token = token;
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

    return resp.json();
  }

  /** Register a new device */
  async register(properties: Record<string, unknown> = {}): Promise<RegisterResponse> {
    return this.request<RegisterResponse>('POST', '/devices', properties);
  }

  /** Get authenticated device info */
  async whoami(): Promise<Device> {
    return this.request<Device>('GET', '/whoami');
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
  async unregister(uuid: string): Promise<{ uuid: string }> {
    return this.request('DELETE', `/devices/${uuid}`);
  }

  /** Search for devices */
  async search(query: Record<string, unknown> = {}): Promise<Device[]> {
    return this.request<Device[]>('POST', '/devices/search', query);
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

  /** Get server status */
  async status(): Promise<{ meshblu: boolean; version: string; connections: number }> {
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
  private wsUrl: string;
  private listeners: Map<string, EventListener<unknown>[]> = new Map();
  private connected = false;
  private pendingReconnect?: ReturnType<typeof setTimeout>;

  constructor(private wsOptions: FreshBluOptions = {}) {
    super(wsOptions);
    const {
      hostname = 'localhost',
      port = 3000,
      secure = false,
    } = wsOptions;
    const scheme = secure ? 'wss' : 'ws';
    this.wsUrl = `${scheme}://${hostname}:${port}/ws`;
  }

  /** Connect to the WebSocket server and authenticate */
  connect(callback?: () => void): void {
    this.ws = new WebSocket(this.wsUrl);

    this.ws.onopen = () => {
      // Send identity message
      this.ws!.send(JSON.stringify({
        event: 'identity',
        uuid: this.wsOptions.uuid,
        token: this.wsOptions.token,
      }));
    };

    this.ws.onmessage = (evt) => {
      try {
        const data = JSON.parse(evt.data);
        const event = data.event;
        if (event) {
          this.emit(event, data);
          if (event === 'ready' && callback) {
            this.connected = true;
            callback();
          }
        }
      } catch (e) {
        // ignore parse errors
      }
    };

    this.ws.onclose = () => {
      this.connected = false;
    };

    this.ws.onerror = (e) => {
      this.emit('error', e);
    };
  }

  /** Listen for events */
  on<K extends keyof DeviceEventMap>(
    event: K,
    listener: EventListener<DeviceEventMap[K]>
  ): this {
    const list = this.listeners.get(event) || [];
    list.push(listener as EventListener<unknown>);
    this.listeners.set(event, list);
    return this;
  }

  /** Remove a listener */
  off<K extends keyof DeviceEventMap>(event: K, listener: Function): this {
    const list = this.listeners.get(event) || [];
    this.listeners.set(
      event,
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
  }

  /** Send a WebSocket command */
  private send(data: Record<string, unknown>): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    } else {
      throw new Error('Not connected');
    }
  }

  /** Send a message over WebSocket */
  sendMessage(msg: {
    devices: string[];
    topic?: string;
    payload?: unknown;
    [key: string]: unknown;
  }): void {
    this.send({ event: 'message', ...msg });
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
  }
}

// ---- MQTT client (Node.js only, thin wrapper) ----

export class FreshBluMqtt {
  private client?: unknown;
  private brokerUrl: string;

  constructor(private options: FreshBluOptions & { mqttPort?: number } = {}) {
    const {
      hostname = 'localhost',
      mqttPort = 1883,
      secure = false,
    } = options;
    this.brokerUrl = `${secure ? 'mqtts' : 'mqtt'}://${hostname}:${mqttPort}`;
  }

  /**
   * Connect using MQTT.js (Node.js).
   * Install: npm install mqtt
   */
  async connect(): Promise<void> {
    // Dynamic import to avoid browser bundle issues
    const mqtt = await import('mqtt');
    this.client = mqtt.connect(this.brokerUrl, {
      username: this.options.uuid,
      password: this.options.token,
      clientId: this.options.uuid,
    });
  }
}

// ---- Default export ----

export default FreshBlu;

// ---- Convenience factory ----

export function createClient(options: FreshBluOptions): FreshBlu {
  return new FreshBlu(options);
}

export function createHttpClient(options: FreshBluOptions): FreshBluHttp {
  return new FreshBluHttp(options);
}
