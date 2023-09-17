import type { UserConfig } from '@farmfe/core';
import plugins from './plugins.ts'
function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    resolve: {
      symlinks: true
    },
    define: {
      BTN: 'Click me'
    },
    output: {
      path: './build'
    },
    sourcemap: true,
    css: {
      // modules: {
      //   indentName: 'farm-[name]-[hash]'
      // },
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    },
    // treeShaking: true,
    minify: false
  },
  server: {
    hmr: true,
    // cors: true,
    port: 8888,
    host: 'localhost'
  },
  plugins
});
