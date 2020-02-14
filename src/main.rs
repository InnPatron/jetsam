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

use std::sync::Arc;
use std::path::PathBuf;

use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_parser::Session;

use clap::{Arg, App};

fn output_directory_validator(arg: String) -> Result<(), String> {
    if PathBuf::from(arg).is_dir() {
        Ok(())
    } else {
        Err("Expected output argument to be a directory".to_string())
    }
}

fn main() {

    let matches = App::new("plank")
        .arg(Arg::with_name("INPUT")
            .short("i")
            .long("input")
            .value_name("ROOT_MODULE")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("OUTPUT")
            .short("o")
            .long("output")
            .value_name("DIR_PATH")
            .takes_value(true)
            .required(true)
            .validator(output_directory_validator))
        .arg(Arg::with_name("REQUIRE PATH")
            .long("require-path")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("OUTPUT FILE STEM")
            .long("output-file-stem")
            .takes_value(true)
            .required(false))
        .get_matches();

    let input_path =
        matches.value_of("INPUT").expect("No input root module");

    let output_dir =
        matches.value_of("OUTPUT").expect("No output directory");

    let require_path =
        matches.value_of("REQUIRE PATH");

    let file_stem =
        matches.value_of("OUTPUT FILE STEM");

    let output_dir = PathBuf::from(output_dir);
    let input_path = PathBuf::from(input_path);

    swc_common::GLOBALS.set(&swc_common::Globals::new(), move || {
        let cm: Arc<SourceMap> = Default::default();
        let handler =
            Handler::with_tty_emitter(ColorConfig::Auto, true, false,
Some(cm.clone()));

        let session = Session {
            handler: &handler,
        };

        let cache = match bind_init::init(cm.clone(), session, input_path) {
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

        let options = emit::EmitOptions {
            json: true,
            js: true,
            require_path: require_path.map(|input| input.to_string()),
            output_file_stem: file_stem.map(|f| f.to_string()),
        };

        match emit::emit(options, &output_dir, &cache.root, &typed_graph) {
            Ok(..) => (),

            Err(e) => {
                eprintln!("json-emit error: {:?}", e);
                std::process::exit(1);
            }
        }
    });
}
