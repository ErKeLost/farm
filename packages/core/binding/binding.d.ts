/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface JsPluginAugmentResourceHashHookFilters {
  resourcePotTypes: Array<string>
  moduleIds: Array<string>
}
export interface JsPluginLoadHookFilters {
  resolvedPaths: Array<string>
}
export interface JsPluginRenderResourcePotHookFilters {
  resourcePotTypes: Array<string>
  moduleIds: Array<string>
}
/** Resolve hook filters, works as `||`. If any importers or sources matches any regex item in the Vec, we treat it as filtered. */
export interface JsPluginResolveHookFilters {
  importers: Array<string>
  sources: Array<string>
}
export interface JsPluginTransformHookFilters {
  resolvedPaths: Array<string>
  moduleTypes: Array<string>
}
export const enum JsPluginTransformHtmlHookOrder {
  Pre = 0,
  Normal = 1,
  Post = 2
}
export interface WatchDiffResult {
  add: Array<string>
  remove: Array<string>
}
export interface JsTracedModule {
  id: string
  contentHash: string
  packageName: string
  packageVersion: string
}
export interface JsTracedModuleGraph {
  root: string
  modules: Array<JsTracedModule>
  edges: Record<string, Array<string>>
  reverseEdges: Record<string, Array<string>>
}
export interface JsUpdateResult {
  added: Array<string>
  changed: Array<string>
  removed: Array<string>
  immutableModules: string
  mutableModules: string
  boundaries: Record<string, Array<Array<string>>>
  dynamicResourcesMap?: Record<string, Array<Array<string>>>
  extraWatchResult: WatchDiffResult
}
export interface ResolveRecord {
  plugin: string
  hook: string
  source: string
  importer?: string
  kind: string
  isHmr: boolean
  startTime: number
  endTime: number
  duration: number
}
export interface TransformRecord {
  plugin: string
  hook: string
  content: string
  sourceMaps?: string
  moduleType: string
  isHmr: boolean
  startTime: number
  endTime: number
  duration: number
}
export interface ModuleRecord {
  plugin: string
  hook: string
  moduleType: string
  isHmr: boolean
  startTime: number
  endTime: number
  duration: number
}
export interface AnalyzeDep {
  source: string
  kind: string
}
export interface AnalyzeDepsRecord {
  plugin: string
  hook: string
  moduleType: string
  isHmr: boolean
  deps: Array<AnalyzeDep>
  startTime: number
  endTime: number
  duration: number
}
export interface Module {
  id: string
  moduleType: string
  moduleGroups: Array<string>
  resourcePot?: string
  sideEffects: boolean
  sourceMapChain: Array<string>
  external: boolean
  immutable: boolean
  size: number
}
export interface ResourcePotRecord {
  name: string
  hook: string
  modules: Array<string>
  resources: Array<string>
}
export type JsCompiler = Compiler
export declare class Compiler {
  constructor(config: object)
  traceDependencies(): object
  traceModuleGraph(): object
  /** async compile, return promise */
  compile(): object
  /** sync compile */
  compileSync(): void
  /** TODO: usage example */
  update(paths: Array<string>, callback: (...args: any[]) => any, sync: boolean, generateUpdateResource: boolean): object
  addWatchFiles(root: string, paths: Array<string>): void
  hasModule(resolvedPath: string): boolean
  getParentFiles(resolvedPath: string): Array<string>
  resources(): Record<string, Buffer>
  resourcesMap(): Record<string, unknown>
  watchModules(): Array<string>
  relativeModulePaths(): Array<string>
  resource(name: string): Buffer | null
  modules(): Array<Module>
  getResolveRecordsById(id: string): Array<ResolveRecord>
  getTransformRecordsById(id: string): Array<TransformRecord>
  getProcessRecordsById(id: string): Array<ModuleRecord>
  getAnalyzeDepsRecordsById(id: string): Array<AnalyzeDepsRecord>
  getResourcePotRecordsById(id: string): Array<ResourcePotRecord>
  pluginStats(): Record<string, unknown>
}
