use std::collections::HashMap;
use std::path::{PathBuf, Path};

use swc_ecma_ast::*;
use super::structures::*;
use super::error::*;

struct Context {
    module_path: PathBuf,
    scope: Scope,
    typing_env: TypeEnv,
}

struct Scope {
    map: HashMap<String, ()>,
}

struct TypeEnv {
    map: HashMap<String, ()>,
}

pub fn process_module(module_path: PathBuf, module: Module) -> Result<BindingModule, BindGenError> {

    let mut context = Context {
        module_path,
        scope: Scope { map: HashMap::new() },
        typing_env: TypeEnv { map: HashMap::new() },
    };

    let mut depedencies: Vec<Dependency> = Vec::new();
    for module_item in module.body {
        let result = process_module_item(&mut context, &mut depedencies, module_item)?;
    }
    todo!();
}

fn process_module_item(
    context: &mut Context,
    dependencies: &mut Vec<Dependency>,
    item: ModuleItem,
    ) -> Result<(), BindGenError> {


    match item {
        ModuleItem::ModuleDecl(decl) => {
            todo!();
        },

        ModuleItem::Stmt(stmt) => todo!(),
    }

    todo!();
}

fn module_item_dependency(
    context: &mut Context,
    decl: &ModuleDecl
    ) -> Result<Option<Dependency>, BindGenError> {

    // TODO: Collect span info?
    match decl {

        // TODO: Collect import names for later?
        ModuleDecl::Import(ImportDecl {
            ref src,
            ..
        }) => Ok(Some(Dependency(src.clone()))),

        // TODO: Collect items for re-export
        ModuleDecl::ExportDecl(ExportDecl { .. }) => todo!(),

        // TODO: Collect items for re-export
        ModuleDecl::ExportNamed(NamedExport {
            ref src,
            ..
        }) => Ok(src.as_ref().map(|src| Dependency(src.clone()))),

        ModuleDecl::ExportAll(ExportAll {
            ref src,
            ..
        }) => Ok(Some(Dependency(src.clone()))),

        ModuleDecl::ExportDefaultDecl(ExportDefaultDecl { ref span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultExport),
                span: span.clone(),
            })
        }

        ModuleDecl::ExportDefaultExpr(ExportDefaultExpr { ref span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultExport),
                span: span.clone(),
            })
        }

        ModuleDecl::TsImportEquals(TsImportEqualsDecl { ref span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsImportEquals),
                span: span.clone(),
            })
        }

        ModuleDecl::TsExportAssignment(TsExportAssignment { ref span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsExportAssignment),
                span: span.clone(),
            })
        }

        ModuleDecl::TsNamespaceExport(TsNamespaceExportDecl { ref span, .. }) => {

            // TODO: Handle TsNamespaceExport?
            //   What is TsNamespaceExport??
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsNamespaceExport),
                span: span.clone(),
            })
        }
    }
}
