use std::collections::HashMap;

use super::type_structs::*;

pub struct JsEmit {
    overrides: HashMap<String, String>
}

impl JsEmit {
    pub fn new() -> Self {
        JsEmit {
            overrides: HashMap::new(),
        }
    }

    pub fn handle_value(&mut self, name: &str, value_type: &Type) {
        // Do nothing for now
    }

    pub fn handle_type(&mut self, name: &str, typ: &Type) {
        // Do nothing for now
    }
}
