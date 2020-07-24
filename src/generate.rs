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
mod config;

use std::sync::Arc;

use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_parser::Session;

pub use self::config::GenConfig;
pub use self::config::EmitConfig;

use crate::compile_opt;

pub fn gen(options: compile_opt::CompileOpt) {
    swc_common::GLOBALS.set(&swc_common::Globals::new(), move || {
        let cm: Arc<SourceMap> = Default::default();
        let handler =
            Handler::with_tty_emitter(ColorConfig::Auto, true, false,
Some(cm.clone()));

        let session = Session {
            handler: &handler,
        };

        let cache = match bind_init::init(cm.clone(), session, options.input_path.clone()) {
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

        match emit::emit(&options, &cache.root, &typed_graph) {
            Ok(..) => (),

            Err(e) => {
                eprintln!("json-emit error: {:?}", e);
                std::process::exit(1);
            }
        }
    });
}
