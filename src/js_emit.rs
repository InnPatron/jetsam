use std::collections::HashMap;

use super::type_structs::*;
use super::emit::EmitOptions;

pub struct JsOutput<'a> {
    options: &'a EmitOptions,
    overrides: HashMap<String, String>
}

impl<'a> JsOutput<'a> {
    pub fn new(options: &'a EmitOptions) -> Self {
        JsOutput {
            options,
            overrides: HashMap::new(),
        }
    }

    pub fn handle_value(&mut self, name: &str, value_type: &Type) {
        // Do nothing for now
    }

    pub fn handle_type(&mut self, name: &str, typ: &Type) {
        // Do nothing for now
    }

    pub fn finalize(self, default_require_path: String) -> String {
        let mut output = String::new();

        let require_path = self.options.require_path
            .as_ref()
            .map(|p| p.clone())
            .unwrap_or(default_require_path);

        output.push_str(
            &format!("const root = require(\"{}\");\n", require_path)
        );

        output.push_str(
            &format!("module.exports = root.exports;")
        );

        output
    }
}
