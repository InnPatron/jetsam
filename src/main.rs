#[macro_use]
extern crate derive_builder;

#[macro_use]
mod macros;
mod common;
mod compile_opt;
mod generate;
mod ts;

use std::error::Error;
use std::path::PathBuf;

use clap::{App, Arg};

use ts::TsFlavor;

fn output_directory_validator(arg: String) -> Result<(), String> {
    if PathBuf::from(arg).is_dir() {
        Ok(())
    } else {
        Err("Expected output argument to be a directory".to_string())
    }
}

fn bool_validator(arg: String) -> Result<(), String> {
    arg.parse::<bool>()
        .map_err(|_| "Expected bool".to_string())
        .map(|_| ())
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
    })
    .unwrap_or(Ok(common::DEFAULT_TS_FLAVOR.0))
}

fn main() {
    let matches = {
        let mut app = App::new("jetsam")
            .arg(
                Arg::with_name("INPUT")
                    .short("i")
                    .long("input")
                    .value_name("root TS module")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("OUTPUT")
                    .short("o")
                    .long("output")
                    .value_name("output directory")
                    .takes_value(true)
                    .required(true)
                    .validator(output_directory_validator),
            )
            .arg(
                Arg::with_name("REQUIRE PATH")
                    .long("require-path")
                    .value_name("require path")
                    .takes_value(true)
                    .help(common::OPTION_REQUIRE_PATH_HELP)
                    .required(false),
            )
            .arg(
                Arg::with_name("OUTPUT FILE STEM")
                    .long("output-file-stem")
                    .takes_value(true)
                    .required(false),
            )
            .arg(
                Arg::with_name(common::OPTION_TS_FLAVOR)
                    .long(common::OPTION_TS_FLAVOR)
                    .short("tsf")
                    .value_name("TS flavor")
                    .possible_values(common::TS_FLAVOR_STRINGS)
                    .default_value(common::DEFAULT_TS_FLAVOR.1)
                    .takes_value(true)
                    .help(common::OPTION_TS_FLAVOR_HELP)
                    .required(false),
            )
            .arg(
                Arg::with_name(common::OPTIONS_GEN_CONFIG)
                    .long(common::OPTIONS_GEN_CONFIG)
                    .value_name("codegen config path")
                    .takes_value(true)
                    .help(common::OPTIONS_GEN_CONFIG_HELP)
                    .long_help(common::OPTIONS_GEN_CONFIG_HELP_LONG)
                    .required(false),
            );

        opt_arg!(app =>
            key: common::OPTION_CONSTRUCTOR_WRAPPERS;
            long: common::OPTION_CONSTRUCTOR_WRAPPERS;
            values: bool_values!();
            validator: bool_validator;
            help: common::OPTION_CONSTRUCTOR_WRAPPERS_HELP;
            help-long: common::OPTION_CONSTRUCTOR_WRAPPERS_HELP_LONG
        );

        opt_arg!(app =>
            key: common::OPTION_OPAQUE_INTERFACES;
            long: common::OPTION_OPAQUE_INTERFACES;
            values: bool_values!();
            validator: bool_validator;
            help: common::OPTION_OPAQUE_INTERFACES_HELP;
            help-long: common::OPTION_OPAQUE_INTERFACES_HELP_LONG
        );

        opt_arg!(app =>
            key: common::OPTION_WRAP_TOP_LEVEL_VARS;
            long: common::OPTION_WRAP_TOP_LEVEL_VARS;
            values: bool_values!();
            validator: bool_validator;
            help: common::OPTION_WRAP_TOP_LEVEL_VARS_HELP;
            help-long: common::OPTION_WRAP_TOP_LEVEL_VARS_HELP_LONG

        );

        app
    }
    .get_matches();

    let target_ts_flavor = match construct_ts_flavor(matches.value_of(common::OPTION_TS_FLAVOR)) {
        Ok(ts_flavor) => ts_flavor,

        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let input_path = matches.value_of("INPUT").expect("No input root module");

    let output_dir = matches.value_of("OUTPUT").expect("No output directory");

    let require_path = matches.value_of("REQUIRE PATH");

    let file_stem = matches.value_of("OUTPUT FILE STEM");

    let mut gen_config = match matches
        .value_of(common::OPTIONS_GEN_CONFIG)
        .map(load_config)
        .unwrap_or(Ok(generate::GenConfig::default()))
    {
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

    let require_path = require_path.map(|p| p.to_string()).unwrap_or({
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
    use serde_json::de;
    use std::fs::File;
    use std::io::BufReader;

    let file = BufReader::new(File::open(path)?);

    let gen_config: generate::GenConfig = de::from_reader(file)?;

    Ok(gen_config)
}
