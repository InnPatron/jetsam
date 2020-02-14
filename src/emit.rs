use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::io::Write;

use super::structures::*;
use super::json_emit::*;
use super::js_emit::*;
use super::error::EmitError;
use super::typify_graph::ModuleGraph;

#[derive(Clone)]
pub struct EmitOptions {
    pub json: bool,
    pub js: bool,
    pub js_include_path: Option<String>,
}

macro_rules! opt {
    ($options: expr, $opt:ident, $body: block) => {
        if $options.$opt {
            $body
        }
    }
}

struct Context {
    json_output: JsonOutput,
    js_output: JsOutput,
}

pub fn emit(
    options: EmitOptions,
    outdir: &Path,
    root_module_path: &CanonPath,
    typed_graph: &ModuleGraph
) -> Result<(), EmitError> {

    let file_name = root_module_path
        .as_path()
        .file_stem()
        .expect("Root module info path has no filename");

    let mut context = Context {
        json_output: JsonOutput::new(),
        js_output: JsOutput::new(),
    };

    traverse(
        &options,
        root_module_path,
        typed_graph,
        &mut context,
    );

    opt!(options, json, {

        let json_output_path = {
            let mut output_path = outdir.to_owned();
            output_path.push(file_name);
            output_path.set_extension("arr.json");

            output_path
        };

        // Emit JSON into file
        let root_path = root_module_path.as_path().to_owned();
        let mut file =
            File::create(json_output_path)
            .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;

        let output = context.json_output
            .finalize()
            .map_err(|json_err| EmitError::JsonError(root_path.to_owned(), json_err))?;

        file.write_all(output.as_bytes())
            .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;

    });

    opt!(options, js, {

        let js_output_path = {
            let mut output_path = outdir.to_owned();
            output_path.push(file_name);
            output_path.set_extension("arr.js");

            output_path
        };

        // Emit JS into file
        let root_path = root_module_path.as_path();
        let default_require_path: String = {
            use std::path::PathBuf;
            let mut buff = PathBuf::new();
            buff.push("./");
            buff.push(root_path.file_stem().unwrap());
            buff.set_extension("js");

            buff.display().to_string()
        };
        let mut file =
            File::create(js_output_path)
            .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;

        let output = context.js_output
            .finalize(&options, default_require_path);

        file.write_all(output.as_bytes())
            .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;

    });

    Ok(())
}

fn traverse(
    options: &EmitOptions,
    root: &CanonPath,
    graph: &ModuleGraph,
    context: &mut Context,
) {
    let mut visited: HashSet<&CanonPath> = HashSet::new();

    let mut stack: Vec<&CanonPath> = vec![root];

    while stack.is_empty() == false {
        let node_path = stack.pop().unwrap();

        if visited.contains(node_path) {
            continue;
        }
        visited.insert(node_path);

        let node = graph.nodes.get(node_path).unwrap();

        opt!(options, json, {
            for (export_key, typ) in node.rooted_export_types.iter() {
                context.json_output.export_type(export_key, typ);
            }

            for (export_key, typ) in node.rooted_export_values.iter() {
                context.json_output.export_value(export_key, typ);
            }
        });


        opt!(options, js, {
            for (export_key, typ) in node.rooted_export_types.iter() {
                context.js_output.handle_type(export_key, typ);
            }

            for (export_key, typ) in node.rooted_export_values.iter() {
                context.js_output.handle_value(export_key, typ);
            }
        });

        let edges = graph.export_edges.get(node_path).unwrap();

        for edge in edges {
            stack.push(edge.export_source());
        }
    }
}
