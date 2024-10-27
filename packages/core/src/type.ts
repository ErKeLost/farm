import type { OutgoingHttpHeaders as HttpServerHeaders } from 'node:http';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import { WatchOptions } from 'chokidar';
import { CorsOrigin } from './server/http.js';
import { HttpServer } from './server/index.js';
import { ProxyOptions } from './server/middlewares/proxy.js';

export interface CorsOptions {
  origin?:
    | CorsOrigin
    | ((
        origin: string | undefined,
        cb: (err: Error, origins: CorsOrigin) => void
      ) => void);
  methods?: string | string[];
  allowedHeaders?: string | string[];
  exposedHeaders?: string | string[];
  credentials?: boolean;
  maxAge?: number;
  preflightContinue?: boolean;
  optionsSuccessStatus?: number;
}

export interface CommonServerOptions {
  port?: number;
  strictPort?: boolean;
  host?: string | boolean;
  https?: HttpsServerOptions;
  open?: boolean | string;
  proxy?: Record<string, string | ProxyOptions>;
  cors?: CorsOptions | boolean;
  headers?: HttpServerHeaders;
}

export interface HmrOptions {
  protocol?: string;
  host?: string;
  port?: number;
  clientPort?: number;
  path?: string;
  overlay?: boolean;
  server?: HttpServer;
}

export interface ServerOptions extends CommonServerOptions {
  appType?: 'spa' | 'mpa' | 'custom';
  hmr?: boolean | HmrOptions;
  ws?: false;
  watch?: WatchOptions | null;
  middlewareMode?:
    | boolean
    | {
        /**
         * Parent server instance to attach to
         *
         * This is needed to proxy WebSocket connections to the parent server.
         */
        server: HttpServer;
      };
  /**
   * Origin for the generated asset URLs.
   *
   * @example `http://127.0.0.1:8080`
   */
  origin?: string;
}
