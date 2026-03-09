import { Command } from 'commander';
import { FreshBluHttp } from 'freshblu';
import pc from 'picocolors';
import { loadConfig, saveConfig, type Credentials } from './config';
import { printJson, success, error } from './output';

const program = new Command();

program
  .name('freshblu')
  .description('FreshBlu CLI - Meshblu-compatible IoT device registry & messaging')
  .version('1.0.0')
  .option('-S, --server <url>', 'FreshBlu server URL', 'http://localhost:3000')
  .option('-U, --uuid <uuid>', 'Device UUID for auth')
  .option('-T, --token <token>', 'Device token for auth')
  .option('-c, --config <path>', 'Path to config file', 'freshblu.json')
  .option('-f, --format <format>', 'Output format (json|pretty)', 'pretty');

function getClient(opts: ReturnType<typeof program.opts>): FreshBluHttp {
  const creds: Credentials = loadConfig(opts.config);

  const uuid = opts.uuid || creds.uuid;
  const token = opts.token || creds.token;
  const server = (opts.server !== 'http://localhost:3000' ? opts.server : creds.server) || opts.server;

  const client = new FreshBluHttp(server);
  if (uuid && token) {
    client.setCredentials(uuid, token);
  }
  return client;
}

program
  .command('register')
  .description('Register a new device')
  .option('-d, --data <json>', 'Device properties as JSON', '{}')
  .option('-t, --type <type>', 'Device type')
  .action(async (cmdOpts) => {
    const opts = program.opts();
    const client = new FreshBluHttp(opts.server);
    try {
      const props = JSON.parse(cmdOpts.data);
      if (cmdOpts.type) props.type = cmdOpts.type;

      const result = await client.register(props);

      const creds: Credentials = {
        uuid: result.uuid,
        token: result.token,
        server: opts.server,
      };
      saveConfig(opts.config, creds);
      success(`Credentials saved to ${opts.config}`);
      printJson(result, opts.format);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

program
  .command('whoami')
  .description('Show authenticated device info')
  .action(async () => {
    const opts = program.opts();
    const client = getClient(opts);
    try {
      const result = await client.whoami();
      printJson(result, opts.format);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

program
  .command('get <uuid>')
  .description('Get a device by UUID')
  .option('--as <uuid>', 'Act as another device')
  .action(async (uuid: string, cmdOpts) => {
    const opts = program.opts();
    const client = getClient(opts);
    try {
      const result = await client.getDevice(uuid, cmdOpts.as);
      printJson(result, opts.format);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

program
  .command('update [uuid]')
  .description('Update a device')
  .requiredOption('-d, --data <json>', 'Properties to update as JSON')
  .action(async (uuid: string | undefined, cmdOpts) => {
    const opts = program.opts();
    const client = getClient(opts);
    const creds = loadConfig(opts.config);
    const target = uuid || opts.uuid || creds.uuid;
    if (!target) { error('UUID required'); process.exit(1); }
    try {
      const props = JSON.parse(cmdOpts.data);
      const result = await client.updateDevice(target, props);
      printJson(result, opts.format);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

program
  .command('unregister [uuid]')
  .description('Delete a device')
  .action(async (uuid: string | undefined) => {
    const opts = program.opts();
    const client = getClient(opts);
    const creds = loadConfig(opts.config);
    const target = uuid || opts.uuid || creds.uuid;
    if (!target) { error('UUID required'); process.exit(1); }
    try {
      await client.unregister(target);
      success(`Device ${target} unregistered`);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

program
  .command('search')
  .description('Search for devices')
  .option('-d, --data <json>', 'Query as JSON', '{}')
  .action(async (cmdOpts) => {
    const opts = program.opts();
    const client = getClient(opts);
    try {
      const query = JSON.parse(cmdOpts.data);
      const result = await client.search(query);
      printJson(result, opts.format);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

program
  .command('message')
  .description('Send a message (data must include devices field)')
  .requiredOption('-d, --data <json>', 'Message as JSON')
  .action(async (cmdOpts) => {
    const opts = program.opts();
    const client = getClient(opts);
    try {
      const msg = JSON.parse(cmdOpts.data);
      const result = await client.message(msg);
      printJson(result, opts.format);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

program
  .command('subscribe <emitter> <type>')
  .description('Create a subscription')
  .action(async (emitter: string, type: string) => {
    const opts = program.opts();
    const client = getClient(opts);
    const creds = loadConfig(opts.config);
    const subscriber = opts.uuid || creds.uuid;
    if (!subscriber) { error('No UUID set - register first or pass -U'); process.exit(1); }
    try {
      const result = await client.createSubscription({
        emitterUuid: emitter,
        subscriberUuid: subscriber,
        type: type.replace('.', '-') as any,
      });
      printJson(result, opts.format);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

const tokenCmd = program.command('token').description('Token management');

tokenCmd
  .command('generate [uuid]')
  .description('Generate a new token for a device')
  .option('--expires-on <epoch>', 'Expiration timestamp')
  .option('--tag <tag>', 'Token tag')
  .action(async (uuid: string | undefined, cmdOpts) => {
    const opts = program.opts();
    const client = getClient(opts);
    const creds = loadConfig(opts.config);
    const target = uuid || opts.uuid || creds.uuid;
    if (!target) { error('UUID required'); process.exit(1); }
    try {
      const tokenOpts: Record<string, unknown> = {};
      if (cmdOpts.expiresOn) tokenOpts.expiresOn = Number(cmdOpts.expiresOn);
      if (cmdOpts.tag) tokenOpts.tag = cmdOpts.tag;
      const result = await client.generateToken(target, tokenOpts);
      printJson(result, opts.format);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

tokenCmd
  .command('revoke [uuid] <token>')
  .description('Revoke a token')
  .action(async (uuid: string, token: string | undefined) => {
    const opts = program.opts();
    const client = getClient(opts);
    const creds = loadConfig(opts.config);
    // commander parses positional args left-to-right; if only one arg, it's the token
    let target: string;
    let revokeToken: string;
    if (token === undefined) {
      revokeToken = uuid;
      target = opts.uuid || creds.uuid;
      if (!target) { error('UUID required'); process.exit(1); }
    } else {
      target = uuid;
      revokeToken = token;
    }
    try {
      await client.revokeToken(target, revokeToken);
      success(`Token revoked for ${target}`);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

program
  .command('status')
  .description('Check server health')
  .action(async () => {
    const opts = program.opts();
    const client = getClient(opts);
    try {
      const result = await client.status();
      printJson(result, opts.format);
    } catch (e: any) {
      error(e.message);
      process.exit(1);
    }
  });

program
  .command('config')
  .description('Show current config')
  .action(() => {
    const opts = program.opts();
    const creds = loadConfig(opts.config);
    const display = {
      server: creds.server || opts.server,
      uuid: creds.uuid || opts.uuid,
      token: creds.token ? creds.token.slice(0, 8) + '...' : undefined,
    };
    printJson(display, opts.format);
  });

program.parseAsync();
