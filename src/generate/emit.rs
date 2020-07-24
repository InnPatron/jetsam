mod ts_full;
mod js_emit_full;
mod json_emit_full;

use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::io::Write;

use crate::generate::structures::*;
use crate::generate::error::EmitError;
use crate::generate::typify_graph::ModuleGraph;
use crate::generate::config::EmitConfig;
use crate::generate::type_structs::Type;
use crate::compile_opt::CompileOpt;

pub trait JsonEmitter {

    fn export_type(&mut self, name: &str, typ: &Type);
    fn export_value(&mut self, name: &str, value_type: &Type);
    fn finalize(self, current_module: &Path) -> Result<String, EmitError>;
}

pub trait JsEmitter {
    fn handle_type(&mut self, name: &str, typ: &Type);
    fn handle_value(&mut self, name: &str, value_type: &Type);
    fn finalize(self, current_module: &Path, default_require_path: String) -> String;
}

struct Context<JS: JsEmitter, JSON: JsonEmitter> {
    json_output: JSON,
    js_output: JS,
}

pub fn ts_full_emit(
    options: &CompileOpt,
    root_module_path: &CanonPath,
    typed_graph: &ModuleGraph,
) -> Result<(), EmitError> {
    use self::js_emit_full::TsFullJsOutput as JsEmitter;
    use self::json_emit_full::TsFullJsonOutput as JsonEmitter;

    let js_emitter = JsEmitter::new(options);
    let json_emitter = JsonEmitter::new(options);

    emit(options, root_module_path, typed_graph, js_emitter, json_emitter)
}

pub fn emit<JS: JsEmitter, JSON: JsonEmitter>(
    options: &CompileOpt,
    root_module_path: &CanonPath,
    typed_graph: &ModuleGraph,
    js_emitter: JS,
    json_emitter: JSON,
) -> Result<(), EmitError> {

    let outdir = &options.output_dir;

    let file_name = options.file_stem
        .as_ref()
        .map(|f| std::ffi::OsStr::new(f))
        .unwrap_or_else(|| {
            root_module_path
                .as_path()
                .file_stem()
                .expect("Root module info path has no filename")
        });

    let mut context = Context {
        json_output: json_emitter,
        js_output: js_emitter,
    };

    traverse(
        options,
        root_module_path,
        typed_graph,
        &mut context,
    );

    opt!(options.emit_config, json, {

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
            .finalize(root_path.as_path())?;

        file.write_all(output.as_bytes())
            .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;

    });

    opt!(options.emit_config, js, {

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
            .finalize(root_path, default_require_path);

        file.write_all(output.as_bytes())
            .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;

    });

    Ok(())
}

fn traverse<JS: JsEmitter, JSON: JsonEmitter>(
    options: &CompileOpt,
    root: &CanonPath,
    graph: &ModuleGraph,
    context: &mut Context<JS, JSON>,
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

        opt!(options.emit_config, json, {
            for (export_key, typ) in node.rooted_export_types.iter() {
                context.json_output.export_type(export_key, typ);
            }

            for (export_key, typ) in node.rooted_export_values.iter() {
                context.json_output.export_value(export_key, typ);
            }
        });


        opt!(options.emit_config, js, {
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
