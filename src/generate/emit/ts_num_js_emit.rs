use std::path::Path;

use indexmap::IndexMap;

use crate::generate::type_structs::*;
use crate::generate::error::EmitError;
use crate::generate::emit_common;
use crate::compile_opt::CompileOpt;

use super::JsEmitter;

pub(super) struct TsNumJsOutput<'a> {
    options: &'a CompileOpt<'a>,
    overrides: IndexMap<String, String>
}

impl<'a> TsNumJsOutput<'a> {
    pub fn new(options: &'a CompileOpt<'a>) -> Self {
        TsNumJsOutput {
            options,
            overrides: IndexMap::new(),
        }
    }
}

impl<'a> JsEmitter for TsNumJsOutput<'a> {
    fn handle_value(&mut self, current_module: &Path, name: &str, value_type: &Type)
        -> Result<(), EmitError> {

        match value_type {
            Type::Number => todo!("Wrap getter around number vars"),
            Type::Fn(..) => todo!("Wrap around number functions"),

            _ => Err(EmitError::Misc(
                    current_module.to_owned(),
                    format!("TS-NUM does not support values of type: {:?}", value_type)
                )),
        }
    }

    fn handle_type(&mut self, _current_module: &Path, _name: &str, _typ: &Type)
        -> Result<(), EmitError> {

        Ok(())
    }

    fn finalize(self, current_module: &Path, default_require_path: String)
        -> Result<String, EmitError> {

        let mut output = String::new();

        let require_path = self.options.require_path
            .as_ref()
            .map(|p| p.clone())
            .unwrap_or(&default_require_path);

        output.push_str(
            &format!("const root = require(\"{}\");\n", require_path)
        );

        output.push_str(
            &format!("module.exports = root;\n\n")
        );

        for (override_key, override_value) in self.overrides.into_iter() {
            output.push_str(
                &format!("module.exports[\"{}\"] = {};\n", override_key, override_value)
            );
        }

        Ok(output)
    }
}
