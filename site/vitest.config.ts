import { defineConfig } from 'vitest/config';
import { resolve } from 'path';

export default defineConfig({
  resolve: {
    alias: {
      '$lib': resolve('./src/lib'),
      '$app/environment': resolve('./src/test-mocks/environment.ts'),
      '$env/static/public': resolve('./src/test-mocks/env.ts'),
    },
  },
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'jsdom',
    setupFiles: ['src/test-setup.ts'],
  },
});
