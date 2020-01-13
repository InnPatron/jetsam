use std::collections::HashMap;

use serde_json::{json, Map, Value};

use serde_json::error::Error as JsonError;

use super::structures::{PrimitiveType, Type};

macro_rules! local_type {
    ($name: expr) => {
        ["local", $name]
    };

    (@V $name: expr) => {
        json!(["local", $name])
    }
}

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
pub struct JsonOutput {
    provides_values: Map<String, Value>,
    provides_aliases: Map<String, Value>,
    provides_datatypes: Map<String, Value>,
}

impl JsonOutput {
    pub fn new() -> Self {
        JsonOutput {
            provides_values: Map::new(),
            provides_aliases: Map::new(),
            provides_datatypes: Map::new(),
        }
    }

    pub fn export_value(&mut self, name: &str, value_type: &Type) {
        let value_type = JsonOutput::in_place_type_to_value(value_type);

        self.provides_values.insert(name.to_string(), value_type);
    }

    pub fn export_type(&mut self, name: &str, typ: &Type) {

        if let Type::Alias {
            ref name,
            ref aliasing_type,
        } = typ {

            self.provides_aliases.insert(name.to_string(), local_type!(@V name));
            return;
        }

        let local_type = local_type!(@V name);
        let actual_type = JsonOutput::define_type(typ);

        self.provides_aliases.insert(name.to_string(), local_type);
        self.provides_datatypes.insert(name.to_string(), actual_type);
    }

    pub fn finalize(self) -> Result<String, JsonError> {
        let map = json!({
            "requires": [],
            "provides": {
                "shorthands": { },
                "values": self.provides_values,
                "aliases": self.provides_aliases,
                "datatypes": self.provides_datatypes,
            }
        });

        serde_json::to_string_pretty(&map)
    }

    fn define_type(typ: &Type) -> Value {

        macro_rules! opaque_type {
            ($name: expr) => {
                json!(["data", $name, [], [], []]);
            }
        }

        match typ {
            Type::Fn {
                ..
            } => JsonOutput::in_place_type_to_value(typ),

            Type::Class {
                ref name,
                ref origin,
                ref constructor,
                ref fields,
            } => opaque_type!(name),

            Type::Interface {
                ref name,
                ref origin,
                ref fields,
            } => opaque_type!(name),

            Type::UnsizedArray(ref e_type) => todo!("Cannot re-define the unsized array type"),

            Type::Array(ref e_type, ref size) => todo!("Cannot re-define the array type"),

            Type::Primitive(..) => todo!("Cannot re-define a primitive type"),

            Type::Alias { .. } => todo!("Aliases are not in the datatypes section"),
        }
    }

    /// Generates the Value representing the Type embedded within another Type.
    /// Assumes types are already defined in the datatypes section.
    fn in_place_type_to_value(typ: &Type) -> Value {
        match typ {
            Type::Fn {
                ref origin,
                ref type_signature,
            } => {
                let params: Vec<Value> = type_signature.params
                    .iter()
                    .map(|t| JsonOutput::in_place_type_to_value(t))
                    .collect();

                // TODO: Default to Any or Nothing?
                let return_type = type_signature.return_type
                    .as_ref()
                    .map(|t| JsonOutput::in_place_type_to_value(t))
                    .unwrap_or(JsonOutput::in_place_type_to_value(&Type::Primitive(PrimitiveType::Any)));

                // [ "arrow", [params], return-type ]
                json!([
                    "arrow",
                    params,
                    return_type
                ])
            }

            Type::Class {
                ref name,
                ref origin,
                ref constructor,
                ref fields,
            } => local_type!(@V name),

            Type::Interface {
                ref name,
                ref origin,
                ref fields,
            } => local_type!(@V name),

            // TODO: How to handle a type alias?
            Type::Alias {
                ref name,
                ref aliasing_type,
            } => local_type!(@V name),


            Type::UnsizedArray(ref e_type) => {
                let e_type = JsonOutput::in_place_type_to_value(e_type);
                json!([
                    "tyapp",
                    {
                        "tag":"name",
                        "origin":
                        {
                            "import-type":"uri",
                            "uri":"builtin://global"
                        },
                        "name":"RawArray"
                    },
                    e_type
                ])
            }

            Type::Array(ref e_type, ref size) => {
                // TODO: Use size somehow
                let e_type = JsonOutput::in_place_type_to_value(e_type);
                json!([
                    "tyapp",
                    {
                        "tag":"name",
                        "origin":
                        {
                            "import-type":"uri",
                            "uri":"builtin://global"
                        },
                        "name":"RawArray"
                    },
                    e_type
                ])
            }

            Type::Primitive(PrimitiveType::Boolean) => json!("Boolean"),

            Type::Primitive(PrimitiveType::Number) => json!("Number"),

            Type::Primitive(PrimitiveType::String) => json!("String"),

            Type::Primitive(PrimitiveType::Void) => json!("Nothing"),

            Type::Primitive(PrimitiveType::Object) => {
                todo!("Object primitive type");
            }

            Type::Primitive(PrimitiveType::Any) => json!("Any"),

            Type::Primitive(PrimitiveType::Never) => json!("tbot"),

            Type::Primitive(PrimitiveType::Undefined) => {
                todo!("Undefined primitive type");
            }

            Type::Primitive(PrimitiveType::Null) => {
                todo!();
            }
        }
    }
}


pub struct JsOutput;

impl JsOutput {

}
