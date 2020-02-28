#[macro_use]
mod macros;
mod error;
mod structures;
mod type_structs;
mod init_type_scope;
mod type_construction;
mod bind_init;
mod bind_common;
mod bind_graph_init;
mod graph_reduce;
mod typify_graph;
mod emit;
mod json_emit;
mod js_emit;
mod emit_common;

use std::sync::Arc;
use std::path::PathBuf;

use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_parser::Session;

pub struct GenOptions<'a> {
    pub input_path: PathBuf,
    pub require_path: Option<&'a str>,
    pub file_stem: Option<&'a str>,
    pub output_constructor_wrappers: bool,
    pub output_opaque_interfaces: bool,
    pub output_dir: PathBuf,
}

pub fn gen(options: GenOptions) {
    swc_common::GLOBALS.set(&swc_common::Globals::new(), move || {
        let cm: Arc<SourceMap> = Default::default();
        let handler =
            Handler::with_tty_emitter(ColorConfig::Auto, true, false,
Some(cm.clone()));

        let session = Session {
            handler: &handler,
        };

        let cache = match bind_init::init(cm.clone(), session, options.input_path) {
            Ok(c) => c,

            Err(e) => {
                eprintln!("module cache error: {:?}", e);
                std::process::exit(1);
            }
        };

        let graph = match bind_graph_init::init(&cache) {
            Ok(g) => g,

            Err(e) => {
                eprintln!("graph init error: {:?}", e);
                std::process::exit(1);
            }
        };

        let graph = match graph_reduce::reduce(graph) {
            Ok(g) => g,

            Err(e) => {
                eprintln!("graph reduction error: {:?}", e);
                std::process::exit(1);
            }
        };

        let typed_graph = match typify_graph::typify(&cache, graph) {
            Ok(g) => g,

            Err(e) => {
                eprintln!("typify error: {:?}", e);
                std::process::exit(1);
            }
        };

        let emit_options = emit::EmitOptions {
            json: true,
            js: true,
            require_path: options.require_path.map(|input| input.to_string()),
            output_file_stem: options.file_stem.map(|f| f.to_string()),
            output_constructor_wrappers: options.output_constructor_wrappers,
            output_opaque_interfaces: options.output_opaque_interfaces,
        };

        match emit::emit(emit_options, &options.output_dir, &cache.root, &typed_graph) {
            Ok(..) => (),

            Err(e) => {
                eprintln!("json-emit error: {:?}", e);
                std::process::exit(1);
            }
        }
    });
}
