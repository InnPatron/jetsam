use std::path::Path;

use serde_json::{json, Map, Value};
use serde_json::error::Error as JsonError;

use crate::compile_opt::CompileOpt;
use crate::generate::emit_common;
use crate::generate::error::EmitError;
use crate::generate::type_structs::*;

use super::JsonEmitter;

macro_rules! local_type {
    ($name: expr) => {
        ["local", $name]
    };

    (@V $name: expr) => {
        json!(["local", $name])
    }
}

/// ``` text
/// datatype formats:
///     [ "data", name, [type-param-names], [variants], [methods] ]
///     [ "arrow", [params], return-type ]
///     [ "forall", [type-param-names], poly-type ]
///     [ "tyapp", type, [type-args] ]
///
///     type:
///         [ "local", name ]
///         [ "tid", name ]
///         {
///           "tag": "name",
///           "origin": {
///             "import-type": "uri"
///             "uri": path
///           }
///           "name": name,
///         }
///
///
/// aliases section:
/// "aliases": {
///     "local-type-name": ["local", "local-type-name"]
/// }
/// ```
pub(super) struct TsNumJsonOutput<'a> {
    provides_values: Map<String, Value>,
    provides_aliases: Map<String, Value>,
    provides_datatypes: Map<String, Value>,
    options: &'a CompileOpt<'a>,
}

impl<'a> TsNumJsonOutput<'a> {
    pub fn new(options: &'a CompileOpt<'a>) -> Self {
        TsNumJsonOutput {
            options,
            provides_values: Map::new(),
            provides_aliases: Map::new(),
            provides_datatypes: Map::new(),
        }
    }

    fn define_type(&mut self, typ: &Type) -> Result<Value, String> {

        macro_rules! opaque_type {
            ($name: expr) => {
                json!(["data", $name, [], [], {}]);
            }
        }

        match typ {
            Type::Fn {
                ..
            } => TsNumJsonOutput::in_place_type_to_value(typ),

            _ => todo!("Can only define function types"),
        }
    }

    /// Generates the Value representing the Type embedded within another Type.
    /// Assumes types are already defined in the datatypes section.
    fn in_place_type_to_value(typ: &Type) -> Result<Value, String> {
        match typ {
            Type::Fn(FnType {
                ref params,
                ref return_type,
            })=> {
                let params = params
                    .iter()
                    .map(|t| TsNumJsonOutput::in_place_type_to_value(t))
                    .collect::<Result<Vec<_>, _>>()?;

                let return_type =
                    TsNumJsonOutput::in_place_type_to_value(return_type)?;

                // [ "arrow", [params], return-type ]
                Ok(json!([
                    "arrow",
                    params,
                    return_type
                ]))
            }

            Type::Number => Ok(json!("Number")),

            t => Err(format!("TS-NUM does not support type: {:?}", t)),
        }
    }
}

impl<'a> JsonEmitter for TsNumJsonOutput<'a> {

    fn export_value(&mut self, current_module: &Path, name: &str, value_type: &Type)
        -> Result<(), EmitError> {
        let value_type = TsNumJsonOutput::in_place_type_to_value(value_type)
            .map_err(|e| EmitError::Misc(current_module.to_owned(), e))?;

        self.provides_values.insert(name.to_string(), value_type);

        Ok(())
    }

    fn export_type(&mut self, current_module: &Path, name: &str, typ: &Type)
        -> Result<(), EmitError> {

        let local_type = local_type!(@V name);
        let actual_type = self.define_type(typ)
            .map_err(|e| EmitError::Misc(current_module.to_owned(), e))?;

        self.provides_aliases.insert(name.to_string(), local_type);
        self.provides_datatypes.insert(name.to_string(), actual_type);

        Ok(())
    }

    fn finalize(self, current_module: &Path) -> Result<String, EmitError> {
        let map = json!({
            "requires": [],
            "provides": {
                "shorthands": { },
                "values": self.provides_values,
                "aliases": self.provides_aliases,
                "datatypes": self.provides_datatypes,
            }
        });

        serde_json::to_string_pretty(&map).map_err(|e| EmitError::JsonError(current_module.to_owned(), e))
    }
}
