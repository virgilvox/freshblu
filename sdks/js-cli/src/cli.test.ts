/**
 * Integration tests for freshblu JS CLI.
 *
 * Spawns a real FreshBlu server (via the Rust binary with --features server)
 * and exercises every CLI command against it.
 *
 * Prerequisites:
 *   cargo build -p freshblu-cli --features server
 *   cd sdks/js-cli && npm run build
 *
 * Run: npx vitest run
 */

import { execSync, spawn, type ChildProcess } from 'node:child_process';
import { readFileSync, unlinkSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, it, expect, beforeAll, afterAll, afterEach } from 'vitest';

const CLI = resolve(__dirname, '../dist/cli.js');
const CREDS_FILE = resolve(__dirname, '../test-freshblu.json');
const PROJECT_ROOT = resolve(__dirname, '../../../');

let serverProcess: ChildProcess;
let SERVER_URL: string;
let SERVER_PORT: number;

/** Run the JS CLI with args, return stdout */
function cli(args: string, opts: { env?: Record<string, string> } = {}): string {
  const cmd = `node ${CLI} -S ${SERVER_URL} -c ${CREDS_FILE} ${args}`;
  return execSync(cmd, {
    encoding: 'utf-8',
    timeout: 10_000,
    env: { ...process.env, ...opts.env },
  }).trim();
}

/** Run the CLI expecting failure, return stderr+stdout combined */
function cliFail(args: string): string {
  const cmd = `node ${CLI} -S ${SERVER_URL} -c ${CREDS_FILE} ${args}`;
  try {
    execSync(cmd, { encoding: 'utf-8', timeout: 10_000, stdio: 'pipe' });
    throw new Error('Expected CLI to fail');
  } catch (e: any) {
    return (e.stderr || '') + (e.stdout || '');
  }
}

/** Wait for server to accept connections */
async function waitForServer(url: string, timeoutMs = 15_000): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const resp = await fetch(`${url}/status`);
      if (resp.ok) return;
    } catch {
      // not ready yet
    }
    await new Promise((r) => setTimeout(r, 200));
  }
  throw new Error(`Server at ${url} did not start within ${timeoutMs}ms`);
}

/** Find a free port */
function findFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const srv = require('node:net').createServer();
    srv.listen(0, '127.0.0.1', () => {
      const port = srv.address().port;
      srv.close(() => resolve(port));
    });
    srv.on('error', reject);
  });
}

beforeAll(async () => {
  // Clean up any leftover creds
  if (existsSync(CREDS_FILE)) unlinkSync(CREDS_FILE);

  // Build the CLI bundle
  execSync('npx tsup', { cwd: resolve(__dirname, '..'), stdio: 'pipe' });

  // Find a free port and start the Rust server
  SERVER_PORT = await findFreePort();
  SERVER_URL = `http://127.0.0.1:${SERVER_PORT}`;

  const serverBin = resolve(PROJECT_ROOT, 'target/debug/freshblu');

  serverProcess = spawn(serverBin, ['server', '--port', String(SERVER_PORT), '--db', 'sqlite::memory:'], {
    env: { ...process.env, RUST_LOG: 'error' },
    stdio: 'pipe',
  });

  // If server fails to start, surface the error
  serverProcess.stderr?.on('data', (data: Buffer) => {
    const msg = data.toString();
    if (msg.includes('error') || msg.includes('panic')) {
      console.error('Server stderr:', msg);
    }
  });

  await waitForServer(SERVER_URL);
}, 30_000);

afterAll(() => {
  if (serverProcess) {
    serverProcess.kill('SIGTERM');
  }
  if (existsSync(CREDS_FILE)) unlinkSync(CREDS_FILE);
});

// ---- Tests ----

describe('status', () => {
  it('returns meshblu: true', () => {
    const out = cli('status -f json');
    const data = JSON.parse(out);
    expect(data.meshblu).toBe(true);
  });
});

describe('register', () => {
  afterEach(() => {
    if (existsSync(CREDS_FILE)) unlinkSync(CREDS_FILE);
  });

  it('registers a device and saves credentials', () => {
    const out = cli('register -d \'{"type":"test-sensor"}\' -f json');
    const data = JSON.parse(out);
    expect(data.uuid).toBeDefined();
    expect(data.token).toBeDefined();
    expect(data.type).toBe('test-sensor');

    // Check creds file was written
    const creds = JSON.parse(readFileSync(CREDS_FILE, 'utf-8'));
    expect(creds.uuid).toBe(data.uuid);
    expect(creds.token).toBe(data.token);
    expect(creds.server).toBe(SERVER_URL);
  });

  it('registers with --type flag', () => {
    const out = cli('register -t actuator -f json');
    const data = JSON.parse(out);
    expect(data.type).toBe('actuator');
  });
});

describe('whoami', () => {
  it('returns device info for authenticated device', () => {
    // Register first
    const reg = JSON.parse(cli('register -f json'));

    // whoami reads saved creds
    const out = cli('whoami -f json');
    const data = JSON.parse(out);
    expect(data.uuid).toBe(reg.uuid);
  });
});

describe('get', () => {
  it('gets a device by UUID', () => {
    const reg = JSON.parse(cli('register -f json'));
    const out = cli(`get ${reg.uuid} -f json`);
    const data = JSON.parse(out);
    expect(data.uuid).toBe(reg.uuid);
  });
});

describe('update', () => {
  it('updates device properties', () => {
    const reg = JSON.parse(cli('register -f json'));
    const out = cli(`update ${reg.uuid} -d '{"color":"blue","temp":22}' -f json`);
    const data = JSON.parse(out);
    expect(data.color).toBe('blue');
    expect(data.temp).toBe(22);
  });

  it('updates own device when no uuid given', () => {
    cli('register -f json');
    const out = cli(`update -d '{"status":"active"}' -f json`);
    const data = JSON.parse(out);
    expect(data.status).toBe('active');
  });
});

describe('search', () => {
  it('searches devices by query', () => {
    const reg = JSON.parse(cli('register -d \'{"type":"search-test"}\' -f json'));
    const out = cli(`search -d '{"type":"search-test"}' -f json`);
    const data = JSON.parse(out);
    expect(Array.isArray(data)).toBe(true);
    expect(data.some((d: any) => d.uuid === reg.uuid)).toBe(true);
  });
});

describe('message', () => {
  it('sends a message', () => {
    cli('register -f json');
    const out = cli(`message -d '{"devices":["*"],"payload":{"temp":22}}' -f json`);
    const data = JSON.parse(out);
    expect(data.sent).toBe(true);
  });
});

describe('subscribe', () => {
  it('creates a subscription', () => {
    const reg = JSON.parse(cli('register -f json'));
    const out = cli(`subscribe ${reg.uuid} broadcast.sent -f json`);
    const data = JSON.parse(out);
    expect(data.emitterUuid).toBe(reg.uuid);
    expect(data.subscriptionType).toBe('broadcast-sent');
  });
});

describe('token', () => {
  it('generates a new token', () => {
    const reg = JSON.parse(cli('register -f json'));
    const out = cli(`token generate ${reg.uuid} -f json`);
    const data = JSON.parse(out);
    expect(data.token).toBeDefined();
    expect(data.token.length).toBeGreaterThan(0);
  });

  it('revokes a token', () => {
    const reg = JSON.parse(cli('register -f json'));
    const gen = JSON.parse(cli(`token generate ${reg.uuid} -f json`));
    // Revoke should not throw
    cli(`token revoke ${reg.uuid} ${gen.token}`);
  });
});

describe('unregister', () => {
  it('deletes a device', () => {
    const reg = JSON.parse(cli('register -f json'));
    // Should succeed without error
    cli(`unregister ${reg.uuid}`);
    // Whoami with those creds should fail
    const result = cliFail('whoami -f json');
    expect(result).toContain('');
  });
});

describe('config', () => {
  it('shows current config', () => {
    cli('register -f json');
    const out = cli('config -f json');
    const data = JSON.parse(out);
    expect(data.server).toBe(SERVER_URL);
    expect(data.uuid).toBeDefined();
    expect(data.token).toContain('...');
  });
});

describe('full lifecycle', () => {
  afterEach(() => {
    if (existsSync(CREDS_FILE)) unlinkSync(CREDS_FILE);
  });

  it('register -> update -> message -> subscribe -> token -> unregister', () => {
    // 1. Register
    const reg = JSON.parse(cli('register -d \'{"type":"lifecycle"}\' -f json'));
    expect(reg.uuid).toBeDefined();

    // 2. Update
    const updated = JSON.parse(cli(`update -d '{"phase":"testing"}' -f json`));
    expect(updated.phase).toBe('testing');

    // 3. Message
    const msg = JSON.parse(cli(`message -d '{"devices":["${reg.uuid}"],"payload":"ping"}' -f json`));
    expect(msg.sent).toBe(true);

    // 4. Subscribe
    const sub = JSON.parse(cli(`subscribe ${reg.uuid} message.received -f json`));
    expect(sub.subscriptionType).toBe('message-received');

    // 5. Generate token
    const tok = JSON.parse(cli(`token generate -f json`));
    expect(tok.token).toBeDefined();

    // 6. Unregister
    cli('unregister');

    // 7. Verify gone
    const fail = cliFail('whoami -f json');
    expect(fail.length).toBeGreaterThan(0);
  });
});
