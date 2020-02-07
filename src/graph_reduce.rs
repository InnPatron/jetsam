use std::collections::HashMap;

use swc_atoms::JsWord;

use super::bind_init::{ModuleData, ParsedModuleCache as ModuleCache};
use super::bind_graph_init::{
    ModuleGraph,
    ModuleNode,
    Import,
    Export,
    ScopeKind,
};
use super::error::*;
use super::structures::CanonPath;

/// Modify graph such that import/export edges point directly towards the rooted value
pub fn reduce(cache: &ModuleCache, graph: ModuleGraph) -> Result<ModuleGraph, BindGenError> {
    let mut session = ResolutionSession {
        nodes: &graph.nodes,
        original_exports: &graph.export_edges,
        original_imports: &graph.import_edges,
        new_exports: Vec::new(),
        new_imports: Vec::new(),

    };

    session.resolve_imports()?;

    todo!();
}

type Resolution = Option<(CanonPath, JsWord)>;

#[derive(Clone, Copy)]
enum ResolutionKind {
    Value,
    Type,
}

struct ResolutionSession<'a> {
    nodes: &'a HashMap<CanonPath, ModuleNode>,
    original_imports: &'a HashMap<CanonPath, Vec<Import>>,
    original_exports: &'a HashMap<CanonPath, Vec<Export>>,
    new_imports: Vec<(&'a CanonPath, Vec<Import>)>,
    new_exports: Vec<(&'a CanonPath, Vec<Export>)>,
}

impl<'a> ResolutionSession<'a> {

    fn get_node(&self, path: &CanonPath) -> &ModuleNode {
        self.nodes
            .get(path)
            .expect(&format!("Missing module for {}", path.as_path().display()))
    }

    fn resolve_imports(&mut self) -> Result<(), BindGenError> {
        for (canon_path, imports) in self.original_imports.iter() {
            let mut new_imports: Vec<Import> = Vec::new();

            for import in imports.iter() {
                match import {

                    Import::NamedType {
                        ref source,
                        ref src_key,
                    } => {
                        let resolution =
                            self.traverse(source, src_key, ResolutionKind::Type);

                        match resolution {
                            Some((path, key)) => {
                                new_imports.push(Import::NamedType {
                                    source: path,
                                    src_key: key
                                });

                            }

                            None => todo!("Error: type import not resolved"),
                        }

                    }

                    Import::NamedValue {
                        ref source,
                        ref src_key,
                    } => {
                        let resolution =
                            self.traverse(source, src_key, ResolutionKind::Value);

                        match resolution {
                            Some((path, key)) => {
                                new_imports.push(Import::NamedValue {
                                    source: path,
                                    src_key: key
                                });

                            }

                            None => todo!("Error: value import not resolved"),
                        }
                    }

                    Import::Named {
                        ref source,
                        ref src_key,
                    } => {

                        let type_resolution =
                            self.traverse(source, src_key, ResolutionKind::Type);

                        let value_resolution =
                            self.traverse(source, src_key, ResolutionKind::Value);

                        if type_resolution.is_none() && value_resolution.is_none() {
                            todo!("Error: import not resolved");
                        }

                        if let Some((path, key)) = type_resolution {
                                new_imports.push(Import::NamedType {
                                    source: path,
                                    src_key: key
                                });
                        }

                        if let Some((path, key)) = value_resolution {
                                new_imports.push(Import::NamedValue {
                                    source: path,
                                    src_key: key
                                });
                        }

                    }
                }
            }

            self.new_imports.push((canon_path, new_imports));
        }

        Ok(())
    }

    fn traverse(&self,
        start: &CanonPath,
        source_key: &JsWord,
        kind: ResolutionKind,
    ) -> Resolution {

        let mut worklist: Vec<(&CanonPath, &JsWord)> = vec![(start, source_key)];

        while worklist.is_empty() == false {
            let (next_path, next_key) = worklist.pop().unwrap();
            let node = self.get_node(next_path);
            match kind {
                ResolutionKind::Type => {
                    if node.is_rooted_type(next_key) {
                        return Some((next_path.clone(), next_key.clone()));
                    }
                },

                ResolutionKind::Value => {
                    if node.is_rooted_value(next_key) {
                        return Some((next_path.clone(), next_key.clone()));
                    }
                }
            }
        }


        None
    }
}
