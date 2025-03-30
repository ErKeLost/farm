#![feature(box_patterns)]
#![feature(path_file_prefix)]

use std::{path::Path, sync::Arc};

use deps_analyzer::DepsAnalyzer;
use farmfe_core::{
  config::{Config, ModuleFormat, TargetEnv},
  context::CompilationContext,
  enhanced_magic_string::collapse_sourcemap::{collapse_sourcemap_chain, CollapseSourcemapOptions},
  error::{CompilationError, Result},
  module::{
    meta_data::script::{CommentsMetaData, ScriptModuleMetaData},
    module_graph::ModuleGraph,
    ModuleMetaData, ModuleSystem, ModuleType, VIRTUAL_MODULE_PREFIX,
  },
  plugin::{
    GeneratedResource, Plugin, PluginAnalyzeDepsHookParam, PluginFinalizeModuleHookParam,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginProcessModuleHookParam,
  },
  resource::{
    meta_data::{js::JsResourcePotMetaData, ResourcePotMetaData},
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  swc_common::{comments::SingleThreadedComments, Globals, Mark, SourceMap, GLOBALS},
  swc_ecma_ast::{EsVersion, Module as SwcModule},
};
use farmfe_swc_transformer_import_glob::transform_import_meta_glob;
use farmfe_toolkit::{
  fs::read_file_utf8,
  script::{
    codegen_module, concatenate_modules::concatenate_modules_ast, module_type_from_id,
    parse_module, set_module_system_for_module_meta, swc_try_with::try_with,
    syntax_from_module_type, CodeGenCommentsConfig, ParseScriptModuleResult,
  },
  sourcemap::{
    build_sourcemap, load_source_original_sourcemap, trace_module_sourcemap,
    SourceMap as JsonSourceMap,
  },
  swc_ecma_transforms::resolver,
  swc_ecma_visit::VisitMutWith,
};

use import_meta_visitor::{replace_import_meta_url, ImportMetaVisitor};
#[cfg(feature = "swc_plugin")]
use swc_plugins::{init_plugin_module_cache_once, transform_by_swc_plugins};

mod deps_analyzer;
mod import_meta_visitor;
#[cfg(feature = "swc_plugin")]
mod swc_plugins;
mod swc_script_transforms;
mod transform_import_meta_url;

use transform_import_meta_url::transform_url_with_import_meta_url;

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx/... files, support loading, parse, analyze dependencies and code generation.
/// Note that we do not do transforms here, the transforms (e.g. strip types, jsx...) are handled in a separate plugin (farmfe_plugin_swc_transforms).
pub struct FarmPluginScript {}

impl Plugin for FarmPluginScript {
  fn name(&self) -> &str {
    "FarmPluginScript"
  }

  fn priority(&self) -> i32 {
    99
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if module_type.is_script() {
        let content = read_file_utf8(param.resolved_path)?;

        let map =
          load_source_original_sourcemap(&content, param.resolved_path, "//# sourceMappingURL");

        Ok(Some(PluginLoadHookResult {
          content,
          module_type,
          source_map: map,
        }))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  fn parse(
    &self,
    param: &PluginParseHookParam,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<ModuleMetaData>> {
    if let Some(syntax) =
      syntax_from_module_type(&param.module_type, context.config.script.parser.clone())
    {
      let ParseScriptModuleResult {
        ast: mut swc_module,
        comments,
        source_map,
      } = parse_module(
        &param.module_id,
        param.content.clone(),
        syntax,
        EsVersion::EsNext,
      )?;

      context
        .meta
        .set_module_source_map(&param.module_id, source_map);
      let globals = Globals::new();

      let meta = GLOBALS.set(&globals, || {
        let top_level_mark = Mark::new();
        let unresolved_mark = Mark::new();

        swc_module.visit_mut_with(&mut resolver(
          unresolved_mark,
          top_level_mark,
          param.module_type.is_typescript(),
        ));

        let meta = ScriptModuleMetaData {
          ast: swc_module,
          top_level_mark: top_level_mark.as_u32(),
          unresolved_mark: unresolved_mark.as_u32(),
          // set module_system to unknown, it will be detected in `finalize_module`
          module_system: ModuleSystem::Custom(String::from("unknown")),
          hmr_self_accepted: false,
          hmr_accepted_deps: Default::default(),
          comments: CommentsMetaData::from(comments),
          custom: Default::default(),
          statements: vec![],
          top_level_idents: Default::default(),
          unresolved_idents: Default::default(),
          feature_flags: Default::default(),
          export_ident_map: Default::default(),
          is_async: false,
        };

        ModuleMetaData::Script(Box::new(meta))
      });

      context.meta.set_globals(&param.module_id, globals);

      Ok(Some(meta))
    } else {
      Ok(None)
    }
  }

  fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if !param.module_type.is_script() {
      return Ok(None);
    }

    let cm = context.meta.get_module_source_map(&param.module_id);
    let globals = context.meta.get_globals(&param.module_id);

    // transform decorators if needed
    // this transform should be done before strip typescript cause it may need to access the type information
    if (param.module_type.is_typescript() && context.config.script.parser.ts_config.decorators)
      || (param.module_type.is_script() && context.config.script.parser.es_config.decorators)
    {
      swc_script_transforms::transform_decorators(param, &cm, globals.value(), context)?;
    }

    // strip typescript
    if param.module_type.is_typescript() {
      swc_script_transforms::strip_typescript(param, &cm, globals.value(), context)?;
    }

    // execute swc plugins
    #[cfg(feature = "swc_plugin")]
    if param.module_type.is_script() && !context.config.script.plugins.is_empty() {
      try_with(cm.clone(), globals.value(), || {
        transform_by_swc_plugins(param, context).unwrap()
      })?;
    }

    if param.module_type.is_script() {
      // transform vite-style `import.meta.glob`
      let script = param.meta.as_script_mut();
      let comments: SingleThreadedComments = script.take_comments().into();
      let ast = &mut script.ast;
      let resolved_path = param.module_id.resolved_path(&context.config.root);
      let cur_dir = if resolved_path.starts_with(VIRTUAL_MODULE_PREFIX) {
        context.config.root.clone()
      } else {
        Path::new(&resolved_path)
          .parent()
          .unwrap()
          .to_string_lossy()
          .to_string()
      };

      transform_url_with_import_meta_url(ast, &comments);

      transform_import_meta_glob(
        ast,
        context.config.root.clone(),
        cur_dir,
        &context.config.resolve.alias,
      )?;
      script.set_comments(comments.into())
    }

    Ok(Some(()))
  }

  fn analyze_deps(
    &self,
    param: &mut PluginAnalyzeDepsHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    let module = param.module;

    if module.module_type.is_script() {
      let module_ast = &module.meta.as_script().ast;

      let mut analyzer = DepsAnalyzer::new(
        &module.id,
        module_ast,
        Mark::from_u32(module.meta.as_script().unresolved_mark),
        Mark::from_u32(module.meta.as_script().top_level_mark),
      );

      GLOBALS.set(context.meta.get_globals(&param.module.id).value(), || {
        let deps = analyzer.analyze_deps();
        param.deps.extend(deps);
      });

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  /// detect [ModuleSystem] for a script module based on its dependencies' [ResolveKind] and detect hmr_accepted
  fn finalize_module(
    &self,
    param: &mut PluginFinalizeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if !param.module.module_type.is_script() {
      return Ok(None);
    }
    // all jsx, js, ts, tsx modules should be transformed to js module for now
    param.module.module_type = ModuleType::Js;
    // set param.module.meta.module_system
    set_module_system_for_module_meta(param, context);

    // TODO: optimize code here
    let target_env = context.config.output.target_env.clone();
    let format = context.config.output.format;

    let is_library = target_env.is_library();

    if is_library && matches!(format, ModuleFormat::CommonJs) {
      replace_import_meta_url(&mut param.module.meta.as_script_mut().ast)
    };

    // find and replace `import.meta.xxx` to `module.meta.xxx` and detect hmr_accepted
    // skip transform import.meta when targetEnv is node
    if !is_library && (target_env.is_browser() || matches!(format, ModuleFormat::CommonJs)) {
      // transform `import.meta.xxx` to `module.meta.xxx`
      let ast = &mut param.module.meta.as_script_mut().ast;
      let mut import_meta_v = ImportMetaVisitor::new();
      ast.visit_mut_with(&mut import_meta_v);
    }

    if matches!(target_env, TargetEnv::Browser) {
      let ast = &mut param.module.meta.as_script_mut().ast;
      let mut hmr_accepted_v =
        import_meta_visitor::HmrAcceptedVisitor::new(param.module.id.clone(), context.clone());
      ast.visit_mut_with(&mut hmr_accepted_v);
      param.module.meta.as_script_mut().hmr_self_accepted = hmr_accepted_v.is_hmr_self_accepted;
      param.module.meta.as_script_mut().hmr_accepted_deps = hmr_accepted_v.hmr_accepted_deps;
    }

    Ok(Some(()))
  }

  fn render_resource_pot(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> Result<Option<ResourcePotMetaData>> {
    // render dynamic entry resource pot like farm runtime or web worker
    if resource_pot.resource_pot_type == ResourcePotType::DynamicEntryJs {
      let module_graph = context.module_graph.read();
      let result = concatenate_modules_ast(
        resource_pot.entry_module.as_ref().unwrap(),
        &resource_pot.modules,
        &module_graph,
        context,
      )
      .map_err(|err| {
        CompilationError::GenericError(format!("failed to concatenate runtime modules: {}", err))
      })?;

      context
        .meta
        .set_resource_pot_source_map(&resource_pot.id, result.source_map);

      return Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
        ast: result.ast,
        external_modules: result
          .external_modules
          .into_iter()
          .map(|(_, id)| id.to_string())
          .collect(),
        rendered_modules: result.module_ids,
        comments: result.comments,
      })));
    }

    Ok(None)
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginGenerateResourcesHookResult>> {
    if let ResourcePotMetaData::Js(JsResourcePotMetaData {
      ast,
      comments: merged_comments,
      ..
    }) = resource_pot.meta.clone()
    {
      let module_graph = context.module_graph.read();
      let merged_sourcemap = context.meta.get_resource_pot_source_map(&resource_pot.id);
      let (code, map) = generate_code_and_sourcemap(
        resource_pot,
        &module_graph,
        &ast,
        merged_sourcemap,
        merged_comments.into(),
        context,
      )?;

      let create_resource = |content: String, ty: ResourceType| Resource {
        name: resource_pot.name.to_string(),
        name_hash: resource_pot.modules_name_hash.to_string(),
        bytes: content.into_bytes(),
        emitted: false,
        should_transform_output_filename: true,
        resource_type: ty,
        origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
        meta: Default::default(),
      };

      Ok(Some(PluginGenerateResourcesHookResult {
        resources: vec![GeneratedResource {
          resource: create_resource(code, ResourceType::Js),
          source_map: map.map(|content| {
            create_resource(
              content,
              ResourceType::SourceMap(resource_pot.id.to_string()),
            )
          }),
        }],
      }))
    } else {
      return Ok(None);
    }
  }
}

impl FarmPluginScript {
  pub fn new(_config: &Config) -> Self {
    #[cfg(feature = "swc_plugin")]
    init_plugin_module_cache_once(_config);
    Self {}
  }
}

pub fn generate_code_and_sourcemap(
  resource_pot: &ResourcePot,
  module_graph: &ModuleGraph,
  wrapped_resource_pot_ast: &SwcModule,
  merged_sourcemap: Arc<SourceMap>,
  merged_comments: SingleThreadedComments,
  context: &Arc<CompilationContext>,
) -> Result<(String, Option<String>)> {
  let sourcemap_enabled = context.config.sourcemap.enabled(resource_pot.immutable);

  let mut mappings = vec![];
  let code_bytes = codegen_module(
    &wrapped_resource_pot_ast,
    context.config.script.target.clone(),
    merged_sourcemap.clone(),
    if sourcemap_enabled {
      Some(&mut mappings)
    } else {
      None
    },
    context.config.minify.enabled(),
    Some(CodeGenCommentsConfig {
      comments: &merged_comments,
      // preserve all comments when generate module code.
      config: &context.config.comments,
    }),
  )
  .map_err(|e| CompilationError::RenderScriptModuleError {
    id: resource_pot.id.to_string(),
    source: Some(Box::new(e)),
  })?;

  let mut map = None;
  if sourcemap_enabled {
    let sourcemap = build_sourcemap(merged_sourcemap, &mappings);
    // trace sourcemap chain of each module
    let sourcemap = trace_module_sourcemap(sourcemap, module_graph, &context.config.root);

    let mut chain = resource_pot
      .source_map_chain
      .iter()
      .map(|s| JsonSourceMap::from_slice(s.as_bytes()).unwrap())
      .collect::<Vec<_>>();
    chain.push(sourcemap);
    // collapse sourcemap chain
    let sourcemap = collapse_sourcemap_chain(
      chain,
      CollapseSourcemapOptions {
        inline_content: true,
        remap_source: None,
      },
    );

    let mut buf = vec![];
    sourcemap
      .to_writer(&mut buf)
      .map_err(|e| CompilationError::RenderScriptModuleError {
        id: resource_pot.id.to_string(),
        source: Some(Box::new(e)),
      })?;
    let sourcemap = String::from_utf8(buf).unwrap();

    map = Some(sourcemap);
  }

  let code = String::from_utf8(code_bytes).unwrap();

  Ok((code, map))
}
