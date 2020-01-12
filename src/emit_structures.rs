use std::collections::HashMap;

use serde_json::{json, Map, Value};

use serde_json::error::Error as JsonError;

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

    pub fn export_opaque_type(&mut self, name: &str) {
        let opaque_type = json!(["data", name, [], [], []]);
        let local_type = local_type!(@V name);

        self.provides_aliases.insert(name.to_string(), local_type);
        self.provides_datatypes.insert(name.to_string(), opaque_type);
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
}


pub struct JsOutput;

impl JsOutput {

}
