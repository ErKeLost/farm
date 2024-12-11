use std::ffi::OsStr;

use crate::{
  script::defined_idents_collector::DefinedIdentsCollector,
  swc_ecma_visit::{noop_visit_mut_type, VisitMut, VisitMutWith, VisitWith},
};
use farmfe_core::{
  regex::Regex,
  swc_common::{util::take::Take, Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ArrowExpr, AssignExpr, AssignOp, AssignTarget, BindingIdent, BlockStmt, BlockStmtOrExpr,
    CallExpr, Callee, Class, ClassDecl, ClassExpr, Decl, ExportAll, ExportDecl, ExportDefaultDecl,
    ExportDefaultExpr, Expr, ExprOrSpread, ExprStmt, FnDecl, FnExpr, Function, Id, Ident,
    IdentName, ImportDecl, ImportSpecifier, KeyValueProp, Lit, MemberExpr, MemberProp,
    Module as SwcModule, ModuleDecl, ModuleExportName, ModuleItem, NamedExport, Pat, Prop,
    ReturnStmt, SimpleAssignTarget, Stmt, Str, VarDecl, VarDeclKind, VarDeclarator,
  },
  HashMap,
};

///
/// only return fn name
///
pub trait RuntimeCalleeAllocator {
  ///
  /// ```js
  /// Object.defineProperty(to, to_k, {
  ///   enumerable: true,
  ///   get
  /// });
  ///
  /// ```
  /// ```ts
  /// #fn(target: object, to_k: string, get: () => any);
  /// ```
  ///
  fn define_property_callee(&self) -> Box<Expr> {
    unimplemented!()
  }

  ///
  /// ```js
  /// exports.xx = xx
  /// ```
  ///
  /// ```ts
  /// #fn(to: object, to_k: string, val: any)
  /// ```
  ///
  fn cjs_export_named_callee(&self) -> Box<Expr> {
    unimplemented!()
  }

  ///
  /// ```js
  /// Object.defineProperty(target, "__esModule", {
  ///   value: true,
  /// })
  /// ```
  ///
  /// ```js
  /// #fn(target: Object)
  /// ```
  ///
  fn es_module_flag_callee(&self) -> Box<Expr> {
    unimplemented!()
  }

  ///
  /// ```js
  /// `export * from`
  /// ```
  ///
  /// ```ts
  /// #fn(to: object, from: object)
  /// ```
  ///
  fn export_star_callee(&self) -> Box<Expr> {
    unimplemented!()
  }

  ///
  /// ```js
  /// `import default from`
  /// ```
  ///
  /// ```ts
  /// #fn(obj: object).default
  /// ```
  ///
  fn import_default_callee(&self) -> Box<Expr> {
    unimplemented!()
  }

  ///
  /// ```js
  /// `import * as ns from`
  /// ```
  ///
  /// ```ts
  /// #fn(obj: object, nodeInterop: any);
  /// ```
  ///
  fn import_namespace_callee(&self) -> Box<Expr> {
    unimplemented!()
  }

  ///
  /// ```js
  /// export { xx } from "helper"
  /// ```
  ///
  /// ```ts
  /// #fn(target: object, to_k: string, from_k: string);
  /// ```
  ///
  fn esm_export_named_callee(&self) -> Box<Expr> {
    unimplemented!()
  }

  /// xxx.default
  fn interop_default(&self) -> Box<Expr> {
    unimplemented!()
  }
}

pub struct OriginalRuntimeCallee {
  pub unresolved_mark: Mark,
}

fn create_module_member_expr(ident: &str, unresolved_mark: Mark) -> Expr {
  Expr::Member(MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(Expr::Ident(Ident::new(
      FARM_MODULE_SYSTEM_MODULE.into(),
      DUMMY_SP,
      SyntaxContext::empty().apply_mark(unresolved_mark),
    ))),
    prop: MemberProp::Ident(ident.into()),
  })
}

impl RuntimeCalleeAllocator for OriginalRuntimeCallee {
  fn define_property_callee(&self) -> Box<Expr> {
    Box::new(create_module_member_expr("o", self.unresolved_mark))
  }

  fn es_module_flag_callee(&self) -> Box<Expr> {
    Box::new(create_module_member_expr("_m", self.unresolved_mark))
  }

  fn export_star_callee(&self) -> Box<Expr> {
    Box::new(create_module_member_expr("_e", self.unresolved_mark))
  }

  fn esm_export_named_callee(&self) -> Box<Expr> {
    Box::new(create_module_member_expr("_", self.unresolved_mark))
  }

  fn import_namespace_callee(&self) -> Box<Expr> {
    Box::new(create_module_member_expr("w", self.unresolved_mark))
  }

  fn cjs_export_named_callee(&self) -> Box<Expr> {
    Box::new(create_module_member_expr("d", self.unresolved_mark))
  }

  fn interop_default(&self) -> Box<Expr> {
    Box::new(create_module_member_expr("f", self.unresolved_mark))
  }

  fn import_default_callee(&self) -> Box<Expr> {
    Box::new(create_module_member_expr("i", self.unresolved_mark))
  }
}

const FARM_MODULE_SYSTEM_MODULE: &str = "module";
const FARM_MODULE_SYSTEM_REQUIRE: &str = "require";
const FARM_MODULE_SYSTEM_DEFAULT: &str = "default";
const FARM_MODULE_SYSTEM_EXPORTS: &str = "exports";

struct ExportModuleItem {
  declare_items: Vec<ModuleItem>,
  export_items: Vec<ModuleItem>,
}

pub struct TransformModuleDeclsOptions {
  pub is_target_legacy: bool,
}

/// Transform import statement to cjs require/exports. Farm doesn't use swc commonjs transformer because it's output is too large.
/// Example, transform:
/// ```js
/// import { a, c as d } from "./a";
/// import * as b from "./b";
/// import e from "./c";
///
/// export { a, d, b, e };
/// export default 'hello';
/// export const f = 1;
/// export function g() {}
/// export * from "./d";
/// ```
/// To:
/// ```js
/// var ra = require("./a");
/// var b = require("./b");
/// var re = require("./c");
///
/// exports.a = ra.a;
/// exports.d = ra.c;
/// exports.b = b;
/// exports.e = re.default;
/// exports.default = 'hello';
/// exports.f = 1;
/// exports.g = function g() {};
///
/// module._e(exports, require("./d"));
///
/// ```
pub fn transform_module_decls<F: RuntimeCalleeAllocator>(
  ast: &mut SwcModule,
  unresolved_mark: Mark,
  callee_allocator: &F,
  options: TransformModuleDeclsOptions,
) {
  let mut items = vec![];
  // all import items should be placed at the top of the module
  let mut import_items = vec![];
  let mut export_items = vec![];
  let mut import_bindings_map = HashMap::default();
  let mut is_es_module = false;

  for item in ast.body.take() {
    match item {
      ModuleItem::ModuleDecl(module_decl) => {
        is_es_module = true;

        match module_decl {
          ModuleDecl::Import(import_decl) => {
            import_items.extend(transform_import_decl(
              import_decl,
              unresolved_mark,
              callee_allocator,
              &mut import_bindings_map,
            ));
          }
          ModuleDecl::ExportDecl(export_decl) => {
            let export =
              transform_export_decl(export_decl, callee_allocator, unresolved_mark, &options);
            items.extend(export.declare_items);
            export_items.extend(export.export_items);
          }
          ModuleDecl::ExportNamed(export_named) => {
            let export =
              transform_export_named(export_named, unresolved_mark, callee_allocator, &options);

            items.extend(export.declare_items);
            export_items.extend(export.export_items);
          }
          ModuleDecl::ExportDefaultDecl(default_decl) => {
            let export = transform_export_default_decl(
              default_decl,
              callee_allocator,
              unresolved_mark,
              &options,
            );
            items.extend(export.declare_items);
            export_items.extend(export.export_items);
          }
          ModuleDecl::ExportDefaultExpr(export_expr) => {
            items.extend(transform_export_default_expr(export_expr, unresolved_mark));
          }
          ModuleDecl::ExportAll(export_all) => {
            items.extend(transform_export_all(
              export_all,
              callee_allocator,
              unresolved_mark,
            ));
          }
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => unreachable!(),
        }
      }

      ModuleItem::Stmt(stmt) => items.push(ModuleItem::Stmt(stmt)),
    }
  }

  export_items.extend(import_items);
  export_items.extend(items);
  let mut items = export_items;

  let mut handler = ImportBindingsHandler::new(import_bindings_map);

  for item in &mut items {
    if let ModuleItem::Stmt(stmt) = item {
      stmt.visit_mut_with(&mut handler);
    }
  }

  if is_es_module {
    items.insert(
      0,
      ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Call(CallExpr {
          span: DUMMY_SP,
          callee: Callee::Expr(callee_allocator.es_module_flag_callee()),
          args: vec![ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Ident(create_exports_ident(unresolved_mark))),
          }],
          type_args: None,
          ctxt: SyntaxContext::default(),
        })),
      })),
    )
  }

  ast.body = items;
}

fn transform_import_decl(
  import_decl: ImportDecl,
  unresolved_mark: Mark,
  callee_allocator: &dyn RuntimeCalleeAllocator,
  import_bindings_map: &mut HashMap<Id, Expr>,
) -> Vec<ModuleItem> {
  let mut items = vec![];

  if import_decl.specifiers.is_empty() {
    items.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
      span: DUMMY_SP,
      expr: create_require_call_expr(*import_decl.src, unresolved_mark),
    })));
    return items;
  }

  // 1. push var val_name = require(src);
  let val_name_ident = create_require_val_ident(import_decl.src.value.as_str());

  let mut contains_default = false;
  let mut contains_named = false;

  for specifier in import_decl.specifiers {
    match specifier {
      ImportSpecifier::Named(specifier) => {
        contains_named = true;
        // 2. push var specifier.local = val_name.imported;
        let specifier_ident = specifier.local;
        let init = if let Some(imported) = specifier.imported {
          let imported_ident = get_ident_from_module_export_name(imported);
          MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(val_name_ident.clone())),
            prop: MemberProp::Ident(IdentName::new(imported_ident.sym, imported_ident.span)),
          }
        } else {
          MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(val_name_ident.clone())),
            prop: MemberProp::Ident(IdentName::new(specifier_ident.sym.clone(), DUMMY_SP)),
          }
        };
        import_bindings_map.insert(specifier_ident.to_id(), Expr::Member(init));
      }
      ImportSpecifier::Default(specifier) => {
        contains_default = true;
        // module.f(val_name_ident)
        let init = create_module_helper_call_expr(
          callee_allocator.interop_default(),
          vec![ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Ident(val_name_ident.clone())),
          }],
        );

        import_bindings_map.insert(specifier.local.to_id(), Expr::Call(init));
      }
      ImportSpecifier::Namespace(specifier) => {
        items.push(create_module_helper_item(
          callee_allocator.import_namespace_callee(),
          val_name_ident.clone(),
          *import_decl.src.clone(),
          unresolved_mark,
        ));
        items.push(create_var_decl_stmt(
          specifier.local,
          Box::new(Expr::Ident(val_name_ident.clone())),
        ));
      }
    }
  }

  if contains_named && contains_default {
    items.push(create_module_helper_item(
      callee_allocator.import_namespace_callee(),
      val_name_ident.clone(),
      *import_decl.src.clone(),
      unresolved_mark,
    ));
  } else if contains_default {
    items.push(create_module_helper_item(
      callee_allocator.import_default_callee(),
      val_name_ident.clone(),
      *import_decl.src.clone(),
      unresolved_mark,
    ));
  } else if contains_named {
    let require_item = create_var_decl_stmt(
      val_name_ident,
      create_require_call_expr(*import_decl.src, unresolved_mark),
    );
    items.push(require_item);
  }

  items
}

fn transform_export_decl(
  export_decl: ExportDecl,
  callee_allocator: &dyn RuntimeCalleeAllocator,
  unresolved_mark: Mark,
  options: &TransformModuleDeclsOptions,
) -> ExportModuleItem {
  let mut export = ExportModuleItem {
    declare_items: vec![],
    export_items: vec![],
  };

  match export_decl.decl {
    Decl::Class(class_decl) => {
      return create_export_class_decl_stmts(
        Some(class_decl.ident.clone()),
        class_decl.ident,
        class_decl.class,
        callee_allocator,
        unresolved_mark,
        options.is_target_legacy,
      )
    }
    Decl::Fn(fn_decl) => {
      return create_export_fn_decl_stmts(
        Some(fn_decl.ident.clone()),
        fn_decl.ident,
        fn_decl.function,
        callee_allocator,
        unresolved_mark,
        options.is_target_legacy,
      )
    }
    Decl::Var(mut var_decls) => {
      let mut local_items = vec![];

      for var_decl in &var_decls.decls {
        let mut idents_collector = DefinedIdentsCollector::new();
        var_decl.name.visit_with(&mut idents_collector);
        let mut defined_idents = idents_collector
          .defined_idents
          .into_iter()
          .collect::<Vec<_>>();
        defined_idents.sort();

        for ident in defined_idents {
          let call_expr = create_define_export_property_ident_call_expr(
            None,
            ident,
            callee_allocator,
            unresolved_mark,
            options.is_target_legacy,
          );
          local_items.push(create_module_item_from_call_expr(call_expr));
        }
      }

      // transform let/const to var
      var_decls.kind = VarDeclKind::Var;

      export
        .declare_items
        .push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decls))));
      export.export_items.extend(local_items);
    }
    _ => unreachable!("invalid export decl when rendering module system"),
  }

  export
}

fn transform_export_named(
  named_export: NamedExport,
  unresolved_mark: Mark,
  callee_allocator: &dyn RuntimeCalleeAllocator,
  options: &TransformModuleDeclsOptions,
) -> ExportModuleItem {
  let mut items = vec![];
  let mut export_items = vec![];
  let mut cached_export_from_item = None;

  let mut contains_default = false;
  let mut contains_named = false;
  let mut extra_items = vec![];

  for export_specifier in named_export.specifiers {
    match export_specifier {
      farmfe_core::swc_ecma_ast::ExportSpecifier::Namespace(specifier) => {
        let ident = get_ident_from_module_export_name(specifier.name);
        // module.o(exports, ident, () => ident)
        let call_expr = create_define_export_property_ident_call_expr(
          Some(ident.to_id()),
          ident.to_id(),
          callee_allocator,
          unresolved_mark,
          options.is_target_legacy,
        );
        export_items.push(create_module_item_from_call_expr(call_expr));
        // var ident = module.w(require(src))
        items.push(create_module_helper_item(
          callee_allocator.import_namespace_callee(),
          ident,
          *named_export.src.clone().unwrap(),
          unresolved_mark,
        ));
      }
      farmfe_core::swc_ecma_ast::ExportSpecifier::Named(specifier) => {
        let (exported_ident, local_ident) = if let Some(exported) = specifier.exported {
          let exported_ident = get_ident_from_module_export_name(exported);
          let local_ident = get_ident_from_module_export_name(specifier.orig);
          (exported_ident, local_ident)
        } else {
          let orig_ident = get_ident_from_module_export_name(specifier.orig);
          (orig_ident.clone(), orig_ident)
        };

        if local_ident.sym == "default" {
          contains_default = true;
        } else {
          contains_named = true;
        }

        if let Some(src) = &named_export.src {
          let export_from_ident = cached_export_from_item
            .clone()
            .unwrap_or(create_require_val_ident(src.value.as_str()));
          let is_equal = exported_ident.to_id() == local_ident.to_id();
          let mut args = vec![
            ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Ident(create_exports_ident(unresolved_mark))),
            },
            ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: exported_ident.sym,
                raw: None,
              }))),
            },
            ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Ident(export_from_ident.clone())),
            },
          ];
          if !is_equal {
            args.push(ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: local_ident.sym,
                raw: None,
              }))),
            })
          }

          cached_export_from_item = Some(export_from_ident);
          // module._(exports, exported_ident, export_from_ident, local_ident)
          let call_expr =
            create_module_helper_call_expr(callee_allocator.esm_export_named_callee(), args);

          extra_items.push(create_module_item_from_call_expr(call_expr));
        } else {
          let call_expr = create_define_export_property_ident_call_expr(
            Some(exported_ident.to_id()),
            local_ident.to_id(),
            callee_allocator,
            unresolved_mark,
            options.is_target_legacy,
          );
          export_items.push(create_module_item_from_call_expr(call_expr));
        }
      }
      farmfe_core::swc_ecma_ast::ExportSpecifier::Default(_) => {
        unreachable!("`export v from 'mod'` is invalid")
      }
    }
  }

  if let Some(export_from_ident) = cached_export_from_item {
    if contains_named && contains_default {
      items.push(create_module_helper_item(
        callee_allocator.import_namespace_callee(),
        export_from_ident.clone(),
        *named_export.src.clone().unwrap(),
        unresolved_mark,
      ));
    } else if contains_default {
      items.push(create_module_helper_item(
        callee_allocator.import_default_callee(),
        export_from_ident.clone(),
        *named_export.src.clone().unwrap(),
        unresolved_mark,
      ));
    } else if contains_named {
      let require_item = create_var_decl_stmt(
        export_from_ident,
        create_require_call_expr(*named_export.src.clone().unwrap(), unresolved_mark),
      );
      items.push(require_item);
    }
  }

  items.extend(extra_items);

  ExportModuleItem {
    declare_items: items,
    export_items: export_items,
  }
}

fn transform_export_default_decl(
  default_decl: ExportDefaultDecl,
  callee_allocator: &dyn RuntimeCalleeAllocator,
  unresolved_mark: Mark,
  options: &TransformModuleDeclsOptions,
) -> ExportModuleItem {
  match default_decl.decl {
    farmfe_core::swc_ecma_ast::DefaultDecl::Class(class_decl) => {
      let exported_ident = Ident::new(
        FARM_MODULE_SYSTEM_DEFAULT.into(),
        DUMMY_SP,
        SyntaxContext::empty(),
      );
      return create_export_class_decl_stmts(
        class_decl.ident,
        exported_ident,
        class_decl.class,
        callee_allocator,
        unresolved_mark,
        options.is_target_legacy,
      );
    }
    farmfe_core::swc_ecma_ast::DefaultDecl::Fn(fn_decl) => {
      let exported_ident = Ident::new(
        FARM_MODULE_SYSTEM_DEFAULT.into(),
        DUMMY_SP,
        SyntaxContext::empty(),
      );
      return create_export_fn_decl_stmts(
        fn_decl.ident,
        exported_ident,
        fn_decl.function,
        callee_allocator,
        unresolved_mark,
        options.is_target_legacy,
      );
    }
    farmfe_core::swc_ecma_ast::DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
  }
}

// export default 'foo';
// export default foo;
fn transform_export_default_expr(
  export_expr: ExportDefaultExpr,
  unresolved_mark: Mark,
) -> Vec<ModuleItem> {
  let mut items = vec![];

  let exports_assign_left = create_exports_assign_left(
    Ident::new(
      FARM_MODULE_SYSTEM_DEFAULT.into(),
      DUMMY_SP,
      SyntaxContext::empty(),
    ),
    unresolved_mark,
  );

  items.push(create_exports_assign_stmt(
    exports_assign_left,
    *export_expr.expr,
  ));
  items
}

fn transform_export_all(
  export_all: ExportAll,
  callee_allocator: &dyn RuntimeCalleeAllocator,
  unresolved_mark: Mark,
) -> Vec<ModuleItem> {
  let mut items = vec![];
  let (require_item, val_name_ident) = create_require_stmt(*export_all.src, unresolved_mark);
  items.push(require_item);

  // module._e(exports, val_name_ident)
  let callee = Callee::Expr(callee_allocator.export_star_callee());

  let call_expr = Expr::Call(CallExpr {
    span: DUMMY_SP,
    callee,
    args: vec![
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Ident(create_exports_ident(unresolved_mark))),
      },
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Ident(val_name_ident.clone())),
      },
    ],
    type_args: None,
    ctxt: SyntaxContext::empty(),
  });
  items.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(call_expr),
  })));

  items
}

fn get_name_from_src(src: &str) -> String {
  let path = std::path::PathBuf::from(src);
  let regex = Regex::new("[^A-Za-z0-9_]").unwrap();
  let val_name = path
    .file_stem()
    .unwrap_or(OsStr::new("_"))
    .to_string_lossy()
    .to_string();
  let val_name = regex.replace_all(&val_name, "_");

  assert!(val_name.len() > 0);

  format!("_f_{val_name}")
}

fn create_var_decl_stmt(val_name_ident: Ident, init: Box<Expr>) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Var,
    declare: false,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent {
        id: val_name_ident,
        type_ann: None,
      }),
      init: Some(init),
      definite: false,
    }],
    ctxt: SyntaxContext::empty(),
  }))))
}

fn create_require_call_expr(src: Str, unresolved_mark: Mark) -> Box<Expr> {
  Box::new(Expr::Call(CallExpr {
    span: DUMMY_SP,
    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
      FARM_MODULE_SYSTEM_REQUIRE.into(),
      DUMMY_SP,
      SyntaxContext::empty().apply_mark(unresolved_mark),
    )))),
    args: vec![ExprOrSpread {
      spread: None,
      expr: Box::new(Expr::Lit(Lit::Str(src))),
    }],
    type_args: None,
    ctxt: SyntaxContext::empty(),
  }))
}

fn create_require_stmt(src: Str, unresolved_mark: Mark) -> (ModuleItem, Ident) {
  let val_name_ident = create_require_val_ident(src.value.as_str());
  (
    create_var_decl_stmt(
      val_name_ident.clone(),
      create_require_call_expr(src, unresolved_mark),
    ),
    val_name_ident,
  )
}

fn create_require_val_ident(src: &str) -> Ident {
  let val_name = get_name_from_src(src);
  Ident::new(
    val_name.into(),
    DUMMY_SP,
    SyntaxContext::empty().apply_mark(Mark::new()),
  )
}

fn create_exports_assign_left(exported_ident: Ident, unresolved_mark: Mark) -> AssignTarget {
  AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(Expr::Ident(create_exports_ident(unresolved_mark))),
    prop: MemberProp::Ident(IdentName::new(exported_ident.sym, exported_ident.span)),
  }))
}

fn create_exports_ident(unresolved_mark: Mark) -> Ident {
  Ident::new(
    FARM_MODULE_SYSTEM_EXPORTS.into(),
    DUMMY_SP,
    SyntaxContext::empty().apply_mark(unresolved_mark),
  )
}

fn create_exports_assign_stmt(
  exports_assign_left: AssignTarget,
  export_assign_right: Expr,
) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Assign(AssignExpr {
      span: DUMMY_SP,
      op: AssignOp::Assign,
      left: exports_assign_left,
      right: Box::new(export_assign_right),
    })),
  }))
}

fn create_export_fn_decl_stmts(
  fn_ident: Option<Ident>,
  exports_ident: Ident,
  function: Box<Function>,
  callee_allocator: &dyn RuntimeCalleeAllocator,
  unresolved_mark: Mark,
  is_target_legacy: bool,
) -> ExportModuleItem {
  let mut export = ExportModuleItem {
    export_items: vec![],
    declare_items: vec![],
  };
  // 1. create fn decl item
  let exports_assign_right = if let Some(ident) = fn_ident {
    let fn_decl = ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
      ident: ident.clone(),
      declare: false,
      function,
    })));
    export.declare_items.push(fn_decl);

    Expr::Ident(ident)
  } else {
    Expr::Fn(FnExpr {
      ident: None,
      function,
    })
  };

  // 2. create exports assign item
  if let Expr::Ident(ident) = exports_assign_right {
    let call_expr = create_define_export_property_ident_call_expr(
      Some(exports_ident.to_id()),
      ident.to_id(),
      callee_allocator,
      unresolved_mark,
      is_target_legacy,
    );
    export
      .export_items
      .push(create_module_item_from_call_expr(call_expr));
  } else {
    let exports_assign_left = create_exports_assign_left(exports_ident, unresolved_mark);
    export.declare_items.push(create_exports_assign_stmt(
      exports_assign_left,
      exports_assign_right,
    ));
  }

  export
}

fn create_export_class_decl_stmts(
  class_ident: Option<Ident>,
  exports_ident: Ident,
  class: Box<Class>,
  callee_allocator: &dyn RuntimeCalleeAllocator,
  unresolved_mark: Mark,
  is_target_legacy: bool,
) -> ExportModuleItem {
  let mut export = ExportModuleItem {
    export_items: vec![],
    declare_items: vec![],
  };
  // 1. create class decl item
  let exports_assign_right = if let Some(ident) = class_ident {
    let fn_decl = ModuleItem::Stmt(Stmt::Decl(Decl::Class(ClassDecl {
      ident: ident.clone(),
      declare: false,
      class,
    })));
    export.declare_items.push(fn_decl);

    Expr::Ident(ident)
  } else {
    Expr::Class(ClassExpr { ident: None, class })
  };

  // 2. create exports assign item
  if let Expr::Ident(ident) = exports_assign_right {
    let call_expr = create_define_export_property_ident_call_expr(
      Some(exports_ident.to_id()),
      ident.to_id(),
      callee_allocator,
      unresolved_mark,
      is_target_legacy,
    );
    export
      .export_items
      .push(create_module_item_from_call_expr(call_expr));
  } else {
    let exports_assign_left = create_exports_assign_left(exports_ident, unresolved_mark);
    export.declare_items.push(create_exports_assign_stmt(
      exports_assign_left,
      exports_assign_right,
    ));
  }

  export
}

// fn create_module_helper_callee(helper: &str, unresolved_mark: Mark) -> Callee {
//   let prop = Ident::new(helper.into(), DUMMY_SP);
//   Callee::Expr(Box::new(Expr::Member(MemberExpr {
//     span: DUMMY_SP,
//     obj: Box::new(Expr::Ident(Ident::new(
//       FARM_MODULE_SYSTEM_MODULE.into(),
//       DUMMY_SP.apply_mark(unresolved_mark),
//     ))),
//     prop: MemberProp::Ident(prop),
//   })))
// }

fn create_module_helper_item(
  helper: Box<Expr>,
  val_name_ident: Ident,
  src: Str,
  unresolved_mark: Mark,
) -> ModuleItem {
  let prop = ExprOrSpread {
    spread: None,
    expr: create_require_call_expr(src, unresolved_mark),
  };
  create_var_decl_stmt(
    val_name_ident,
    Box::new(Expr::Call(create_module_helper_call_expr(
      helper,
      vec![prop],
    ))),
  )
}

fn create_module_helper_call_expr(helper: Box<Expr>, args: Vec<ExprOrSpread>) -> CallExpr {
  let callee = Callee::Expr(helper);
  let call_expr = CallExpr {
    span: DUMMY_SP,
    callee,
    args,
    type_args: None,
    ctxt: SyntaxContext::empty(),
  };
  call_expr
}

fn create_define_export_property_ident_call_expr(
  exported_ident: Option<Id>,
  local_ident: Id,
  callee_allocator: &dyn RuntimeCalleeAllocator,
  unresolved_mark: Mark,
  is_target_legacy: bool,
) -> CallExpr {
  let exported_ident = if let Some(exported_ident) = exported_ident {
    exported_ident
  } else {
    local_ident.clone()
  };
  let expr = if is_target_legacy {
    Expr::Fn(FnExpr {
      ident: None,
      function: Box::new(Function {
        params: vec![],
        decorators: vec![],
        span: DUMMY_SP,
        body: Some(BlockStmt {
          span: DUMMY_SP,
          stmts: vec![Stmt::Return(ReturnStmt {
            span: DUMMY_SP,
            arg: Some(Box::new(Expr::Ident(Ident::new(
              local_ident.0,
              DUMMY_SP,
              local_ident.1,
            )))),
          })],
          ctxt: SyntaxContext::empty(),
        }),
        is_generator: false,
        is_async: false,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
      }),
    })
  } else {
    Expr::Arrow(ArrowExpr {
      span: DUMMY_SP,
      params: vec![],
      body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Ident(Ident::new(
        local_ident.0,
        DUMMY_SP,
        local_ident.1,
      ))))),
      is_generator: false,
      is_async: false,
      return_type: None,
      type_params: None,
      ctxt: SyntaxContext::empty(),
    })
  };
  // module.o(exports, ident, function(){return ident;})
  create_module_helper_call_expr(
    callee_allocator.define_property_callee(),
    vec![
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Ident(create_exports_ident(unresolved_mark))),
      },
      ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Str(Str {
          span: DUMMY_SP,
          value: exported_ident.0.clone(),
          raw: None,
        }))),
      },
      ExprOrSpread {
        spread: None,
        expr: Box::new(expr),
      },
    ],
  )
}

fn create_module_item_from_call_expr(call_expr: CallExpr) -> ModuleItem {
  ModuleItem::Stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: Box::new(Expr::Call(call_expr)),
  }))
}

fn get_ident_from_module_export_name(name: ModuleExportName) -> Ident {
  match name {
    ModuleExportName::Ident(ident) => ident,
    ModuleExportName::Str(_) => unreachable!("invalid `str` export as"),
  }
}

struct ImportBindingsHandler {
  import_bindings_map: HashMap<Id, Expr>,
}

impl ImportBindingsHandler {
  pub fn new(import_bindings_map: HashMap<Id, Expr>) -> Self {
    Self {
      import_bindings_map,
    }
  }
}

/// @License MIT SWC Project.
/// This binding handler is modified from swc and modified by brightwu
impl VisitMut for ImportBindingsHandler {
  noop_visit_mut_type!();

  /// replace bar in binding pattern
  /// input:
  /// ```JavaScript
  /// const foo = { bar }
  /// ```
  /// output:
  /// ```JavaScript
  /// const foo = { bar: baz }
  /// ```
  fn visit_mut_prop(&mut self, n: &mut Prop) {
    if let Prop::Shorthand(shorthand) = n {
      if let Some(expr) = self.import_bindings_map.get(&shorthand.to_id()) {
        *n = KeyValueProp {
          key: shorthand.take().into(),
          value: Box::new(expr.clone()),
        }
        .into()
      }
    } else {
      n.visit_mut_children_with(self)
    }
  }

  fn visit_mut_expr(&mut self, n: &mut Expr) {
    if let Expr::Ident(ident) = n {
      let id = ident.to_id();
      if let Some(member_expr) = self.import_bindings_map.get(&id) {
        *n = member_expr.clone();
      }
    } else {
      n.visit_mut_children_with(self);
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use crate::{
    common::{create_swc_source_map, Source},
    script::{codegen_module, parse_module, swc_try_with::try_with},
  };
  use farmfe_core::{swc_common::Globals, swc_ecma_ast::EsVersion, swc_ecma_parser::Syntax};

  use super::*;

  #[test]
  fn test_get_name_from_src() {
    assert_eq!(get_name_from_src("./a"), "_f_a");
    assert_eq!(get_name_from_src("./a.js"), "_f_a");
    assert_eq!(get_name_from_src("./a.jsx"), "_f_a");
    assert_eq!(get_name_from_src("b"), "_f_b");
    assert_eq!(get_name_from_src("src/index.css"), "_f_index");
    assert_eq!(get_name_from_src("src/button-comp.css"), "_f_button_comp");
  }

  #[test]
  fn test_transform_module_decls() {
    let path = "any";
    let content = r#"
import { a, c as d, default as de } from "./a";
import * as b from "./b";
import e from "./c";

console.log(de);

export * from './c';
export { a, d, b, e };
export { a1, d1, b1, e1 as e2} from './d';
export * as b2 from './d';

export const f = 1, h = 2;
export function g() {}
export class i {}

export default 'hello';
export default class j {}
export default function k() {}

export * from './e';
    "#;
    let (cm, _) = create_swc_source_map(Source {
      path: std::path::PathBuf::from(path),
      content: Arc::new(content.to_string()),
    });
    let mut ast = parse_module(
      path,
      content,
      Syntax::Es(Default::default()),
      EsVersion::latest(),
    )
    .unwrap()
    .ast;

    try_with(cm.clone(), &Globals::new(), || {
      let callee_allocator = OriginalRuntimeCallee {
        unresolved_mark: Mark::new(),
      };

      transform_module_decls(
        &mut ast,
        Mark::new(),
        &callee_allocator,
        TransformModuleDeclsOptions {
          is_target_legacy: true,
        },
      );

      let code_bytes =
        codegen_module(&mut ast, EsVersion::latest(), cm, None, false, None).unwrap();
      let code = String::from_utf8(code_bytes).unwrap();

      println!("{code}");

      assert_eq!(
        code,
        r#"module._m(exports);
module.o(exports, "a", function() {
    return _f_a.a;
});
module.o(exports, "d", function() {
    return _f_a.c;
});
module.o(exports, "b", function() {
    return b;
});
module.o(exports, "e", function() {
    return module.f(_f_c);
});
module.o(exports, "b2", function() {
    return b2;
});
module.o(exports, "f", function() {
    return f;
});
module.o(exports, "h", function() {
    return h;
});
module.o(exports, "g", function() {
    return g;
});
module.o(exports, "i", function() {
    return i;
});
module.o(exports, "default", function() {
    return j;
});
module.o(exports, "default", function() {
    return k;
});
var _f_a = require("./a");
var _f_b = module.w(require("./b"));
var b = _f_b;
var _f_c = module.i(require("./c"));
console.log(_f_a.default);
var _f_c = require('./c');
module._e(exports, _f_c);
var _f_d = require('./d');
module._(exports, "a1", _f_d);
module._(exports, "d1", _f_d);
module._(exports, "b1", _f_d);
module._(exports, "e2", _f_d, "e1");
var b2 = module.w(require('./d'));
var f = 1, h = 2;
function g() {}
class i {
}
exports.default = 'hello';
class j {
}
function k() {}
var _f_e = require('./e');
module._e(exports, _f_e);
"#
      )
    })
    .unwrap();
  }

  #[test]
  fn test_transform_module_decls_not_target_legacy() {
    let path = "any";
    let content = r#"
export const f = 1, h = 2;
    "#;
    let (cm, _) = create_swc_source_map(Source {
      path: std::path::PathBuf::from(path),
      content: Arc::new(content.to_string()),
    });
    let mut ast = parse_module(
      path,
      content,
      Syntax::Es(Default::default()),
      EsVersion::latest(),
    )
    .unwrap()
    .ast;

    try_with(cm.clone(), &Globals::new(), || {
      let callee_allocator = OriginalRuntimeCallee {
        unresolved_mark: Mark::new(),
      };
      transform_module_decls(
        &mut ast,
        Mark::new(),
        &callee_allocator,
        TransformModuleDeclsOptions {
          is_target_legacy: false,
        },
      );

      let code_bytes =
        codegen_module(&mut ast, EsVersion::latest(), cm, None, false, None).unwrap();
      let code = String::from_utf8(code_bytes).unwrap();

      println!("{code}");

      assert_eq!(
        code,
        r#"module._m(exports);
module.o(exports, "f", ()=>f);
module.o(exports, "h", ()=>h);
var f = 1, h = 2;
"#
      )
    })
    .unwrap();
  }
}