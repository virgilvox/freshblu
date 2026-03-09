import { defineConfig } from 'tsup';

export default defineConfig([
  {
    entry: ['src/index.ts'],
    format: ['cjs', 'esm'],
    dts: true,
    external: ['mqtt'],
    clean: true,
  },
  {
    entry: { index: 'src/browser.ts' },
    format: ['iife'],
    globalName: 'FreshBluSDK',
    external: ['mqtt'],
    clean: false,
    footer: {
      js: `if(typeof window!=="undefined"){window.FreshBlu=FreshBluSDK.FreshBlu;window.FreshBluHttp=FreshBluSDK.FreshBluHttp;}`,
    },
  },
]);
