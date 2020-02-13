use std::collections::{HashSet, HashMap};

use indexmap::IndexSet;

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
///   All Export::All edges are transformed into a set of Export::NamedType and/or
///      Export::NamedValue
///   All new edges point directly to a rooted value
pub fn reduce(mut graph: ModuleGraph) -> Result<ModuleGraph, BindGenError> {


    let scc_session = SccSession::init(&graph.nodes, &graph.export_edges);
    let (module_scc_map, scc_map) = scc_session.get_sccs();

    let expansion_session = ExpansionSession {
        nodes: &graph.nodes,
        original_exports: &graph.export_edges,
        scc_map,
        module_scc_map,
    };

    // Expand Export::All edges
    // Does NOT remove them b/c may be needed during resolution
    let expanded: HashMap<CanonPath, Vec<Export>> =
        expansion_session.expand_exports();

    //use std::convert::TryFrom;
    //dbg!(
    //    expanded.get(
    //    &CanonPath::try_from(std::path::PathBuf::from("/home/q/Documents/three.js/src/materials/Materials.d.ts")).unwrap()));

    // Add expanded edges to graph
    for (path, mut expanded) in expanded.into_iter() {
        graph.export_edges.get_mut(&path).unwrap().append(&mut expanded);
    }

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
        .map(|(p, edges)| {
            let edges = edges.into_iter().filter(|e| match e {
                Export::All { .. } => false,
                _ => true,
            }).collect();

            (p.clone(), edges)
        })
        .collect();

    let import_edges = session.new_imports
        .into_iter()
        .map(|(p, edges)| (p.clone(), edges))
        .collect();

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

                            None => todo!("Error in [{}]: type import not resolved [{}]:{} (as {})",
                                canon_path.as_path().display(), source.as_path().display(), src_key, export_key),
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

                            None => todo!("Error in [{}]: value import not resolved [{}]:{} (as {})",
                                canon_path.as_path().display(), source.as_path().display(), src_key, export_key),
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
                            todo!("Error in [{}]: import not resolved [{}]:{} (as {})",
                                canon_path.as_path().display(), source.as_path().display(), src_key, export_key);
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

#[derive(Clone)]
struct ExportSet {
    types: HashSet<JsWord>,
    values: HashSet<JsWord>,
    nebulous: HashSet<JsWord>,
}

impl ExportSet {
    fn new() -> Self {
        ExportSet {
            types: HashSet::new(),
            values: HashSet::new(),
            nebulous: HashSet::new(),
        }
    }

    fn union_add(&mut self, other: &ExportSet) {
        for t in other.types.iter() {
            self.types.insert(t.clone());
        }

        for v in other.values.iter() {
            self.values.insert(v.clone());
        }

        for n in other.nebulous.iter() {
            self.nebulous.insert(n.clone());
        }
    }

    fn difference(&self, other: &ExportSet) -> ExportSet {
        ExportSet {
            types: self.types.difference(&other.types).cloned().collect(),
            values: self.values.difference(&other.values).cloned().collect(),
            nebulous: self.nebulous.difference(&other.nebulous).cloned().collect(),
        }
    }
}

struct ExpansionSession<'a> {
    nodes: &'a HashMap<CanonPath, ModuleNode>,
    original_exports: &'a HashMap<CanonPath, Vec<Export>>,
    scc_map: HashMap<SccId, Scc<'a>>,
    module_scc_map: HashMap<&'a CanonPath, SccId>,
}

impl<'a> ExpansionSession<'a> {
    fn node_direct_export_set(&self, path: &CanonPath) -> ExportSet {
        let mut set = ExportSet::new();

        let node = self.nodes.get(path).unwrap();
        for v in node.rooted_export_values.iter() {
            set.values.insert(v.clone());
        }

        for t in node.rooted_export_types.iter() {
            set.types.insert(t.clone());
        }

        let edges = self.original_exports.get(path).unwrap();

        for edge in edges.iter() {
            match edge {
                Export::NamedType {
                    ref export_key,
                    ..
                } => {
                    set.types.insert(export_key.clone());
                }

                Export::NamedValue {
                    ref export_key,
                    ..
                } => {
                    set.values.insert(export_key.clone());
                }

                Export::Named {
                    ref export_key,
                    ..
                } => {
                    set.nebulous.insert(export_key.clone());
                }

                Export::All { .. } => (),
            }
        }

        set
    }

    fn scc_direct_export_set(&self,
        scc: &Scc<'a>,
        node_sets: &mut HashMap<&'a CanonPath, ExportSet>,
    ) -> ExportSet {
        let mut scc_set = ExportSet::new();

        for node_path in scc.set.iter() {
            let node_set = self.node_direct_export_set(node_path);
            scc_set.union_add(&node_set);
            node_sets.insert(node_path, node_set);
        }

        scc_set
    }

    fn scc_export_set(&self,
        scc: &Scc<'a>,
        scc_sets: &mut HashMap<SccId, ExportSet>,
        node_sets: &mut HashMap<&'a CanonPath, ExportSet>
    ) -> SccId {
        match scc_sets.get(&scc.id) {
            Some(..) => scc.id,

            None => {
                let current_id = scc.id;
                let mut scc_export_set = self.scc_direct_export_set(scc, node_sets);

                for scc_out_edge in scc.outgoing_edges.iter() {
                    if scc.set.contains(scc_out_edge) == false {
                        //dbg!(scc_out_edge);
                        //dbg!(&self.module_scc_map);
                        let dep_scc_id = self.module_scc_map.get(scc_out_edge).unwrap();

                        if *dep_scc_id == current_id {
                            continue;
                        }

                        let dep_scc: &Scc = self.scc_map.get(dep_scc_id).unwrap();
                        let dep_export_set = {
                            self.scc_export_set(dep_scc, scc_sets, node_sets);
                            scc_sets.get(dep_scc_id).unwrap()
                        };

                        scc_export_set.union_add(dep_export_set);
                    }
                }

                scc_sets.insert(current_id, scc_export_set);

                current_id
            }
        }
    }

    fn expand_exports(mut self) -> HashMap<CanonPath, Vec<Export>> {

        let mut expanded_exports = HashMap::new();
        let mut scc_sets = HashMap::new();
        let mut node_sets = HashMap::new();

        // For each SCC, expand the Export::All edges with respect to SCC export set
        for (id, scc) in self.scc_map.iter() {
            self.scc_export_set(&scc, &mut scc_sets, &mut node_sets);
            let scc_set = scc_sets.get(id).expect("Missing scc set");

            let scc_root: CanonPath
                = (*scc.set.iter().next().unwrap()).clone();

            // For each node, export missing types, values, or nebulous edges
            for node_path in scc.set.iter() {
                let mut expanded = Vec::new();

                let node_set = node_sets.get(node_path).unwrap();
                let difference = scc_set.difference(node_set);

                for export_key in difference.types.into_iter() {
                    expanded.push(Export::NamedType {
                        source: scc_root.clone(),
                        src_key: export_key.clone(),
                        export_key,
                    });
                }

                for export_key in difference.values.into_iter() {
                    expanded.push(Export::NamedValue {
                        source: scc_root.clone(),
                        src_key: export_key.clone(),
                        export_key,
                    });
                }

                for export_key in difference.nebulous.into_iter() {
                    expanded.push(Export::Named {
                        source: scc_root.clone(),
                        src_key: export_key.clone(),
                        export_key,
                    });
                }

                // Return cloned CanonPath's necessary to mutate original graph
                expanded_exports.insert((*node_path).clone(), expanded);
            }
        }

        expanded_exports
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct SccId(u64);

struct Scc<'a> {
    id: SccId,
    set: IndexSet<&'a CanonPath>,
    outgoing_edges: HashSet<&'a CanonPath>,
}

impl<'a> Scc<'a> {
    fn is_sink(&self) -> bool {
        self.outgoing_edges.len() == 0
    }
}

/// Tarjan's strongly connected components algorithm
/// https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm#Complexity
struct SccSession<'a> {

    module_scc_map: HashMap<&'a CanonPath, SccId>,
    results: HashMap<SccId, Scc<'a>>,

    current: Option<&'a CanonPath>,
    work_stack: Vec<&'a CanonPath>,
    curr_index: usize,
    scc_index: u64,

    vertex_indices: HashMap<&'a CanonPath, usize>,
    vertex_low_links: HashMap<&'a CanonPath, usize>,
    vertex_on_stack: HashSet<&'a CanonPath>,

    nodes: &'a HashMap<CanonPath, ModuleNode>,
    original_exports: &'a HashMap<CanonPath, Vec<Export>>,
}

impl<'a> SccSession<'a> {

    fn init(
        nodes: &'a HashMap<CanonPath, ModuleNode>,
        original_exports: &'a HashMap<CanonPath, Vec<Export>>,
    ) -> Self {
        let session = SccSession {

            module_scc_map: HashMap::new(),
            results: HashMap::new(),

            current: None,
            work_stack: Vec::new(),
            curr_index: 0,
            scc_index: 0,

            vertex_indices: HashMap::new(),
            vertex_low_links: HashMap::new(),
            vertex_on_stack: HashSet::new(),

            nodes,
            original_exports,
        };

        session
    }

    fn get_sccs(mut self) -> (HashMap<&'a CanonPath, SccId>, HashMap<SccId, Scc<'a>>) {
        self.export_alls_scc();

        for (_, scc) in self.results.iter_mut() {
            for scc_member in scc.set.iter() {
                for edge in self.original_exports.get(scc_member).unwrap().iter() {
                    let edge_endpoint = edge.export_source();

                    if scc.set.contains(edge_endpoint) == false {
                        scc.outgoing_edges.insert(edge_endpoint);
                    }
                }
            }
        }

        (self.module_scc_map, self.results)
    }

    fn export_alls_scc(&mut self) {

        for (node_path, _) in self.nodes.iter() {
            if self.vertex_indices.contains_key(node_path) == false {
                self.current = Some(node_path);
                self.scc();
            }
        }
    }

    fn scc(&mut self) {
        let current_path = self.current.unwrap();
        let edges = self.original_exports
            .get(current_path)
            .map(|edges| {
                edges.iter()
                    .filter(|edge| match edge {
                        Export::All { .. } => true,

                        _ => false,
                    })
                    .map(|export_all| match export_all {
                        Export::All { ref source } => source,

                        _ => unreachable!("Should be filtered"),

                    })
            }).unwrap();

        self.vertex_indices.insert(current_path, self.curr_index);
        self.vertex_low_links.insert(current_path, self.curr_index);

        self.curr_index += 1;

        self.work_stack.push(current_path);
        self.vertex_on_stack.insert(current_path);

        //  w
        for to in edges {

            if self.vertex_indices.contains_key(to) == false {

                self.current = Some(to);
                self.scc();
                let low_link = {
                    let v_ll = self.vertex_low_links.get(current_path)
                        .unwrap();

                    let w_ll = self.vertex_low_links.get(to)
                        .unwrap();

                    std::cmp::min(v_ll, w_ll)
                };
                self.vertex_low_links.insert(current_path, *low_link);

            } else if self.vertex_on_stack.contains(to) {

                let low_link = {
                    let v_ll = self.vertex_low_links.get(current_path)
                        .unwrap();

                    let w_index = self.vertex_indices.get(to)
                        .unwrap();

                    std::cmp::min(v_ll, w_index)
                };
                self.vertex_low_links.insert(current_path, *low_link);

            }
        }

        let v_ll = self.vertex_low_links.get(current_path)
            .unwrap();

        let v_index = self.vertex_indices.get(current_path)
            .unwrap();

        if v_ll == v_index {
            let scc_id = SccId(self.scc_index);
            let mut scc = Scc {
                id: scc_id,
                set: IndexSet::new(),
                outgoing_edges: HashSet::new(),
            };

            let work_stack = {
                let mut tmp = Vec::new();
                std::mem::swap(&mut tmp, &mut self.work_stack);
                tmp
            };

            for path in work_stack.into_iter() {
                if path != current_path {
                    self.module_scc_map.insert(path, scc_id);
                    self.vertex_on_stack.remove(path);
                    scc.set.insert(path);
                }
            }
            self.module_scc_map.insert(current_path, scc_id);
            scc.set.insert(current_path);
            self.results.insert(scc_id, scc);

            self.scc_index += 1;
        }
    }
}
