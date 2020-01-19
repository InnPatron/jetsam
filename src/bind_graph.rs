use std::collections::HashMap;

use swc_ecma_ast::*;

use super::bind_init;
use super::error::*;
use super::structures::*;

pub struct ModuleGraph {

}

struct Context {
    value_scope: Scope<Nebulous<Value>>,
    type_scope: Scope<Nebulous<Type>>,
}

struct Scope<T> {
    map: HashMap<String, T>,
}

impl<T> Scope<T> {
    fn new() -> Self {
        Scope {
            map: HashMap::new(),
        }
    }

    fn insert(&mut self, key: String, v: T) {
        self.map.insert(key, v);
    }

    fn get(&self, key: &str) -> Option<&T> {
        self.map.get(key)
    }
}

pub fn build_module_graph(
    module_cache: bind_init::ParsedModuleCache
) -> Result<ModuleGraph, BindGenError> {

    let mut module_graph: HashMap<CanonPath, ModuleInfo> = HashMap::new();
    for (canon_path, mut module) in module_cache.0.into_iter() {

        let mut context = Context {
            value_scope: Scope::new(),
            type_scope: Scope::new(),
        };

        let mut module_info = ModuleInfo::new(canon_path.into());

        hoist_imports(&mut module.module_ast);

    }

    todo!();
}

fn hoist_imports(module: &mut Module) {
    use std::mem;

    let capacity = module.body.len();

    let mut module_items = Vec::with_capacity(capacity);
    mem::swap(&mut module_items, &mut module.body);

    let mut other_buffer = Vec::with_capacity(capacity);

    for module_item in module_items {
        match module_item {
            import @ ModuleItem::ModuleDecl(ModuleDecl::Import(..))
                => module.body.push(import),

            other => other_buffer.push(other),
        }
    }

    module.body.append(&mut other_buffer);
}

struct BindGenSession;

impl BindGenSession {
    fn process_module_item(
        &mut self,
        context: &mut Context,
        module_info: &mut ModuleInfo,
        item: ModuleItem,
    ) -> Result<(), BindGenError> {

        match item {
            ModuleItem::ModuleDecl(decl) => self.process_module_decl(context, module_info, decl),

            ModuleItem::Stmt(stmt) => todo!(),
        }
    }

    fn process_module_decl(
        &mut self,
        context: &mut Context,
        module_info: &mut ModuleInfo,
        decl: ModuleDecl
        ) -> Result<(), BindGenError> {

        // TODO: Collect span info?
        match decl {

            ModuleDecl::Import(ImportDecl {
                src,
                span,
                specifiers,
            }) => {
                let dep_module_info = self.bind_module_from_src(context, &src, Some(span))?;

                for specifier in specifiers {
                    match specifier {
                        ImportSpecifier::Specific(ImportSpecific {
                            span,
                            local,
                            imported,
                        }) => {
                            context.value_scope.try_import(
                                &dep_module_info,
                                local.sym.to_string(),
                                imported.as_ref().map(|ident| ident.sym.to_string())
                            );

                            context.type_scope.try_import(
                                &dep_module_info,
                                local.sym.to_string(),
                                imported.map(|ident| ident.sym.to_string())
                            );
                        }

                        ImportSpecifier::Default(def) => {
                            return Err(BindGenError {
                                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultImport),
                                span: def.span,
                            });
                        }

                        ImportSpecifier::Namespace(namespace) => {
                            return Err(BindGenError {
                                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultImport),
                                span: namespace.span,
                            });
                        }
                    }
                }

                Ok(())
            }

            ModuleDecl::ExportDecl(ExportDecl {
                span,
                decl,
            }) => {
                let decl_item = BindGenSession::process_decl(context, module_info, decl, span)?;
                let item_kind = decl_item.item_kind();
                let (name, typ) = decl_item.into_data();

                match item_kind {
                    ItemKind::Value => {
                        module_info.export_value(name.clone(), typ.clone());
                        context.value_scope.insert(name, Value::Rooted(typ));
                    }

                    ItemKind::Type => {
                        module_info.export_type(name.clone(), typ.clone());
                        context.type_scope.insert(name, typ);
                    }
                    ItemKind::ValueType => {
                        module_info.export_value(name.clone(), typ.clone());
                        module_info.export_type(name.clone(), typ.clone());
                        context.value_scope.insert(name.clone(), Value::Rooted(typ.clone()));
                        context.type_scope.insert(name, typ);
                    }
                }

                Ok(())
            }

            ModuleDecl::ExportNamed(NamedExport {
                src,
                span,
                specifiers,
            }) => {

                let mut exporter: Box<FnMut(String, Option<String>) -> ()> = match src {

                    // Open the source module and re-export an exported item
                    Some(src) => {

                        let dep_module_info = self.bind_module_from_src(context, &src, Some(span))?;

                        Box::new(move |original_key: String, as_key: Option<String>| -> () {
                            module_info.merge_export(
                                        &dep_module_info,
                                        original_key,
                                        as_key);
                        })
                    }

                    // Export an item from the current module
                    None => {
                        Box::new(|original_key: String, as_key: Option<String>| -> () {

                            let as_key = as_key.unwrap_or(original_key.clone());

                            let value_type = context.value_scope
                                .get(&original_key)
                                .map(|v| v.clone());
                            let type_item = context.type_scope
                                .get(&original_key)
                                .map(|t| t.clone());

                            if value_type.is_none() && type_item.is_none() {
                                panic!("Invalid export. No item named {} in scope", &original_key);
                            }

                            if let Some(value_type) = value_type {
                                module_info.export_value(as_key.clone(), value_type);
                            }

                            if let Some(type_item) = type_item {
                                module_info.export_type(as_key, type_item);
                            }
                        })
                    }
                };

                for specifier in specifiers.into_iter() {
                    match specifier {
                        ExportSpecifier::Named(NamedExportSpecifier {
                            orig,
                            exported: exported_as,
                            ..
                        }) => {

                            let orig_key = orig.sym.to_string();
                            let exported_key = exported_as
                                .map(|x| x.sym.to_string());

                            exporter(orig_key, exported_key);
                        },

                        ExportSpecifier::Namespace(NamespaceExportSpecifier {
                            span,
                            ..
                        }) => {
                            return Err(BindGenError {
                                kind: BindGenErrorKind::UnsupportedFeature(
                                          UnsupportedFeature::NamespaceExport),
                                span,
                            });
                        }

                        ExportSpecifier::Default(..) => {
                            return Err(BindGenError {
                                kind: BindGenErrorKind::UnsupportedFeature(
                                          UnsupportedFeature::DefaultExport),
                                span,
                            });
                        }
                    }
                }

                Ok(())
            }

            ModuleDecl::ExportAll(ExportAll {
                src,
                span,
                ..
            }) => {
                let dep_module_info = self.bind_module_from_src(context, &src, Some(span))?;

                // Take all exports and merge into the current module
                module_info.merge_all(&*dep_module_info);

                Ok(())
            }

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
}
