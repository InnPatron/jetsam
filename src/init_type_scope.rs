use std::collections::HashMap;

use swc_ecma_ast::*;
use swc_atoms::JsWord;

use super::bind_init::ModuleData;

use super::structures::{ ItemState, Scope, CanonPath };
use super::error::*;


pub fn init(data: &ModuleData)
    -> Result<Scope<ItemState>, BindGenError> {

    let mut init_session = InitSession {
        path: &data.path,
        dependency_map: &data.dependencies,
        scope: Scope::new(),
    };

    for module_item in data.module_ast.body.iter() {
    }
    todo!();
}

struct InitSession<'a> {
    path: &'a CanonPath,
    dependency_map: &'a HashMap<String, CanonPath>,
    scope: Scope<ItemState>,
}

impl<'a> InitSession<'a> {
    fn process_module_item(&mut self, item: &ModuleItem) -> Result<(), BindGenError> {
        match item {
            ModuleItem::ModuleDecl(ref decl) => self.process_module_decl(decl),

            ModuleItem::Stmt(ref stmt) => self.process_stmt(stmt),
        }
    }

    fn process_stmt(&mut self, stmt: &Stmt) -> Result<(), BindGenError> {
        if let Stmt::Decl(ref decl) = stmt {
            self.process_decl(decl, false)?;
        }

        Ok(())
    }

    fn process_module_decl(&mut self, module_decl: &ModuleDecl) -> Result<(), BindGenError> {
        match module_decl {

            ModuleDecl::Import(ref import) => {
                let src_canon_path: &CanonPath =
                    get_dep_src!(self, import.src);

                for specifier in import.specifiers.iter() {
                    self.handle_import_specifier(src_canon_path, specifier)?;
                }

                Ok(())
            },

            ModuleDecl::ExportDecl(ExportDecl {
                ref decl,
                ..
            }) => self.process_decl(decl, true),

            _ => Ok(()),
        }
    }

    fn handle_import_specifier(&mut self, source: &CanonPath, spec: &ImportSpecifier)
        -> Result<(), BindGenError> {
        match spec {
            ImportSpecifier::Specific(ref specific) => {

                let src_key = specific
                    .imported
                    .as_ref()
                    .map(|export_key| export_key.sym.clone())
                    .unwrap_or(specific.local.sym.clone());

                let as_key = specific.local.sym.clone();

                let item_state = ItemState::Imported {
                    source: source.clone(),
                    src_key,
                    as_key,
                };

                let import_key = specific.local.sym.clone();
                self.scope.insert(import_key, item_state);

                Ok(())
            }

            ImportSpecifier::Default(def) => {
                Err(BindGenError {
                    module_path: self.path.as_path().to_owned(),
                    kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultImport),
                    span: def.span,
                })
            }

            ImportSpecifier::Namespace(namespace) => {
                Err(BindGenError {
                    module_path: self.path.as_path().to_owned(),
                    kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultImport),
                    span: namespace.span,
                })
            }
        }
    }

    fn process_decl(&mut self, decl: &Decl, export: bool) -> Result<(), BindGenError> {
        let symbol = match decl {
            Decl::Class(ClassDecl {
                ref ident,
                ..
            }) => Some(ident.sym.clone()),

            Decl::TsInterface(TsInterfaceDecl {
                id,
                ..
            }) => Some(id.sym.clone()),

            Decl::TsTypeAlias(TsTypeAliasDecl {
                id,
                ..
            }) => Some(id.sym.clone()),

            Decl::TsEnum(TsEnumDecl {
                id,
                ..
            }) => {
                Some(id.sym.clone())
            },

            _ => None,
        };

        if let Some(symbol) = symbol {
            self.scope.insert(symbol, ItemState::Rooted);
        }

        Ok(())
    }
}

