declare module 'mqtt' {
  export function connect(url: string, opts?: Record<string, unknown>): unknown;
}
