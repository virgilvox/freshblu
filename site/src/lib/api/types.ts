export interface WhitelistEntry {
  uuid: string;
}

export interface DiscoverWhitelist {
  view: WhitelistEntry[];
  as: WhitelistEntry[];
}

export interface ConfigureWhitelist {
  update: WhitelistEntry[];
  sent: WhitelistEntry[];
  received: WhitelistEntry[];
  as: WhitelistEntry[];
}

export interface MessageWhitelist {
  from: WhitelistEntry[];
  sent: WhitelistEntry[];
  received: WhitelistEntry[];
  as: WhitelistEntry[];
}

export interface BroadcastWhitelist {
  sent: WhitelistEntry[];
  received: WhitelistEntry[];
  as: WhitelistEntry[];
}

export interface Whitelists {
  discover: DiscoverWhitelist;
  configure: ConfigureWhitelist;
  message: MessageWhitelist;
  broadcast: BroadcastWhitelist;
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

export interface MeshbluMeta {
  version: string;
  createdAt: string;
  updatedAt?: string;
  hash: string;
  whitelists: Whitelists;
  forwarders?: Forwarders;
  publicKey?: string;
  owner?: string;
}

export interface Device {
  uuid: string;
  online: boolean;
  type?: string;
  meshblu: MeshbluMeta;
  [key: string]: unknown;
}

export interface RegisterResponse {
  uuid: string;
  token: string;
  online: boolean;
  meshblu: MeshbluMeta;
  [key: string]: unknown;
}

export interface Message {
  devices: string[];
  fromUuid?: string;
  topic?: string;
  payload?: unknown;
  [key: string]: unknown;
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
  subscriberUuid: string;
  emitterUuid: string;
  type: SubscriptionType;
}

export interface StatusResponse {
  meshblu: boolean;
  sky: string;
}

export interface ApiError {
  error: string;
}
