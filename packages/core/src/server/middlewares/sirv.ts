import type { Context } from 'koa';
import sirv from 'sirv';
import { DevServer } from '../index.js';
import path from 'node:path';

export function StaticFilesHandler(ctx: Context, distDir: string) {
  const staticFilesServer = sirv(distDir, {
    etag: true,
    single: true
  });
  return new Promise<void>((resolve) => {
    staticFilesServer(ctx.req, ctx.res, () => {
      resolve();
    });
  });
}

export function useSirv(devSeverContext: DevServer) {
  // const { config } = devSeverContext._context;
  // console.log('我是 config', devSeverContext._context.compilationConfig);
  const { output, root } = devSeverContext._context.compilationConfig.config;
  return async (ctx: Context) => {
    const requestPath = ctx.request.path;
    console.log(requestPath);

    const distDir = path.resolve(root, output.path);

    if (requestPath.startsWith(output.publicPath)) {
      const modifiedPath = requestPath.substring(output.publicPath.length);

      if (modifiedPath.startsWith('/')) {
        ctx.request.path = modifiedPath;
      } else {
        ctx.request.path = `/${modifiedPath}`;
      }
    }
    await StaticFilesHandler(ctx, distDir);
  };
}

export function sirvPlugin(distance: DevServer) {
  distance._context.app.use(useSirv(distance));
}
