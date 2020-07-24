use std::collections::HashMap;

use swc_ecma_ast::*;
use swc_atoms::JsWord;
use swc_common::Span;

use super::bind_common;
use super::bind_init::{ModuleData, ParsedModuleCache as ModuleCache};
use super::type_structs::*;
use super::structures::{
    Scope,
    CanonPath,
    ItemState,
    ItemStateT,
};
use super::error::*;
use super::bind_graph_init::{
    ModuleGraph as UTModuleGraph,
    Import,
    Export,
};
use super::type_construction as type_cons;

pub fn typify(cache: &ModuleCache, ut_graph: UTModuleGraph) -> Result<ModuleGraph, BindGenError> {
    let mut graph = ModuleGraph {
        nodes: HashMap::new(),
        export_edges: ut_graph.export_edges,
        import_edges: ut_graph.import_edges,
    };

    for (_, module_data) in cache.iter() {
        NodeInitSession::init(&mut graph, module_data)?;
    }

    Ok(graph)
}

#[derive(Debug)]
pub struct ModuleNode {
    pub path: CanonPath,
    pub rooted_export_types: HashMap<JsWord, Type>,
    pub rooted_export_values: HashMap<JsWord, Type>,
}

/// ORDER OF EXPORTS MATTERS
/// ORDER OF IMPORTS MATTERS
///
/// Ordered by occurence in the AST
pub struct ModuleGraph {
    pub nodes: HashMap<CanonPath, ModuleNode>,
    pub export_edges: HashMap<CanonPath, Vec<Export>>,
    pub import_edges: HashMap<CanonPath, Vec<Import>>,
}

impl ModuleGraph {
    fn module_instantiated(&self, p: &CanonPath) -> bool {
        self.nodes.contains_key(p)
    }
}

struct NodeInitSession<'a, 'b> {
    path: &'a CanonPath,
    dependency_map: &'a HashMap<String, CanonPath>,
    type_scope: &'b Scope<ItemState>,
    value_scope: Scope<ItemStateT>,

    rooted_values: HashMap<JsWord, Type>,
    rooted_types: HashMap<JsWord, Type>,
    generated_types: HashMap<JsWord, Type>,

}

macro_rules! get_dep_src {
    ($self: expr, $src_str: expr) => {
        $self.dependency_map.get(&*$src_str.value).expect("Source path not found in dependency_map")
    }

}

impl<'a, 'b> NodeInitSession<'a, 'b> {

    fn init(
        g: &mut ModuleGraph,
        module_data: &ModuleData
    ) -> Result<(), BindGenError> {

        let type_scope = super::init_type_scope::init(module_data)?;
        let mut session = NodeInitSession {
            path: &module_data.path,
            dependency_map: &module_data.dependencies,

            generated_types: HashMap::new(),
            rooted_values: HashMap::new(),
            rooted_types: HashMap::new(),

            value_scope: Scope::new(),
            type_scope: &type_scope,
        };

        for item in module_data.module_ast.body.iter() {
            session.process_module_item(item)?;
        }

        let rooted_export_types = session.rooted_types;
        let rooted_export_values = session.rooted_values;

        let module_node = ModuleNode {
            path: module_data.path.clone(),
            rooted_export_types,
            rooted_export_values,
        };

        g.nodes.insert(module_data.path.clone(), module_node);

        Ok(())
    }

    fn scope_value(&mut self, key: JsWord, state: ItemStateT) {
        self.value_scope.insert(key, state);
    }

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

            ModuleDecl::ExportNamed(ref exp) => self.process_named_export(exp),

            ModuleDecl::ExportAll(ExportAll {
                ref src,
                ..
            }) => {
                Ok(())
            }

            ModuleDecl::ExportDefaultDecl(..)
                => unreachable!("Caught in bind init"),

            ModuleDecl::ExportDefaultExpr(..)
                => unreachable!("Caught in bind init"),

            ModuleDecl::TsImportEquals(..)
                => unreachable!("Caught in bind init"),

            ModuleDecl::TsExportAssignment(..)
                => unreachable!("Caught in bind init"),

            ModuleDecl::TsNamespaceExport(..)
                => unreachable!("Caught in bind init"),
        }
    }

    fn prune_export_specifiers<'c, T>(&self, specifiers: T, exp_span: &Span)
        -> Result<Vec<&'c ExportSpecifier>, BindGenError>
            where T: Iterator<Item=&'c ExportSpecifier> {

        let mut buff = Vec::new();
        for spec in specifiers {
            match spec {
                ExportSpecifier::Named(..) => {
                    buff.push(spec);
                },

                ExportSpecifier::Namespace(ExportNamespaceSpecifier {
                    ref span,
                    ..
                }) => {
                    return Err(BindGenError {
                        module_path: self.path.as_path().to_owned(),
                        kind: BindGenErrorKind::UnsupportedFeature(
                                  UnsupportedFeature::NamespaceExport),
                        span: span.clone(),
                    });
                }

                ExportSpecifier::Default(..) => {
                    return Err(BindGenError {
                        module_path: self.path.as_path().to_owned(),
                        kind: BindGenErrorKind::UnsupportedFeature(
                                  UnsupportedFeature::DefaultExport),
                        span: exp_span.clone(),
                    });
                }
            }
        }

        Ok(buff)
    }

    fn process_named_export(&mut self, exp: &NamedExport) -> Result<(), BindGenError> {
        let specifiers = self.prune_export_specifiers(exp.specifiers.iter(), &exp.span)?;

        match exp.src {
            Some(ref src) => Ok(()),

            None => {
                for spec in specifiers {
                    match spec {
                        ExportSpecifier::Named(ExportNamedSpecifier {
                            ref orig,
                            exported: ref exported_as,
                            ..
                        }) => {

                            let orig_key = orig.sym.clone();
                            let export_key = exported_as
                                .as_ref()
                                .map(|x| x.sym.clone())
                                .unwrap_or(orig_key.clone());

                            // Handle the named export if it refers to a rooted item or imported
                            //   item by adding an edge if it is an imported item
                            //   or by marking the item as rooted (under its export key)


                            // Handle types
                            if let Some(ref state) = self.type_scope.get(&orig_key) {
                                if let ItemState::Rooted = state {
                                    let rooted_type = self.generated_types
                                        .get(&orig_key)
                                        .unwrap();
                                    self.rooted_values.insert(
                                        export_key.clone(),
                                        rooted_type.clone()
                                    );
                                }
                            }

                            // Handle values
                            if let Some(ref state) = self.value_scope.get(&orig_key) {
                                if let ItemStateT::Rooted(ref typ) = state {
                                    self.rooted_values.insert(export_key, typ.clone());
                                }
                            }
                        },

                        _ => unreachable!("Invalid specifier should be pruned"),
                    }
                }

                Ok(())
            }
        }
    }

    fn process_decl(&mut self, decl: &Decl, export: bool) -> Result<(), BindGenError> {


        match decl {
            Decl::Var(ref decl) => {
                let vars = type_cons::construct_variable_types(
                    self.path,
                    self.type_scope,
                    decl,
                )?;

                for (symbol, typ) in vars.into_iter() {
                    if export {
                        self.rooted_values.insert(symbol.clone(), typ.clone());
                    }

                    self.scope_value(symbol, ItemStateT::Rooted(typ));
                }
            }

            Decl::Fn(ref decl) => {
                let typ = type_cons::construct_fn_type(
                    self.path,
                    self.type_scope,
                    &decl.function,
                )?;

                let symbol = decl
                    .ident
                    .sym.clone();

                if export {
                    self.rooted_values.insert(symbol.clone(), typ.clone());
                }

                self.scope_value(symbol, ItemStateT::Rooted(typ));
            }

            decl @ Decl::Class(..) |
            decl @ Decl::TsInterface(..) |
            decl @ Decl::TsTypeAlias(..) |
            decl @ Decl::TsEnum(..) => {
                let typ = type_cons::construct_type(
                    self.path,
                    self.type_scope,
                    decl
                )?;
                let ident = bind_common::get_decl_ident(decl);

                self.generated_types.insert(ident.sym.clone(), typ.clone());

                if export {
                    self.rooted_types.insert(ident.sym.clone(), typ);
                }
            }

            _ => (),
        };

        Ok(())
    }

    fn handle_import_specifier(&mut self, source: &CanonPath, spec: &ImportSpecifier)
        -> Result<(), BindGenError> {
        match spec {
            ImportSpecifier::Named(ref named) => {

                let src_key = named
                    .imported
                    .as_ref()
                    .map(|export_key| export_key.sym.clone())
                    .unwrap_or(named.local.sym.clone());

                let as_key = named.local.sym.clone();

                let state = ItemStateT::Imported {
                    source: source.clone(),
                    src_key,
                    as_key,
                };

                let import_key = named.local.sym.clone();
                self.scope_value(import_key, state);

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
}
