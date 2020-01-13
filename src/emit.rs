use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::io::Write;

use serde_json::{json, Value};

use super::structures::*;
use super::emit_structures::*;
use super::error::EmitError;

#[derive(PartialEq, Eq, Hash)]
struct TypeKey {
    name: String,
    js_origin: String,
}

pub fn emit_json(outdir: &Path, root_module_info: &ModuleInfo) -> Result<(), EmitError> {

    let file_name = root_module_info
        .path()
        .file_stem()
        .expect("Root module info path has no filename");
    let json_output_path = {
        let mut output_path = outdir.to_owned();
        output_path.push(file_name);
        output_path.set_extension("arr.json");

        output_path
    };

    let mut emitted_types: HashSet<TypeKey> = HashSet::new();

    let mut output = JsonOutput::new();

    for (export_key, typ) in root_module_info.exported_types() {
        output.export_type(export_key, typ);
    }

    for (export_key, typ) in root_module_info.exported_values() {
        output.export_value(export_key, typ);
    }

    // TODO: Emit values

    // Emit JSON into file
    let mut file =
        File::create(json_output_path)
        .map_err(|io_err| EmitError::IoError(root_module_info.path().to_owned(), io_err))?;

    let output = output
        .finalize()
        .map_err(|json_err| EmitError::JsonError(root_module_info.path().to_owned(), json_err))?;

    file.write_all(output.as_bytes());

    Ok(())
}
