import { defineConfig } from '@farmfe/core';
import path from 'path';

export default defineConfig(() => {
  // console.log(__dirname);
  // console.log(__filename);
  // console.log(__dirname);

  return {
    root: path.resolve(process.cwd(), './html'),
    compilation: {
      sourcemap: false,
      persistentCache: true,
      presetEnv: false,
      progress: false,
      output: {
        publicPath: '/dist/'
      }
    },
    server: {
      port: 6532,
      hmr: {
        path: '/__farm_hmr'
      }
    },
    plugins: [
      ['@farmfe/plugin-react', { runtime: 'automatic' }],
      '@farmfe/plugin-sass'
    ]
  };
});
