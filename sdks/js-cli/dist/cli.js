#!/usr/bin/env node
"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));

// src/cli.ts
var import_commander = require("commander");

// ../js/dist/index.mjs
function resolveBaseUrl(optionsOrUrl) {
  if (typeof optionsOrUrl === "string") {
    const baseUrl = optionsOrUrl.replace(/\/+$/, "");
    return { baseUrl };
  }
  const {
    hostname = "localhost",
    port = 3e3,
    secure = false,
    uuid,
    token
  } = optionsOrUrl;
  const scheme = secure ? "https" : "http";
  return { baseUrl: `${scheme}://${hostname}:${port}`, uuid, token };
}
var FreshBluHttp = class {
  constructor(optionsOrUrl = {}) {
    const resolved = resolveBaseUrl(optionsOrUrl);
    this.baseUrl = resolved.baseUrl;
    this.uuid = resolved.uuid;
    this.token = resolved.token;
  }
  /** Change the base URL */
  setBaseUrl(url) {
    this.baseUrl = url.replace(/\/+$/, "");
  }
  authHeader() {
    if (!this.uuid || !this.token) return void 0;
    const creds = btoa(`${this.uuid}:${this.token}`);
    return `Basic ${creds}`;
  }
  headers(extra) {
    const h = {
      "Content-Type": "application/json"
    };
    const auth = this.authHeader();
    if (auth) h["Authorization"] = auth;
    return { ...h, ...extra };
  }
  async request(method, path, body, extraHeaders) {
    const resp = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers: this.headers(extraHeaders),
      body: body !== void 0 ? JSON.stringify(body) : void 0
    });
    if (!resp.ok) {
      const err = await resp.json().catch(() => ({ error: resp.statusText }));
      throw new Error(err.error || `HTTP ${resp.status}`);
    }
    if (resp.status === 204) return void 0;
    return resp.json();
  }
  /** Authenticate with current credentials */
  async authenticate() {
    return this.request("POST", "/authenticate", { uuid: this.uuid, token: this.token });
  }
  /** Register a new device */
  async register(properties = {}) {
    return this.request("POST", "/devices", properties);
  }
  /** Get authenticated device info */
  async whoami() {
    return this.request("GET", "/whoami");
  }
  /** Get devices owned by authenticated device */
  async myDevices() {
    return this.request("GET", "/mydevices");
  }
  /** Get a device by UUID */
  async getDevice(uuid, asUuid) {
    const headers = asUuid ? { "x-meshblu-as": asUuid } : void 0;
    return this.request("GET", `/devices/${uuid}`, void 0, headers);
  }
  /** Update a device */
  async updateDevice(uuid, properties) {
    return this.request("PUT", `/devices/${uuid}`, properties);
  }
  /** Unregister a device */
  async unregister(uuid) {
    await this.request("DELETE", `/devices/${uuid}`);
  }
  /** Search for devices */
  async search(query = {}) {
    return this.request("POST", "/devices/search", query);
  }
  /** Claim a device by UUID */
  async claimDevice(uuid) {
    return this.request("POST", `/claimdevice/${uuid}`);
  }
  /** Send a message */
  async message(msg) {
    return this.request("POST", "/messages", msg);
  }
  /** Broadcast a message */
  async broadcast(msg) {
    await this.request("POST", "/broadcasts", msg);
  }
  /** Create a subscription */
  async createSubscription(params) {
    return this.request(
      "POST",
      `/devices/${params.subscriberUuid}/subscriptions`,
      params
    );
  }
  /** Delete a subscription */
  async deleteSubscription(subscriberUuid, emitterUuid, type) {
    await this.request(
      "DELETE",
      `/devices/${subscriberUuid}/subscriptions/${emitterUuid}/${type.replace(".", "-")}`
    );
  }
  /** List subscriptions for a device */
  async subscriptions(subscriberUuid) {
    return this.request("GET", `/devices/${subscriberUuid}/subscriptions`);
  }
  /** Generate a new token for a device */
  async generateToken(uuid, opts = {}) {
    return this.request("POST", `/devices/${uuid}/tokens`, opts);
  }
  /** Revoke a token */
  async revokeToken(uuid, token) {
    await this.request("DELETE", `/devices/${uuid}/tokens/${token}`);
  }
  /** Reset token for a device (revokes all existing, returns new one) */
  async resetToken(uuid) {
    return this.request("POST", `/devices/${uuid}/token`);
  }
  /** Get server status */
  async status() {
    const resp = await fetch(`${this.baseUrl}/status`);
    return resp.json();
  }
  /** Set credentials (after registration) */
  setCredentials(uuid, token) {
    this.uuid = uuid;
    this.token = token;
  }
};

// src/config.ts
var import_node_fs = require("fs");
var import_node_path = require("path");
function loadConfig(path) {
  try {
    const raw = (0, import_node_fs.readFileSync)((0, import_node_path.resolve)(path), "utf-8");
    return JSON.parse(raw);
  } catch {
    return {};
  }
}
function saveConfig(path, creds) {
  (0, import_node_fs.writeFileSync)((0, import_node_path.resolve)(path), JSON.stringify(creds, null, 2) + "\n");
}

// src/output.ts
var import_picocolors = __toESM(require("picocolors"));
function printJson(data, format) {
  if (format === "json") {
    console.log(JSON.stringify(data));
  } else {
    console.log(JSON.stringify(data, null, 2));
  }
}
function success(msg) {
  console.error(import_picocolors.default.green("\u2713") + " " + msg);
}
function error(msg) {
  console.error(import_picocolors.default.red("error") + ": " + msg);
}

// src/cli.ts
var program = new import_commander.Command();
program.name("freshblu").description("FreshBlu CLI - Meshblu-compatible IoT device registry & messaging").version("1.0.0").option("-S, --server <url>", "FreshBlu server URL", "http://localhost:3000").option("-U, --uuid <uuid>", "Device UUID for auth").option("-T, --token <token>", "Device token for auth").option("-c, --config <path>", "Path to config file", "freshblu.json").option("-f, --format <format>", "Output format (json|pretty)", "pretty");
function getClient(opts) {
  const creds = loadConfig(opts.config);
  const uuid = opts.uuid || creds.uuid;
  const token = opts.token || creds.token;
  const server = (opts.server !== "http://localhost:3000" ? opts.server : creds.server) || opts.server;
  const client = new FreshBluHttp(server);
  if (uuid && token) {
    client.setCredentials(uuid, token);
  }
  return client;
}
program.command("register").description("Register a new device").option("-d, --data <json>", "Device properties as JSON", "{}").option("-t, --type <type>", "Device type").action(async (cmdOpts) => {
  const opts = program.opts();
  const client = new FreshBluHttp(opts.server);
  try {
    const props = JSON.parse(cmdOpts.data);
    if (cmdOpts.type) props.type = cmdOpts.type;
    const result = await client.register(props);
    const creds = {
      uuid: result.uuid,
      token: result.token,
      server: opts.server
    };
    saveConfig(opts.config, creds);
    success(`Credentials saved to ${opts.config}`);
    printJson(result, opts.format);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
program.command("whoami").description("Show authenticated device info").action(async () => {
  const opts = program.opts();
  const client = getClient(opts);
  try {
    const result = await client.whoami();
    printJson(result, opts.format);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
program.command("get <uuid>").description("Get a device by UUID").option("--as <uuid>", "Act as another device").action(async (uuid, cmdOpts) => {
  const opts = program.opts();
  const client = getClient(opts);
  try {
    const result = await client.getDevice(uuid, cmdOpts.as);
    printJson(result, opts.format);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
program.command("update [uuid]").description("Update a device").requiredOption("-d, --data <json>", "Properties to update as JSON").action(async (uuid, cmdOpts) => {
  const opts = program.opts();
  const client = getClient(opts);
  const creds = loadConfig(opts.config);
  const target = uuid || opts.uuid || creds.uuid;
  if (!target) {
    error("UUID required");
    process.exit(1);
  }
  try {
    const props = JSON.parse(cmdOpts.data);
    const result = await client.updateDevice(target, props);
    printJson(result, opts.format);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
program.command("unregister [uuid]").description("Delete a device").action(async (uuid) => {
  const opts = program.opts();
  const client = getClient(opts);
  const creds = loadConfig(opts.config);
  const target = uuid || opts.uuid || creds.uuid;
  if (!target) {
    error("UUID required");
    process.exit(1);
  }
  try {
    await client.unregister(target);
    success(`Device ${target} unregistered`);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
program.command("search").description("Search for devices").option("-d, --data <json>", "Query as JSON", "{}").action(async (cmdOpts) => {
  const opts = program.opts();
  const client = getClient(opts);
  try {
    const query = JSON.parse(cmdOpts.data);
    const result = await client.search(query);
    printJson(result, opts.format);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
program.command("message").description("Send a message (data must include devices field)").requiredOption("-d, --data <json>", "Message as JSON").action(async (cmdOpts) => {
  const opts = program.opts();
  const client = getClient(opts);
  try {
    const msg = JSON.parse(cmdOpts.data);
    const result = await client.message(msg);
    printJson(result, opts.format);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
program.command("subscribe <emitter> <type>").description("Create a subscription").action(async (emitter, type) => {
  const opts = program.opts();
  const client = getClient(opts);
  const creds = loadConfig(opts.config);
  const subscriber = opts.uuid || creds.uuid;
  if (!subscriber) {
    error("No UUID set - register first or pass -U");
    process.exit(1);
  }
  try {
    const result = await client.createSubscription({
      emitterUuid: emitter,
      subscriberUuid: subscriber,
      type: type.replace(".", "-")
    });
    printJson(result, opts.format);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
var tokenCmd = program.command("token").description("Token management");
tokenCmd.command("generate [uuid]").description("Generate a new token for a device").option("--expires-on <epoch>", "Expiration timestamp").option("--tag <tag>", "Token tag").action(async (uuid, cmdOpts) => {
  const opts = program.opts();
  const client = getClient(opts);
  const creds = loadConfig(opts.config);
  const target = uuid || opts.uuid || creds.uuid;
  if (!target) {
    error("UUID required");
    process.exit(1);
  }
  try {
    const tokenOpts = {};
    if (cmdOpts.expiresOn) tokenOpts.expiresOn = Number(cmdOpts.expiresOn);
    if (cmdOpts.tag) tokenOpts.tag = cmdOpts.tag;
    const result = await client.generateToken(target, tokenOpts);
    printJson(result, opts.format);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
tokenCmd.command("revoke [uuid] <token>").description("Revoke a token").action(async (uuid, token) => {
  const opts = program.opts();
  const client = getClient(opts);
  const creds = loadConfig(opts.config);
  let target;
  let revokeToken;
  if (token === void 0) {
    revokeToken = uuid;
    target = opts.uuid || creds.uuid;
    if (!target) {
      error("UUID required");
      process.exit(1);
    }
  } else {
    target = uuid;
    revokeToken = token;
  }
  try {
    await client.revokeToken(target, revokeToken);
    success(`Token revoked for ${target}`);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
program.command("status").description("Check server health").action(async () => {
  const opts = program.opts();
  const client = getClient(opts);
  try {
    const result = await client.status();
    printJson(result, opts.format);
  } catch (e) {
    error(e.message);
    process.exit(1);
  }
});
program.command("config").description("Show current config").action(() => {
  const opts = program.opts();
  const creds = loadConfig(opts.config);
  const display = {
    server: creds.server || opts.server,
    uuid: creds.uuid || opts.uuid,
    token: creds.token ? creds.token.slice(0, 8) + "..." : void 0
  };
  printJson(display, opts.format);
});
program.parseAsync();
