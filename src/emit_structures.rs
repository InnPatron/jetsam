use std::collections::HashMap;

use serde_json::{Map, Value};

pub struct JsonOutput {
    provides_values: Map<String, Value>,
    provides_aliases: Map<String, Value>,
    provides_datatypes: Map<String, Value>,
}

impl JsonOutput {
}


pub struct JsOutput;

impl JsOutput {

}
