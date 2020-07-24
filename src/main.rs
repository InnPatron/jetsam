#[macro_use]
extern crate derive_builder;

mod generate;
mod ts;
mod compile_opt;

use std::path::PathBuf;

use clap::{Arg, App};

use ts::TsFlavor;

const DEFAULT_OUTPUT_CONSTRUCTOR_WRAPPERS: &'static str = "true";
const DEFAULT_OUTPUT_OPAQUE_INTERFACES: &'static str = "true";

fn output_directory_validator(arg: String) -> Result<(), String> {
    if PathBuf::from(arg).is_dir() {
        Ok(())
    } else {
        Err("Expected output argument to be a directory".to_string())
    }
}

fn bool_validator(arg: String) -> Result<(), String> {
    arg.parse::<bool>().map_err(|_| "Expected bool".to_string()).map(|_| ())
}

fn construct_ts_flavor(arg: Option<&str>) -> Result<TsFlavor, String> {

    arg.map(|s| match s {
        "ts-num" | "tsnum" | "NUM" => Ok(TsFlavor::ts_num()),

        "all" | "any" | "full" => Ok(TsFlavor::all()),

        _ => Err(format!("Unknown TS flavor \"{}\"", s)),

    }).unwrap_or(Ok(TsFlavor::all()))
}

fn main() {

    let matches = App::new("jetsam")
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
        .arg(Arg::with_name("TARGET TS FLAVOR")
            .long("ts-flavor")
            .short("tsf")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("OUTPUT CONTRUCTOR WRAPPERS")
            .long("constructor-wrappers")
            .takes_value(true)
            .required(false)
            .default_value(DEFAULT_OUTPUT_CONSTRUCTOR_WRAPPERS)
            .validator(bool_validator))
        .arg(Arg::with_name("OUTPUT OPAQUE INTERFACES")
            .long("opaque-interfaces")
            .takes_value(true)
            .required(false)
            .default_value(DEFAULT_OUTPUT_OPAQUE_INTERFACES)
            .validator(bool_validator))
        .get_matches();

    let target_ts_flavor = match construct_ts_flavor(matches.value_of("TARGET TS FLAVOR")) {
        Ok(ts_flavor) => ts_flavor,

        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let input_path =
        matches.value_of("INPUT").expect("No input root module");

    let output_dir =
        matches.value_of("OUTPUT").expect("No output directory");

    let require_path =
        matches.value_of("REQUIRE PATH");

    let file_stem =
        matches.value_of("OUTPUT FILE STEM");

    let output_constructor_wrappers =
        matches.value_of("OUTPUT CONTRUCTOR WRAPPERS")
        .expect("No output constructor wrapper");
    let output_constructor_wrappers = output_constructor_wrappers.parse::<bool>()
        .expect("Failed validation");

    let output_opaque_interfaces =
        matches.value_of("OUTPUT OPAQUE INTERFACES")
        .expect("No output constructor wrapper");
    let output_opaque_interfaces = output_opaque_interfaces.parse::<bool>()
        .expect("Failed validation");

    let output_dir = PathBuf::from(output_dir);
    let input_path = PathBuf::from(input_path);

    let gen_config = generate::GenConfig {
        output_constructor_wrappers,
        output_opaque_interfaces,
    };

    let emit_config = generate::EmitConfig {
        json: true,
        js: true,
    };

    let options = compile_opt::CompileOpt {
        input_path,
        require_path,
        file_stem,
        output_dir,
        gen_config,
        emit_config,
        ts_flavor: target_ts_flavor,
    };
    generate::gen(options);
}
