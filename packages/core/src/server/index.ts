import { readFileSync } from 'fs';
import http from 'node:http';

import Koa from 'koa';
// import serve from 'koa-static';
import { WebSocketServer } from 'ws';
import chalk from 'chalk';
import boxen from 'boxen';
import figlet from 'figlet';

import { Compiler } from '../compiler/index.js';
import {
  UserServerConfig,
  NormalizedServerConfig,
  normalizeDevServerOptions
} from '../config/index.js';

import { brandColor, Logger } from '../logger.js';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { HmrEngine } from './hmr-engine.js';
import { hmrPlugin } from './middlewares/hmr.js';
import { lazyCompilationPlugin } from './middlewares/lazy-compilation.js';
import { resourcesPlugin } from './middlewares/resources.js';
import { proxyPlugin } from './middlewares/proxy.js';

/**
 * Farm Dev Server, responsible for:
 * * parse and normalize dev server options
 * * launch http server based on options
 * * compile the project in dev mode and serve the production
 * * HMR middleware and websocket supported
 */

interface FarmServerContext {
  config: UserServerConfig;
  app: Koa;
  server: http.Server;
  compiler: Compiler;
  logger: Logger;
}

export class DevServer {
  private _app: Koa;

  _context: FarmServerContext;
  ws: WebSocketServer;
  config: NormalizedServerConfig;
  hmrEngine?: HmrEngine;
  server?: http.Server;

  constructor(
    private _compiler: Compiler,
    public logger: Logger,
    options?: UserServerConfig
  ) {
    this.createFarmServer(options);
  }

  createFarmServer(options: UserServerConfig) {
    this.config = normalizeDevServerOptions(options, 'development');
    this._app = new Koa();
    this.server = http.createServer(this._app.callback());
    this._context = {
      config: this.config,
      app: this._app,
      server: this.server,
      compiler: this._compiler,
      logger: this.logger
    };
    this.resolvedFarmServerPlugins();
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
    this.server.listen(this.config.port);
    this.startDevLogger(start, end);
  }

  private resolvedFarmServerPlugins() {
    const resolvedPlugins = [
      lazyCompilationPlugin,
      hmrPlugin,
      resourcesPlugin,
      proxyPlugin
    ];
    // this._app.use(serve(this._dist));
    resolvedPlugins.forEach((p) => p(this));
  }

  private startDevLogger(start: number, end: number) {
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
            width: 40
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
          borderStyle: 'round'
        }
      ),
      false
    );
  }
}
