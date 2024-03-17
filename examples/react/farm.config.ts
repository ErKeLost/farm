import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    persistentCache: false,
    presetEnv: false,
    minify: false,
    sourcemap: false
  },
  plugins: [
    ['@farmfe/plugin-react', { runtime: 'automatic' }],
    '@farmfe/plugin-sass',
  ],
});
