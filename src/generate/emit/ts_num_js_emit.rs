use std::path::Path;

use indexmap::IndexMap;

use crate::generate::type_structs::*;
use crate::generate::error::EmitError;
use crate::generate::emit_common;
use crate::compile_opt::CompileOpt;

use super::JsEmitter;

const C_TS_NUMBER_PY_NUMBER: &'static str = "C_ts_number_py_number";
const C_PY_NUMBER_TS_NUMBER: &'static str = "C_py_number_ty_number";

macro_rules! root_value {
    ($i: expr) => {
        format!("root[\"{}\"]", $i)
    }
}

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

    fn c_ts_number_py_number(&self, binding: &str) -> String {
        format!("{}({})", C_TS_NUMBER_PY_NUMBER, binding)
    }

    fn c_py_number_ts_number(&self, binding: &str) -> String {
        format!("{}({})", C_PY_NUMBER_TS_NUMBER, binding)
    }

    fn c_ts_fn_py_fn(&self, fn_type: &FnType, binding: &str) -> String {
        let mut header = "function(".to_string();
        let mut body = "".to_string();
        let mut result = format!("let _result = {}(", binding);
        let result_id = "_result";

        for (index, param_type) in fn_type.params.iter().enumerate() {
            let param_id = format!("_{}", index);
            header.push_str(&param_id);
            header.push(',');

            result.push_str(&param_id);
            result.push(',');

            let converted = match param_type {
                Type::Fn(ref inner_fn_type) => self.c_py_fn_ts_fn(inner_fn_type, &param_id),

                Type::Number => self.c_py_number_ts_number(&param_id),

                ref t => unreachable!("c_ts_fn_py_fn: {} {:?} (param {})", binding, t, index),

            };

            body.push_str(&format!("{} = {};\n", param_id, converted));
        }
        header.push(')');

        result.push(')');
        result.push(';');

        let return_conversion = match *fn_type.return_type {
            Type::Fn(ref fn_type) => self.c_ts_fn_py_fn(fn_type, result_id),

            Type::Number => self.c_ts_number_py_number(result_id),

            ref t => unreachable!("c_ts_fn_py_fn: {} {:?} (return)", binding, t),
        };

        format!("{} {{\n{}{}\nreturn {};}}", header, body, result, return_conversion)
    }

    fn c_py_fn_ts_fn(&self, fn_type: &FnType, binding: &str) -> String {
        let mut header = "function(".to_string();
        let mut body = "".to_string();
        let mut result = format!("let _result = {}(", binding);
        let result_id = "_result";

        for (index, param_type) in fn_type.params.iter().enumerate() {
            let param_id = format!("_{}", index);
            header.push_str(&param_id);
            header.push(',');

            result.push_str(&param_id);
            result.push(',');

            let converted = match param_type {
                Type::Fn(ref inner_fn_type) => self.c_ts_fn_py_fn(inner_fn_type, &param_id),

                Type::Number => self.c_ts_number_py_number(&param_id),

                ref t => unreachable!("c_py_fn_ts_fn: {} {:?} (param {})", binding, t, index),

            };

            body.push_str(&format!("{} = {};\n", param_id, converted));
        }
        header.push(')');

        result.push(')');
        result.push(';');

        let return_conversion = match *fn_type.return_type {
            Type::Fn(ref fn_type) => self.c_py_fn_ts_fn(fn_type, result_id),

            Type::Number => self.c_py_number_ts_number(result_id),

            ref t => unreachable!("c_py_fn_ts_fn: {} {:?} (return)", binding, t),
        };

        format!("{} {{\n{}{}\nreturn {};}}", header, body, result, return_conversion)
    }
}

impl<'a> JsEmitter for TsNumJsOutput<'a> {
    fn handle_value(&mut self, current_module: &Path, name: &str, value_type: &Type)
        -> Result<(), EmitError> {

        match value_type {
            Type::Number => {
                let value = self.c_ts_number_py_number(&root_value!(name));
                let getter = format!("function() {{ return {}; }}", &value);

                self.overrides.insert(name.to_string(), getter);

                Ok(())
            }

            Type::Fn(ref fn_type) => {
                let value = self.c_ts_fn_py_fn(fn_type, &root_value!(name));
                self.overrides.insert(name.to_string(), value);
                Ok(())
            }

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
