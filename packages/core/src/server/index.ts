import { readFileSync } from 'fs';

import Koa from 'koa';
// import serve from 'koa-static';
import { WebSocketServer } from 'ws';
import net from 'node:net';
import chalk from 'chalk';
import boxen from 'boxen';
import figlet from 'figlet';

import { Compiler } from '../compiler/index.js';
import {
  UserServerConfig,
  NormalizedServerConfig,
  normalizeDevServerOptions,
} from '../config/index.js';
import { hmr } from './middlewares/hmr.js';
import { HmrEngine } from './hmr-engine.js';
import { brandColor, Logger } from '../logger.js';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { lazyCompilation } from './middlewares/lazy-compilation.js';
import { resources } from './middlewares/resources.js';

/**
 * Farm Dev Server, responsible for:
 * * parse and normalize dev server options
 * * launch http server based on options
 * * compile the project in dev mode and serve the production
 * * HMR middleware and websocket supported
 */
export class DevServer {
  private _app: Koa;

  ws: WebSocketServer;
  config: NormalizedServerConfig;
  hmrEngine?: HmrEngine;

  constructor(
    private _compiler: Compiler,
    public logger: Logger,
    options?: UserServerConfig
  ) {
    this.config = normalizeDevServerOptions(options);
    this._app = new Koa();

    // this._app.use(serve(this._dist));
    this._app.use(resources(this._compiler));
    if (this.config.hmr) {
      isPortTaken(this.config.hmr.port)
        .then((result) => {
          let socketServer = {};
          if (result) {
            socketServer = {
              port: ++this.config.hmr.port,
              host: this.config.hmr.host,
            };
          } else {
            socketServer = {
              port: this.config.hmr.port,
              host: this.config.hmr.host,
            };
          }
          console.log(socketServer);

          this.ws = new WebSocketServer(socketServer);
          this._app.use(hmr(this));
          this.hmrEngine = new HmrEngine(this._compiler, this, this.logger);
          console.log(this.config.hmr.port);
        })
        .catch((err) => {
          console.error(err);
        });
    }

    if (this._compiler.config.config.lazyCompilation) {
      this._app.use(lazyCompilation(this));
    }
  }

  getCompiler(): Compiler {
    return this._compiler;
  }

  async listen(): Promise<void> {
    const start = Date.now();
    // compile the project and start the dev server
    if (process.env.FARM_PROFILE) {
      this._compiler.compileSync();
    } else {
      await this._compiler.compile();
    }

    const end = Date.now();
    await httpServerStart(this);
    const version = JSON.parse(
      readFileSync(
        join(fileURLToPath(import.meta.url), '../../../package.json'),
        'utf-8'
      )
    ).version;
    this.logger.info(
      boxen(
        `${brandColor(
          figlet.textSync('FARM', {
            width: 40,
          })
        )}
Version ${chalk.green.bold(version)}

🔥 Ready on ${chalk.green.bold(
          `http://localhost:${this.config.port}`
        )} in ${chalk.green.bold(`${end - start}ms`)}.
    `,
        {
          padding: 1,
          margin: 1,
          align: 'center',
          borderColor: 'cyan',
          borderStyle: 'round',
        }
      ),
      false
    );
  }
}

export async function httpServerStart(self: any) {
  return new Promise((resolve) => {
    const onError = (e: Error & { code?: string }) => {
      // TODO strickPort side effect
      if (e.code === 'EADDRINUSE') {
        self.logger.info(
          `Port ${self.config.port} is in use, trying another one...`
        );
        self._app.listen(++self.config.port, () => {
          resolve(self.config.port);
        });
      }
    };
    self._app
      .listen(self.config.port, () => {
        resolve(self.config.port);
      })
      .on('error', onError);
  });
}

export async function isPortTaken(port: number) {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    console.log(port);

    // server.once('listening', () => {
    //   server.close();
    //   console.log(`Port ${port} is available`);
    //   resolve(false);
    // });
    server.once('error', (err: Error & { code: string }) => {
      if (err.code === 'EADDRINUSE') {
        console.log('端口被占用');
        resolve(true);
      } else {
        reject(err);
      }
    });
    server.listen(port);
  });
}
