use std::collections::{HashMap, HashSet};

use swc_ecma_ast::*;
use swc_common::Span;

use super::bind_init;
use super::structures::CanonPath;
use super::error::*;

pub fn init(cache: bind_init::ParsedModuleCache) -> Result<ModuleGraph, InitError> {
    let mut graph = ModuleGraph {
        nodes: HashMap::new(),
        export_edges: HashMap::new(),
        import_edges: HashMap::new(),
    };
    todo!("Init module graph");
}

pub struct InitError;

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
        export_key: String,
        module_key: String,
    },
}

pub enum Export {
    // Unused until TS 3.8
    NamedType {

    },
    Named {
        source: CanonPath,
        export_key: String,
        module_key: String,
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

struct NodeInitSession<'a> {
    path: &'a CanonPath,
    rooted_export_types: HashSet<String>,
    rooted_export_values: HashSet<String>,

    value_scope: HashSet<String>,
    type_scope: HashSet<String>,
}

impl<'a> NodeInitSession<'a> {

    fn init(g: &mut ModuleGraph, module_data: bind_init::ModuleData) -> Result<(), InitError> {
        let mut session = NodeInitSession {
            path: &module_data.path,
            rooted_export_types: HashSet::new(),
            rooted_export_values: HashSet::new(),

            value_scope: HashSet::new(),
            type_scope: HashSet::new(),
        };

        for item in module_data.module_ast.body.iter() {
            session.process_module_item(item)?;
        }

        todo!("Insert node and edges into graph");

        Ok(())
    }

    fn process_module_item(&mut self, item: &ModuleItem) -> Result<(), InitError> {
        todo!();
    }
}
