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
    ) -> Result<Dependency, BindGenError> {

    // TODO: Collect span info?
    match decl {

        // TODO: Collect import names for later?
        ModuleDecl::Import(ImportDecl {
            src,
            ..
        }) => Ok(Dependency(src)),

        // TODO: Collect items for re-export
        ModuleDecl::ExportDecl(ExportDecl { .. }) => todo!(),

        ModuleDecl::ExportNamed(NamedExport) => todo!(),

        ModuleDecl::ExportDefaultDecl(ExportDefaultDecl) => todo!(),

        ModuleDecl::ExportDefaultExpr(ExportDefaultExpr) => todo!(),

        ModuleDecl::ExportAll(ExportAll {
            src,
            ..
        }) => Ok(Dependency(src)),

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


        _ => todo!(),
    }
}
