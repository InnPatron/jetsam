use std::collections::{HashMap, HashSet};

use swc_ecma_ast::*;
use swc_atoms::JsWord;
use swc_common::Span;

use super::bind_init;
use super::structures::CanonPath;
use super::error::*;

pub fn init(cache: bind_init::ParsedModuleCache) -> Result<ModuleGraph, BindGenError> {
    let mut graph = ModuleGraph {
        nodes: HashMap::new(),
        export_edges: HashMap::new(),
        import_edges: HashMap::new(),
    };
    todo!("Init module graph");
}

pub struct ModuleNode {
    pub path: CanonPath,
    pub ast: Module,
    pub rooted_export_types: HashSet<String>,
    pub rooted_export_values: HashSet<String>,
}

pub enum Import {
    // Unused until TS 3.8
    NamedType {

    },
    Named {
        source: CanonPath,
        export_key: JsWord,
    },
}

pub enum Export {
    // Unused until TS 3.8
    NamedType {

    },
    Named {
        source: CanonPath,
        export_key: JsWord,
        module_key: JsWord,
    },
    All {
        source: CanonPath,
    },
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

enum ItemState {
    MaybeImported {
        source: CanonPath,
        item: String,
    },

    Rooted,
}

struct NodeInitSession<'a> {
    path: &'a CanonPath,
    dependency_map: &'a HashMap<String, CanonPath>,
    import_edges: Vec<Import>,
    export_edges: Vec<Export>,

    value_scope: HashMap<String, ItemState>,
    type_scope: HashMap<String, ItemState>,
}

macro_rules! get_dep_src {
    ($self: expr, $src_str: expr) => {
        $self.dependency_map.get(&*$src_str.value).expect("Source path not found in dependency_map")
    }

}

impl<'a> NodeInitSession<'a> {

    fn init(g: &mut ModuleGraph, module_data: bind_init::ModuleData) -> Result<(), BindGenError> {
        let mut session = NodeInitSession {
            path: &module_data.path,
            dependency_map: &module_data.dependencies,
            import_edges: Vec::new(),
            export_edges: Vec::new(),


            value_scope: HashMap::new(),
            type_scope: HashMap::new(),
        };

        for item in module_data.module_ast.body.iter() {
            session.process_module_item(item)?;
        }

        todo!("Insert node and edges into graph");

        Ok(())
    }

    fn process_module_item(&mut self, item: &ModuleItem) -> Result<(), BindGenError> {
        match item {

            ModuleItem::ModuleDecl(ref decl) => self.process_module_decl(decl),

            ModuleItem::Stmt(ref stmt) => self.process_stmt(stmt),
        }
    }

    fn process_stmt(&mut self, stmt: &Stmt) -> Result<(), BindGenError> {
        if let Stmt::Decl(ref decl) = stmt {
            todo!("Handle decl statement");
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


            x => todo!("Unhandled {:?}", x),
        }
    }

    fn handle_import_specifier(&mut self, source: &CanonPath, spec: &ImportSpecifier)
        -> Result<(), BindGenError> {
        match spec {
            ImportSpecifier::Specific(ref specific) => {

                let export_key = specific
                    .imported
                    .as_ref()
                    .map(|export_key| export_key.sym.clone())
                    .unwrap_or(specific.local.sym.clone());

                self.import_edges.push(Import::Named {
                    source: source.clone(),
                    export_key,
                });

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
