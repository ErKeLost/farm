import httpProxy from 'http-proxy';
import type { Context, Next } from 'koa';
export function proxyMiddleware(options: any) {
  const proxy = httpProxy.createProxyServer({
    // 配置目标服务器地址
    target: 'http://localhost:3000',
  });
  console.log(options);
  return async (ctx: Context, next: Next) => {
    try {
      // 捕获 http 代理请求
      await new Promise<void>((resolve, reject) => {
        proxy.web(ctx.req, ctx.res, {}, (err) => {
          if (err) {
            reject(err);
          }
          resolve();
        });
      });
    } catch (err) {
      console.error(err);
      ctx.status = 500;
      ctx.body = 'Internal Server Error';
    }
  };
}
