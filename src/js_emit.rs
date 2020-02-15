use indexmap::IndexMap;

use super::type_structs::*;
use super::emit::EmitOptions;
use super::emit_common;

pub struct JsOutput<'a> {
    options: &'a EmitOptions,
    overrides: IndexMap<String, String>
}

impl<'a> JsOutput<'a> {
    pub fn new(options: &'a EmitOptions) -> Self {
        JsOutput {
            options,
            overrides: IndexMap::new(),
        }
    }

    pub fn handle_value(&mut self, name: &str, value_type: &Type) {
        // Do nothing for now
    }

    pub fn handle_type(&mut self, name: &str, typ: &Type) {
        match typ {
            Type::Class(ref class_type) => {
                opt!(self.options, output_constructor_wrappers, {

                    for (index, constructor) in class_type.constructors.iter().enumerate() {

                        let constructor_name =
                            emit_common::constuctor_name(index, &*class_type.name);

                        let string_constructor =
                            self.build_constructor(&*class_type.name, &constructor_name, constructor);

                        self.overrides.insert(constructor_name, string_constructor);
                    }
                });
            }

            _ => (),
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
        let body = format!("return new {}({});", class_name, list);

        format!("function {}({}) {{ {} }}", constructor_name, params, body)
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
            &format!("module.exports = root;\n\n")
        );

        for (override_key, override_value) in self.overrides.into_iter() {
            output.push_str(
                &format!("module.exports[\"{}\"] = {};\n", override_key, override_value)
            );
        }

        output
    }
}
