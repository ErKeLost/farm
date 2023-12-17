import { defineConfig } from '@farmfe/core';
export default defineConfig({
  compilation: {
    output: {
      targetEnv: 'node'
    },
    presetEnv: false
    // external: ['@farmfe/core']
  }
});
