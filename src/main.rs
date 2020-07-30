#[macro_use]
extern crate derive_builder;

#[macro_use]
mod macros;
mod generate;
mod ts;
mod compile_opt;
mod common;

use std::error::Error;
use std::path::PathBuf;

use clap::{Arg, App};

use ts::TsFlavor;

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

    arg.map(|s| {
        if common::TS_NUM_STRINGS.contains(&s) {
            Ok(TsFlavor::TsNum)
        } else if common::TS_FULL_STRINGS.contains(&s) {
            Ok(TsFlavor::TsFull)
        } else {
            Err(format!("Unknown TS flavor \"{}\"", s))
        }
    }).unwrap_or(Ok(common::DEFAULT_TS_FLAVOR.0))
}

fn main() {

    let matches = {
        let mut app = App::new("jetsam")
            .arg(Arg::with_name("INPUT")
                .short("i")
                .long("input")
                .value_name("root TS module")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("OUTPUT")
                .short("o")
                .long("output")
                .value_name("output directory")
                .takes_value(true)
                .required(true)
                .validator(output_directory_validator))
            .arg(Arg::with_name("REQUIRE PATH")
                .long("require-path")
                .value_name("require path")
                .takes_value(true)
                .help("Import path of the TS implementation file relative to the generated bindings file [default: Same directory as the generated bindings file]")
                .required(false))
            .arg(Arg::with_name("OUTPUT FILE STEM")
                .long("output-file-stem")
                .takes_value(true)
                .required(false))
            .arg(Arg::with_name(common::OPTION_TS_FLAVOR)
                .long(common::OPTION_TS_FLAVOR)
                .short("tsf")
                .value_name("TS flavor")
                .possible_values(common::TS_FLAVOR_STRINGS)
                .default_value(common::DEFAULT_TS_FLAVOR.1)
                .takes_value(true)
                .help("TypeScript subset to accept as input")
                .required(false))
            .arg(Arg::with_name(common::OPTIONS_BASE_CONFIG)
                .long(common::OPTIONS_BASE_CONFIG)
                .value_name("base config path")
                .takes_value(true)
                .help("Path to base configuration path")
                .required(false));

        opt_arg!(app =>
            key: common::OPTION_CONSTRUCTOR_WRAPPERS;
            long: common::OPTION_CONSTRUCTOR_WRAPPERS;
            values: bool_values!();
            validator: bool_validator;
            help:
            "Generate Pyret functions around class constructors";
            help-long:
"Generate Pyret functions around class constructors.
Used by:
    * TS-FULL

[default: true]
"
        );

        opt_arg!(app =>
            key: common::OPTION_OPAQUE_INTERFACES;
            long: common::OPTION_OPAQUE_INTERFACES;
            values: bool_values!();
            validator: bool_validator;
            help:
            "Generate 1:1 opaque nominal datatypes for Pyret interfaces";
            help-long:
"Generate 1:1 opaque nominal datatypes for Pyret interfaces
Used by:
    * TS-FULL
[default: true]
"
        );

        opt_arg!(app =>
            key: common::OPTION_WRAP_TOP_LEVEL_VARS;
            long: common::OPTION_WRAP_TOP_LEVEL_VARS;
            values: bool_values!();
            validator: bool_validator;
            help:
            "Generate converter getters around exported top-level variables";
            help-long:
"Generate converter getters around exported top-level variables
Used by:
    * TS-FULL
    * TS-NUM
[default: true]
"
        );

        app
    }.get_matches();

    let target_ts_flavor = match construct_ts_flavor(matches.value_of(common::OPTION_TS_FLAVOR)) {
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


    let mut gen_config = match matches.value_of(common::OPTIONS_BASE_CONFIG)
        .map(load_config)
        .unwrap_or(Ok(generate::GenConfig::default())) {

        Ok(b) => b,

        Err(e) => {
            eprintln!("{}", e);
            eprintln!("Unable to open the base config file");
            std::process::exit(1);
        }
    };

    let _ = extract_opt_arg!(matches =>
        key: common::OPTION_CONSTRUCTOR_WRAPPERS;
        converter: str::parse::<bool>;
        =>
        gen_config: &mut gen_config;
        gen_key: output_constructor_wrappers
    );

    let _ = extract_opt_arg!(matches =>
        key: common::OPTION_OPAQUE_INTERFACES;
        converter: str::parse::<bool>;
        =>
        gen_config: &mut gen_config;
        gen_key: output_opaque_interfaces
    );

    let _ = extract_opt_arg!(matches =>
        key: common::OPTION_WRAP_TOP_LEVEL_VARS;
        converter: str::parse::<bool>;
        =>
        gen_config: &mut gen_config;
        gen_key: wrap_top_level_vars
    );

    let output_dir = PathBuf::from(output_dir);
    let input_path = PathBuf::from(input_path);

    let emit_config = generate::EmitConfig {
        json: true,
        js: true,
    };

    let require_path = require_path
        .map(|p| p.to_string())
        .unwrap_or({
            let mut buff = PathBuf::new();
            buff.push("./");
            buff.push(input_path.file_stem().unwrap());
            buff.set_extension("js");

            buff.display().to_string()
        });

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

fn load_config(path: &str) -> Result<generate::GenConfig, Box<dyn Error>> {
    use std::io::BufReader;
    use std::fs::File;
    use serde_json::de;

    let file = BufReader::new(File::open(path)?);

    let gen_config: generate::GenConfig = de::from_reader(file)?;

    Ok(gen_config)
}
