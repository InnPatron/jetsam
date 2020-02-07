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

    todo!();
}

struct ResolutionSession<'a> {
    nodes: &'a HashMap<CanonPath, ModuleNode>,
    original_imports: &'a HashMap<CanonPath, Vec<Import>>,
    original_exports: &'a HashMap<CanonPath, Vec<Export>>,
    new_imports: Vec<(&'a CanonPath, Vec<Import>)>,
    new_exports: Vec<(&'a CanonPath, Vec<Export>)>,
}

impl<'a> ResolutionSession<'a> {

    fn resolve_imports(&mut self) -> Result<(), BindGenError> {
        for (canon_path, imports) in self.original_imports.iter() {
            let mut imports = Vec::new();

            for import in imports.iter() {
                imports.push(todo!());
            }

            self.new_imports.push((canon_path, imports));
        }
        todo!();
    }

    fn resolve_type(&self) -> Option<(CanonPath, JsWord)> {

    }
}
