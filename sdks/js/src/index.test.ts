import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { FreshBluHttp } from './core';

const BASE = 'http://localhost:3000';

const mockDevice = {
  uuid: 'test-uuid-1234',
  name: 'test-device',
  type: 'sensor',
  online: false,
  meshblu: { whitelists: {} },
};

const server = setupServer(
  http.get(`${BASE}/status`, () => HttpResponse.json({ meshblu: true, server: 'freshblu' })),
  http.post(`${BASE}/authenticate`, () => HttpResponse.json(mockDevice)),
  http.post(`${BASE}/devices`, () => HttpResponse.json({ uuid: mockDevice.uuid, token: 'new-token' })),
  http.get(`${BASE}/devices/:uuid`, ({ params }) => {
    if (params.uuid === 'not-found') return HttpResponse.json({ error: 'Not found' }, { status: 404 });
    return HttpResponse.json(mockDevice);
  }),
  http.put(`${BASE}/devices/:uuid`, () => HttpResponse.json({ ...mockDevice, name: 'updated' })),
  http.delete(`${BASE}/devices/:uuid`, () => new HttpResponse(null, { status: 204 })),
  http.get(`${BASE}/whoami`, () => HttpResponse.json(mockDevice)),
  http.get(`${BASE}/mydevices`, () => HttpResponse.json([mockDevice])),
  http.post(`${BASE}/devices/search`, () => HttpResponse.json([mockDevice])),
  http.post(`${BASE}/messages`, () => HttpResponse.json({ sent: true })),
  http.post(`${BASE}/broadcasts`, () => HttpResponse.json({ sent: true })),
  http.post(`${BASE}/claimdevice/:uuid`, () => HttpResponse.json(mockDevice)),
  http.post(`${BASE}/devices/:uuid/subscriptions`, () =>
    HttpResponse.json({ subscriberUuid: 'test-uuid-1234', emitterUuid: 'emitter-1', type: 'broadcast-sent' })),
  http.get(`${BASE}/devices/:uuid/subscriptions`, () => HttpResponse.json([])),
  http.delete(`${BASE}/devices/:uuid/subscriptions/:emitter/:type`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/devices/:uuid/tokens`, () => HttpResponse.json({ uuid: 'test-uuid-1234', token: 'gen-token' })),
  http.delete(`${BASE}/devices/:uuid/tokens/:token`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/devices/:uuid/token`, () => HttpResponse.json({ uuid: 'test-uuid-1234', token: 'reset-token' })),
);

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('FreshBluHttp', () => {
  describe('options constructor', () => {
    const client = new FreshBluHttp({ hostname: 'localhost', port: 3000 });

    it('fetches server status', async () => {
      const status = await client.status();
      expect(status.meshblu).toBe(true);
    });

    it('registers a new device', async () => {
      const res = await client.register();
      expect(res.uuid).toBe('test-uuid-1234');
      expect(res.token).toBe('new-token');
    });
  });

  describe('URL string constructor', () => {
    const client = new FreshBluHttp(BASE);

    it('sets credentials and authenticates', async () => {
      client.setCredentials('test-uuid', 'test-token');
      const device = await client.authenticate();
      expect(device.uuid).toBe('test-uuid-1234');
    });

    it('fetches server status', async () => {
      const status = await client.status();
      expect(status.meshblu).toBe(true);
    });

    it('registers a new device', async () => {
      const res = await client.register();
      expect(res.uuid).toBe('test-uuid-1234');
      expect(res.token).toBe('new-token');
    });

    it('gets a device by uuid', async () => {
      const device = await client.getDevice('test-uuid-1234');
      expect(device.name).toBe('test-device');
    });

    it('throws on 404', async () => {
      await expect(client.getDevice('not-found')).rejects.toThrow('Not found');
    });

    it('updates a device', async () => {
      const device = await client.updateDevice('test-uuid-1234', { name: 'updated' });
      expect(device.name).toBe('updated');
    });

    it('unregisters a device', async () => {
      await expect(client.unregister('test-uuid-1234')).resolves.toBeUndefined();
    });

    it('fetches whoami', async () => {
      const device = await client.whoami();
      expect(device.uuid).toBe('test-uuid-1234');
    });

    it('fetches myDevices', async () => {
      const devices = await client.myDevices();
      expect(devices).toHaveLength(1);
    });

    it('searches devices', async () => {
      const results = await client.search({ type: 'sensor' });
      expect(results).toHaveLength(1);
    });

    it('claims a device', async () => {
      const device = await client.claimDevice('test-uuid-1234');
      expect(device.uuid).toBe('test-uuid-1234');
    });

    it('sends a message', async () => {
      const res = await client.message({ devices: ['target-1'] });
      expect(res.sent).toBe(true);
    });

    it('broadcasts', async () => {
      await expect(client.broadcast({ topic: 'test' })).resolves.toBeUndefined();
    });

    it('creates a subscription', async () => {
      const sub = await client.createSubscription({
        subscriberUuid: 'test-uuid-1234',
        emitterUuid: 'emitter-1',
        type: 'broadcast.sent',
      });
      expect(sub.type).toBe('broadcast-sent');
    });

    it('lists subscriptions', async () => {
      const subs = await client.subscriptions('test-uuid-1234');
      expect(Array.isArray(subs)).toBe(true);
    });

    it('deletes a subscription', async () => {
      await expect(
        client.deleteSubscription('test-uuid-1234', 'emitter-1', 'broadcast.sent')
      ).resolves.toBeUndefined();
    });

    it('generates a token', async () => {
      const res = await client.generateToken('test-uuid-1234');
      expect(res.token).toBe('gen-token');
    });

    it('revokes a token', async () => {
      await expect(client.revokeToken('test-uuid-1234', 'some-token')).resolves.toBeUndefined();
    });

    it('resets a token', async () => {
      const res = await client.resetToken('test-uuid-1234');
      expect(res.token).toBe('reset-token');
    });
  });
});
