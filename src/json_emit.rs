use serde_json::{json, Map, Value};

use serde_json::error::Error as JsonError;

use super::type_structs::*;

macro_rules! local_type {
    ($name: expr) => {
        ["local", $name]
    };

    (@V $name: expr) => {
        json!(["local", $name])
    }
}

macro_rules! opaque_record {
    () => {
        json!(["record", {}])
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
    anon_counter: u64,
    provides_values: Map<String, Value>,
    provides_aliases: Map<String, Value>,
    provides_datatypes: Map<String, Value>,
}

impl JsonOutput {
    pub fn new() -> Self {
        JsonOutput {
            anon_counter: 0,
            provides_values: Map::new(),
            provides_aliases: Map::new(),
            provides_datatypes: Map::new(),
        }
    }

    fn anon_inc(&mut self) -> u64 {
        self.anon_counter += 1;

        self.anon_counter
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
        let actual_type = self.define_type(typ);

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

    fn define_type(&mut self, typ: &Type) -> Value {

        macro_rules! opaque_type {
            ($name: expr) => {
                json!(["data", $name, [], [], []]);
            }
        }

        match typ {
            Type::Fn {
                ..
            } => JsonOutput::in_place_type_to_value(typ),

            Type::Class(ClassType {
                ref name,
                ..
            }) => opaque_type!(name),

            Type::Interface {
                ref name,
                ..
            } => opaque_type!(name),

            Type::Literal {
                ..
            } => todo!("define type literal"),
            //opaque_type!(format!("anon{}", self.anon_inc())),

            Type::Named { .. } => todo!("Cannot define a named type"),

            Type::UnsizedArray(ref _e_type) => todo!("Cannot re-define the unsized array type"),

            Type::Array(ref _e_type, ref _size) => todo!("Cannot re-define the array type"),

            Type::Alias { .. } => todo!("Aliases are not in the datatypes section"),

            Type::Union => opaque_type!(format!("union{}", self.anon_inc())),

            Type::Opaque {
                ref name,
                ..
            } => opaque_type!(name),

            Type::Boolean   |
            Type::Number    |
            Type::String    |
            Type::Void      |
            Type::Object    |
            Type::Any       |
            Type::Never     |
            Type::Undefined |
            Type::Null => todo!("Cannot redefine primitives"),
        }
    }

    /// Generates the Value representing the Type embedded within another Type.
    /// Assumes types are already defined in the datatypes section.
    fn in_place_type_to_value(typ: &Type) -> Value {
        match typ {
            Type::Fn(FnType {
                ref params,
                ref return_type,
            })=> {
                let params: Vec<Value> = params
                    .iter()
                    .map(|t| JsonOutput::in_place_type_to_value(t))
                    .collect();

                let return_type =
                    JsonOutput::in_place_type_to_value(return_type);

                // [ "arrow", [params], return-type ]
                json!([
                    "arrow",
                    params,
                    return_type
                ])
            }

            Type::Class(ClassType {
                ref name,
                ..
            }) => local_type!(@V name),

            Type::Interface {
                ref name,
                ..
            } => local_type!(@V name),

            // TODO: How to handle a type alias?
            Type::Alias {
                ref name,
                ..
            } => local_type!(@V name),

            Type::Named {
                ref name,
                ..
            } => local_type!(@V name),

            Type::Opaque {
                ref name,
                ..
            } => local_type!(@V name),

            Type::Literal {
                ..
            } => opaque_record!(),

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

            Type::Array(ref e_type, ref _size) => {
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

            Type::Boolean => json!("Boolean"),

            Type::Number => json!("Number"),

            Type::String => json!("String"),

            Type::Void => json!("Nothing"),

            Type::Object => {
                todo!("Object primitive type");
            }

            Type::Any => json!("Any"),

            Type::Never => json!("tbot"),

            Type::Undefined => {
                todo!("Undefined primitive type");
            }

            Type::Null => {
                todo!();
            }

            // TODO: Union types default to Any
            Type::Union => json!("Any"),
        }
    }
}
