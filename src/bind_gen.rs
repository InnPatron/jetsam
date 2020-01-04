use std::path::{PathBuf, Path};

use swc_ecma_ast::*;
use super::structures::*;
use super::error::*;

pub struct Context {
    pub module_path: PathBuf,
}

pub fn process_module(mut context: self::Context, module: Module) -> Result<BindingModule, BindGenError> {

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

fn process_module_decl(
    context: &mut Context,
    decl: ModuleDecl
    ) -> Result<Option<Dependency>, BindGenError> {

    // TODO: Collect span info?
    match decl {

        // TODO: Collect import names for later?
        ModuleDecl::Import(ImportDecl {
            src,
            ..
        }) => Ok(Some(Dependency(src))),

        // TODO: Collect items for re-export
        ModuleDecl::ExportDecl(ExportDecl { .. }) => todo!(),

        // TODO: Collect items for re-export
        ModuleDecl::ExportNamed(NamedExport {
            src,
            ..
        }) => Ok(src.map(|src| Dependency(src))),

        ModuleDecl::ExportAll(ExportAll {
            src,
            ..
        }) => Ok(Some(Dependency(src))),

        ModuleDecl::ExportDefaultDecl(ExportDefaultDecl { span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultExport),
                span,
            })
        }

        ModuleDecl::ExportDefaultExpr(ExportDefaultExpr { span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultExport),
                span,
            })
        }

        ModuleDecl::TsImportEquals(TsImportEqualsDecl { span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsImportEquals),
                span,
            })
        }

        ModuleDecl::TsExportAssignment(TsExportAssignment { span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsExportAssignment),
                span,
            })
        }

        ModuleDecl::TsNamespaceExport(TsNamespaceExportDecl { span, .. }) => {

            // TODO: Handle TsNamespaceExport?
            //   What is TsNamespaceExport??
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsNamespaceExport),
                span,
            })
        }
    }
}
