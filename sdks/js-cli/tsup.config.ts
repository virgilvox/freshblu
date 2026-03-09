import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/cli.ts'],
  format: ['cjs'],
  clean: true,
  banner: { js: '#!/usr/bin/env node' },
  noExternal: ['freshblu'],
  external: ['mqtt'],
});
