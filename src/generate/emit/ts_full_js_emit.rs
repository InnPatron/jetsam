use std::path::Path;

use indexmap::IndexMap;

use crate::generate::type_structs::*;
use crate::generate::error::EmitError;
use crate::generate::emit_common;
use crate::compile_opt::CompileOpt;

use super::JsEmitter;

pub(super) struct TsFullJsOutput<'a> {
    options: &'a CompileOpt<'a>,
    overrides: IndexMap<String, String>
}

impl<'a> TsFullJsOutput<'a> {
    pub fn new(options: &'a CompileOpt<'a>) -> Self {
        TsFullJsOutput {
            options,
            overrides: IndexMap::new(),
        }
    }

    fn build_constructor(&self,
        class_name: &str,
        constructor_name: &str,
        constructor: &FnType
    ) -> String {

        let list = {
            let mut params = String::new();
            for (index, _) in constructor.params.iter().enumerate() {
                params.push('p');
                params.push_str(&index.to_string());

                if index < constructor.params.len() - 1 {
                    params.push(',');
                }
            }

            params
        };

        let params = &list;
        let body = format!("return new root.{}({});", class_name, list);

        format!("function {}({}) {{ {} }}", constructor_name, params, body)
    }
}

impl<'a> JsEmitter for TsFullJsOutput<'a> {
    fn handle_value(&mut self, name: &str, value_type: &Type) -> Result<(), EmitError> {
        // Do nothing for now
        Ok(())
    }

    fn handle_type(&mut self, name: &str, typ: &Type) -> Result<(), EmitError> {
        match typ {
            Type::Class(ref class_type) => {
                opt!(self.options.gen_config, output_constructor_wrappers, {

                    for (index, constructor) in class_type.constructors.iter().enumerate() {

                        let constructor_name =
                            emit_common::constuctor_name(index, &*class_type.name);

                        let string_constructor =
                            self.build_constructor(&*class_type.name, &constructor_name, constructor);

                        self.overrides.insert(constructor_name, string_constructor);
                    }
                });

                Ok(())
            }

            _ => Ok(()),
        }
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
