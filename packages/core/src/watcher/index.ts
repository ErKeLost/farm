import { readFileSync } from 'node:fs';
import { basename, relative } from 'node:path';
import { createRequire } from 'node:module';
import debounce from 'lodash.debounce';
import chalk from 'chalk';

import { Compiler } from '../compiler/index.js';
import { DevServer } from '../server/index.js';
import { Config, JsFileWatcher } from '../../binding/index.js';
import { compilerHandler, DefaultLogger } from '../utils/index.js';
import {
  DEFAULT_HMR_OPTIONS,
  normalizeUserCompilationConfig,
  resolveUserConfig
} from '../index.js';
import type { UserConfig } from '../config/index.js';
import { setProcessEnv } from '../config/env.js';

interface ImplFileWatcher {
  watch(): Promise<void>;
}

export class FileWatcher implements ImplFileWatcher {
  private _root: string;
  private _watcher: JsFileWatcher;
  private _logger: DefaultLogger;
  private _awaitWriteFinish: number;

  constructor(
    public serverOrCompiler: DevServer | Compiler,
    public options?: Config & UserConfig
  ) {
    this._root = options.config.root;
    this._awaitWriteFinish = DEFAULT_HMR_OPTIONS.watchOptions.awaitWriteFinish;

    if (serverOrCompiler instanceof DevServer) {
      this._awaitWriteFinish =
        serverOrCompiler.config.hmr.watchOptions.awaitWriteFinish ??
        this._awaitWriteFinish;
    } else if (serverOrCompiler instanceof Compiler) {
      this._awaitWriteFinish =
        serverOrCompiler.config.config?.watch?.watchOptions?.awaitWriteFinish ??
        this._awaitWriteFinish;
    }

    this._logger = new DefaultLogger();
  }

  async watch() {
    // Determine how to compile the project
    const compiler = this.getCompilerFromServerOrCompiler(
      this.serverOrCompiler
    );

    let handlePathChange = async (path: string): Promise<void> => {
      // TODO prepare watch restart server
      const fileName = basename(path);
      const isEnv = fileName === '.env' || fileName.startsWith('.env.');
      const isConfig = path === this.options.resolveConfigPath;
      // TODO configFileDependencies
      if (isEnv || isConfig) {
        this._logger.info(
          `Restarting server due to ${chalk.green(
            relative(process.cwd(), path)
          )} change...`
        );
        if (this.serverOrCompiler instanceof DevServer) {
          await this.serverOrCompiler.close();
        }
        setProcessEnv('development');
        const config: UserConfig = await resolveUserConfig(
          this.options.inlineConfig,
          this._logger
        );
        await readFileSync(this.options.resolveConfigPath, 'utf-8');
        const _require = createRequire(import.meta.url);
        delete _require.cache[_require.resolve(this.options.resolveConfigPath)];
        const data = await import(this.options.resolveConfigPath);
        console.log(data);
        const moduleSpecifier = new URL(
          this.options.resolveConfigPath,
          import.meta.url
        ).toString();
        const module = await import(moduleSpecifier);
        console.log(module.default); // 根据需要访问导出的内容
        const normalizedConfig = await normalizeUserCompilationConfig(config);
        setProcessEnv(normalizedConfig.config.mode);
        const compiler = new Compiler(normalizedConfig);
        const devServer = new DevServer(compiler, this._logger, config);
        devServer.listen();
        return;
      }
      try {
        if (this.serverOrCompiler instanceof DevServer) {
          await this.serverOrCompiler.hmrEngine.hmrUpdate(path);
        }

        if (
          this.serverOrCompiler instanceof Compiler &&
          this.serverOrCompiler.hasModule(path)
        ) {
          compilerHandler(async () => {
            await compiler.update([path], true);
            compiler.writeResourcesToDisk();
          }, this.options);
        }
      } catch (error) {
        this._logger.error(error);
      }
    };

    if (process.platform === 'win32') {
      handlePathChange = debounce(handlePathChange, this._awaitWriteFinish);
    }

    this._watcher = new JsFileWatcher((paths: string[]) => {
      paths.forEach(handlePathChange);
    });

    this._watcher.watch([
      ...compiler.resolvedModulePaths(this._root),
      ...compiler.resolvedWatchPaths()
    ]);

    if (this.serverOrCompiler instanceof DevServer) {
      this.serverOrCompiler.hmrEngine?.onUpdateFinish((updateResult) => {
        const added = [
          ...updateResult.added,
          ...updateResult.extraWatchResult.add
        ].map((addedModule) => {
          const resolvedPath = compiler.transformModulePath(
            this._root,
            addedModule
          );
          return resolvedPath;
        });
        this._watcher.watch(added);
      });
    }
  }

  private getCompilerFromServerOrCompiler(
    serverOrCompiler: DevServer | Compiler
  ): Compiler {
    return serverOrCompiler instanceof DevServer
      ? serverOrCompiler.getCompiler()
      : serverOrCompiler;
  }
}

export async function restartServer(server: DevServer) {
  await server.close();
}

export function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function clearModuleCache(modulePath: string) {
  const _require = createRequire(import.meta.url);
  delete _require.cache[_require.resolve(modulePath)];
}
