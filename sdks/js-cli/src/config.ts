import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

export interface Credentials {
  uuid?: string;
  token?: string;
  server?: string;
}

export function loadConfig(path: string): Credentials {
  try {
    const raw = readFileSync(resolve(path), 'utf-8');
    return JSON.parse(raw);
  } catch {
    return {};
  }
}

export function saveConfig(path: string, creds: Credentials): void {
  writeFileSync(resolve(path), JSON.stringify(creds, null, 2) + '\n');
}
