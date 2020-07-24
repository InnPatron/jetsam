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
mod ts_flavor_detector;
mod ts_flavor_compat;

use std::sync::Arc;

use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_parser::Session;

pub use self::config::GenConfig;
pub use self::config::EmitConfig;

use crate::ts::TsFlavor;
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

        let detected_ts = ts_flavor_detector::detect(&typed_graph);

        let target_ts = options.ts_flavor.features();

        if let Err(e) = ts_flavor_compat::compatible(&detected_ts, &target_ts) {
            eprintln!("Compatibility errors:");
            for err in e {
                eprintln!("\t{:?}", err);
            }
            std::process::exit(1);
        }

        let result = match options.ts_flavor {
            TsFlavor::TsNum => {
                todo!();
            }

            TsFlavor::TsFull => emit::emit(&options, &cache.root, &typed_graph),

            TsFlavor::TsCustom(..) => todo!("TsCustom"),
        };

        match result {
            Ok(..) => (),

            Err(e) => {
                eprintln!("json-emit error: {:?}", e);
                std::process::exit(1);
            }
        }
    });
}
