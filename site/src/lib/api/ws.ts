import { PUBLIC_API_URL } from '$env/static/public';

export type WsEventType = 'ready' | 'notReady' | 'message' | 'config' | 'broadcast' | 'unregistered';

export interface WsEvent {
  event: WsEventType;
  [key: string]: unknown;
}

export type WsEventHandler = (event: WsEvent) => void;

export class FreshBluWs {
  private ws: WebSocket | null = null;
  private handlers: Map<string, WsEventHandler[]> = new Map();
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(
    private uuid: string,
    private token: string,
    private baseUrl?: string,
  ) {}

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const url = (this.baseUrl || PUBLIC_API_URL || 'http://localhost:3000')
        .replace(/^http/, 'ws') + '/ws';

      this.ws = new WebSocket(url);

      this.ws.onopen = () => {
        this.send({ event: 'identity', uuid: this.uuid, token: this.token });
      };

      this.ws.onmessage = (e) => {
        const data = JSON.parse(e.data) as WsEvent;
        if (data.event === 'ready') {
          resolve();
        } else if (data.event === 'notReady') {
          reject(new Error((data as Record<string, unknown>).reason as string || 'Authentication failed'));
          this.close();
          return;
        }
        this.emit(data.event, data);
      };

      this.ws.onclose = () => {
        this.emit('close', { event: 'close' as WsEventType });
      };

      this.ws.onerror = () => {
        reject(new Error('WebSocket connection failed'));
      };
    });
  }

  on(event: string, handler: WsEventHandler) {
    if (!this.handlers.has(event)) {
      this.handlers.set(event, []);
    }
    this.handlers.get(event)!.push(handler);
  }

  off(event: string, handler: WsEventHandler) {
    const list = this.handlers.get(event);
    if (list) {
      this.handlers.set(event, list.filter(h => h !== handler));
    }
  }

  private emit(event: string, data: WsEvent) {
    const list = this.handlers.get(event);
    if (list) {
      for (const handler of list) handler(data);
    }
    const allList = this.handlers.get('*');
    if (allList) {
      for (const handler of allList) handler(data);
    }
  }

  send(data: Record<string, unknown>) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    }
  }

  sendMessage(devices: string[], payload?: unknown, topic?: string) {
    this.send({ event: 'message', devices, payload, topic });
  }

  close() {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.ws?.close();
    this.ws = null;
  }

  get connected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}
