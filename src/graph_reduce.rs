use std::collections::{HashSet, HashMap};

use swc_atoms::JsWord;

use super::bind_graph_init::{
    ModuleGraph,
    ModuleNode,
    Import,
    Export,
};
use super::error::*;
use super::structures::CanonPath;

/// Modify graph such that import/export edges point directly towards the rooted item
///
/// POST-CONDITION:
///   All Import::Named edges transformed into Import::NamedType and/or Import::NamedValue
///   All Export::Named edges transformed into Export::NamedType and/or Export::NamedValue
///   All new edges point directly to a rooted value
pub fn reduce(graph: ModuleGraph) -> Result<ModuleGraph, BindGenError> {
    let mut session = ResolutionSession {
        nodes: &graph.nodes,
        original_exports: &graph.export_edges,
        original_imports: &graph.import_edges,
        new_exports: Vec::new(),
        new_imports: Vec::new(),

    };

    session.resolve_imports()?;
    session.resolve_exports()?;

    let export_edges = session.new_exports
        .into_iter()
        .map(|(p, edges)| (p.clone(), edges))
        .collect();

    let import_edges = session.new_imports
        .into_iter()
        .map(|(p, edges)| (p.clone(), edges))
        .collect();

    // TODO: Remove Export::All edges
    //    and unify strongly connected components export interfaces

    Ok(ModuleGraph {
        nodes: graph.nodes,
        export_edges,
        import_edges,
    })
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

    fn get_node(&self, path: &CanonPath) -> &'a ModuleNode {
        self.nodes
            .get(path)
            .expect(&format!("Missing module for {}", path.as_path().display()))
    }

    /// Remove extraneous export edges and connects re-exports directly to values
    /// Does NOT remove Export::All edges
    fn resolve_exports(&mut self) -> Result<(), BindGenError> {
        for (canon_path, exports) in self.original_exports.iter() {
            let mut new_exports: Vec<Export> = Vec::new();

            for export in exports.iter() {
                match export {

                    Export::NamedType {
                        ref source,
                        ref src_key,
                        ref export_key,
                    } => {
                        let resolution =
                            self.traverse(source, src_key, ResolutionKind::Type);

                        match resolution {
                            Some((path, key)) => {
                                new_exports.push(Export::NamedType {
                                    source: path,
                                    src_key: key,
                                    export_key: export_key.clone(),
                                });

                            }

                            None => todo!("Error: type import not resolved"),
                        }

                    }

                    Export::NamedValue {
                        ref source,
                        ref src_key,
                        ref export_key,
                    } => {
                        let resolution =
                            self.traverse(source, src_key, ResolutionKind::Value);

                        match resolution {
                            Some((path, key)) => {
                                new_exports.push(Export::NamedValue {
                                    source: path,
                                    src_key: key,
                                    export_key: export_key.clone(),
                                });

                            }

                            None => todo!("Error: value import not resolved"),
                        }
                    }

                    Export::Named {
                        ref source,
                        ref src_key,
                        ref export_key,
                    } => {

                        let type_resolution =
                            self.traverse(source, src_key, ResolutionKind::Type);

                        let value_resolution =
                            self.traverse(source, src_key, ResolutionKind::Value);

                        if type_resolution.is_none() && value_resolution.is_none() {
                            todo!("Error: import not resolved");
                        }

                        if let Some((path, key)) = type_resolution {
                                new_exports.push(Export::NamedType {
                                    source: path,
                                    src_key: key,
                                    export_key: export_key.clone(),
                                });
                        }

                        if let Some((path, key)) = value_resolution {
                                new_exports.push(Export::NamedValue {
                                    source: path,
                                    src_key: key,
                                    export_key: export_key.clone(),
                                });
                        }

                    }

                    Export::All {
                        ref source,
                    } => {
                        new_exports.push(Export::All {
                            source: source.clone(),
                        });
                    }
                }
            }

            self.new_exports.push((canon_path, new_exports));
        }

        Ok(())
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

    ///
    /// Scans re-export edges and adds to the worklist if matching export keys
    ///
    fn worklist_exports(&self,
        worklist: &mut Vec<(&'a CanonPath, &'a JsWord)>,
        export_source: &'a CanonPath,
        key: &'a JsWord,
        kind: ResolutionKind
    ) {
        let exports = self.original_exports
            .get(export_source)
            .expect(&format!("Missing original exports from {}", export_source.as_path().display()));

        for export in exports.iter() {
            match export {
                Export::NamedType {
                    ref source,
                    ref src_key,
                    ref export_key,
                } => {

                    if let ResolutionKind::Type = kind {
                        if export_key == key {
                            worklist.push((source, src_key));
                        }
                    }

                },

                Export::NamedValue {
                    ref source,
                    ref src_key,
                    ref export_key,
                } => {

                    if let ResolutionKind::Value = kind {
                        if export_key == key {
                            worklist.push((source, src_key));
                        }
                    }

                }

                Export::Named {
                    ref source,
                    ref src_key,
                    ref export_key,
                } => {
                    if export_key == key {
                        worklist.push((source, src_key));
                    }
                }

                Export::All {
                    ref source,
                } => {
                    worklist.push((source, key));
                }
            }
        }
    }

    fn traverse(&self,
        start: &'a CanonPath,
        source_key: &'a JsWord,
        kind: ResolutionKind,
    ) -> Resolution {

        let mut visited_set: HashSet<&CanonPath> = HashSet::new();
        let mut worklist: Vec<(&CanonPath, &JsWord)> = vec![(start, source_key)];

        while worklist.is_empty() == false {
            let (next_path, next_key) = worklist.pop().unwrap();

            if visited_set.contains(next_path) {
                continue;
            }

            visited_set.insert(next_path);

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

            self.worklist_exports(&mut worklist, next_path, next_key, kind);
        }

        None
    }
}
