import type { FreshBluOptions } from './core';

/** MQTT client (Node.js only, thin wrapper around mqtt.js) */
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
    const mqtt = await import('mqtt');
    this.client = mqtt.connect(this.brokerUrl, {
      username: this.options.uuid,
      password: this.options.token,
      clientId: this.options.uuid,
    });
  }
}
