#[macro_use]
mod macros;

//mod ts_full_js_emit;
//mod ts_full_json_emit;

mod ts_num_js_emit;
mod ts_num_json_emit;


use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use swc_ecma_ast::Module as AstModule;

use crate::generate::js_pp::PrettyPrinter;
use crate::generate::structures::*;
use crate::generate::error::EmitError;
use crate::generate::typify_graph::ModuleGraph;
use crate::generate::config::EmitConfig;
use crate::generate::type_structs::Type;
use crate::compile_opt::CompileOpt;

pub trait JsonEmitter {
    fn export_type(&mut self, current_module: &Path, name: &str, typ: &Type) -> Result<(), EmitError>;
    fn export_value(&mut self, current_module: &Path, name: &str, value_type: &Type) -> Result<(), EmitError>;
    fn finalize(self, current_module: &Path) -> Result<String, EmitError>;
}

pub trait JsEmitter {
    fn handle_type(&mut self, current_module: &Path, name: &str, typ: &Type) -> Result<(), EmitError>;
    fn handle_value(&mut self, current_module: &Path, name: &str, value_type: &Type) -> Result<(), EmitError>;
    fn finalize(self, current_module: &Path, default_require_path: String) -> Result<AstModule, EmitError>;
}

struct Context<JS: JsEmitter, JSON: JsonEmitter> {
    json_output: JSON,
    js_output: JS,
}

pub fn ts_num_emit(
    options: &CompileOpt,
    root_module_path: &CanonPath,
    typed_graph: &ModuleGraph,
) -> Result<(), EmitError> {
    use self::ts_num_js_emit::TsNumJsOutput as JsEmitter;
    use self::ts_num_json_emit::TsNumJsonOutput as JsonEmitter;

    let js_emitter = JsEmitter::new(options);
    let json_emitter = JsonEmitter::new(options);

    emit(options, root_module_path, typed_graph, js_emitter, json_emitter)
}

pub fn ts_full_emit(
    options: &CompileOpt,
    root_module_path: &CanonPath,
    typed_graph: &ModuleGraph,
) -> Result<(), EmitError> {
    //use self::ts_full_js_emit::TsFullJsOutput as JsEmitter;
    //use self::ts_full_json_emit::TsFullJsonOutput as JsonEmitter;

    //let js_emitter = JsEmitter::new(options);
    //let json_emitter = JsonEmitter::new(options);

    //emit(options, root_module_path, typed_graph, js_emitter, json_emitter)
    todo!("TS-FULL")
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
    )?;

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
        let file =
            File::create(&js_output_path)
            .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;

        let file = BufWriter::new(file);

        let ast_module = context.js_output
            .finalize(root_path, default_require_path)?;

        // NOTE: Cannot use swc_ecma_codegen for whatever reason
        //   Provided emitter appears to rely on SourceMap and Spans
        let _ = PrettyPrinter::print(file, &ast_module)
            .map_err(|io_err| EmitError::IoError(root_path.to_owned(), io_err))?;
    });

    Ok(())
}

fn traverse<JS: JsEmitter, JSON: JsonEmitter>(
    options: &CompileOpt,
    root: &CanonPath,
    graph: &ModuleGraph,
    context: &mut Context<JS, JSON>,
) -> Result<(), EmitError> {
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
            opt!(options.emit_config, json, {
                context.json_output.export_type(node.path.as_path(), export_key, typ)?;
            });
            opt!(options.emit_config, js, {
                context.js_output.handle_type(node.path.as_path(), export_key, typ)?;
            });
        }

        for (export_key, typ) in node.rooted_export_values.iter() {
            opt!(options.emit_config, json, {
                context.json_output.export_value(node.path.as_path(), export_key, typ)?;
            });


            opt!(options.emit_config, js, {
                context.js_output.handle_value(node.path.as_path(), export_key, typ)?;
            });
        }


        let edges = graph.export_edges.get(node_path).unwrap();

        for edge in edges {
            stack.push(edge.export_source());
        }
    }

    Ok(())
}
