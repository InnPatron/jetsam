use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::io::Write;

use serde_json::{json, Value};

use super::structures::*;
use super::json_emit::*;
use super::error::EmitError;
use super::typify_graph::{ModuleGraph, ModuleNode};

#[derive(PartialEq, Eq, Hash)]
struct TypeKey {
    name: String,
    js_origin: String,
}

pub fn emit_json(outdir: &Path, root_module_path: &CanonPath, typed_graph: &ModuleGraph)
    -> Result<(), EmitError> {

    let file_name = root_module_path
        .as_path()
        .file_stem()
        .expect("Root module info path has no filename");
    let json_output_path = {
        let mut output_path = outdir.to_owned();
        output_path.push(file_name);
        output_path.set_extension("arr.json");

        output_path
    };

    let mut emitted_types: HashSet<TypeKey> = HashSet::new();

    let mut output = JsonOutput::new();

    traverse(root_module_path, typed_graph, &mut output);

    // Emit JSON into file
    let root_path = root_module_path.as_path().to_owned();
    let mut file =
        File::create(json_output_path)
        .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;

    let output = output
        .finalize()
        .map_err(|json_err| EmitError::JsonError(root_path.to_owned(), json_err))?;

    file.write_all(output.as_bytes())
        .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;

    Ok(())
}

fn traverse(root: &CanonPath, graph: &ModuleGraph, json_output: &mut JsonOutput) {
    let mut visited: HashSet<&CanonPath> = HashSet::new();

    let mut stack: Vec<&CanonPath> = vec![root];

    while stack.is_empty() == false {
        let node_path = stack.pop().unwrap();

        if visited.contains(node_path) {
            continue;
        }
        visited.insert(node_path);

        let node = graph.nodes.get(node_path).unwrap();

        for (export_key, typ) in node.rooted_export_types.iter() {
            json_output.export_type(export_key, typ);
        }

        for (export_key, typ) in node.rooted_export_values.iter() {
            json_output.export_value(export_key, typ);
        }

        let edges = graph.export_edges.get(node_path).unwrap();

        for edge in edges {
            stack.push(edge.export_source());
        }
    }
}
