import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { uuid, token, authenticated, setCredentials, clearCredentials } from './auth';

describe('auth store', () => {
  beforeEach(() => {
    clearCredentials();
  });

  it('starts empty', () => {
    expect(get(uuid)).toBe('');
    expect(get(token)).toBe('');
    expect(get(authenticated)).toBe(false);
  });

  it('sets credentials', () => {
    setCredentials('device-1', 'token-abc');
    expect(get(uuid)).toBe('device-1');
    expect(get(token)).toBe('token-abc');
    expect(get(authenticated)).toBe(true);
  });

  it('clears credentials', () => {
    setCredentials('device-1', 'token-abc');
    clearCredentials();
    expect(get(uuid)).toBe('');
    expect(get(token)).toBe('');
    expect(get(authenticated)).toBe(false);
  });
});
