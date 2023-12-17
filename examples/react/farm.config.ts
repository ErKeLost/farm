import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    presetEnv: false,
    output: {
      publicPath: '/react/'
    }
  },
  plugins: [
    ['@farmfe/plugin-react', { runtime: 'automatic' }],
    '@farmfe/plugin-sass'
  ]
});
