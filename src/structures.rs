use std::collections::HashMap;
use std::path::PathBuf;

use swc_common::Span;
use swc_ecma_ast::Str;

pub struct TypeId(pub u64);

pub struct ModuleInfo {
    path: PathBuf,
    // TODO: Use TypeId
    exports: HashMap<String, OwnedItem>,
    // TODO:
    private_types: Vec<(TypeId, TypeAst)>,
}

impl ModuleInfo {
    pub fn new(path: PathBuf) -> Self {
        ModuleInfo {
            exports: HashMap::new(),
            private_types: Vec::new(),
            path,
        }
    }

    pub fn insert(&mut self, key: String, item: OwnedItem) {

        self.exports.insert(key, item);

        // The following passes tsc 3.7.4:
        //   export { foo } from "module"
        //   export * from "module"
        //   export * from "module"         <- this line is not an accident
        /*
        if self.exports.insert(key.clone(), item).is_some() {
            panic!("Duplicate exported key (\"{}\") in `{}`. Should not occur if passing static analysis.",
                key,
                self.path.display());
        }
        */
    }

    pub fn merge_item(&mut self, other: &Self, other_key: String, as_key: Option<String>) {
        let item = other.exports.get(&other_key)
            .expect(&format!("Missing exported key \"{}\" from `{}`", &other_key, other.path.display()))
            .clone();

        let insert_key = as_key.unwrap_or(other_key);
        self.insert(insert_key, item);
    }

    pub fn merge(&mut self, mut other: Self) {
        self.private_types.append(&mut other.private_types);
        for (exported_key, owned_item) in other.exports.into_iter() {

            self.exports.insert(exported_key, owned_item);

            /*
            if self.exports.insert(exported_key.clone(), owned_item).is_some() {
                panic!("Duplicate exported key (\"{}\") from `{}` conflicting with key in `{}`.\
                    Should not occur if passing static analysis.",
                    exported_key,
                    other.path.display(),
                    self.path.display(),
                );
            }
            */
        }
    }
}


/// Declared inside the current module
#[derive(Debug, Clone)]
pub enum OwnedItem {

    Class(TypeAst),

    Function {
        name: Str,
        type_signature: TypeAst,
    },

    Binding {
        name: Str,
        type_signature: Option<TypeAst>,
    },
}

/// Dependency source path of the **input project, NOT the generated code**
pub struct Dependency(pub Str);

#[derive(Debug, Clone)]
pub enum TypeAst {
    Fn {
        origin: Str,
        type_signaure: FnType,
    },
    Class{
        name: Str,
        origin: Str,
        constructor: Box<TypeAst>,
        fields: HashMap<String, TypeAst>,
    },
    Interface {
        origin: Str,
        fields: HashMap<String, TypeAst>,
    },
    Binding(BindingType),
    Array(Box<TypeAst>),
    Primitive(PrimitiveType),
}

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Boolean,
    Number,
    String,
    Void,
    Object,
    Any,
    Never,
}

/// Primitive types are not type-aliasable
#[derive(Debug, Clone)]
pub struct BindingType {
    name: Str,
    origin: Str,
}

#[derive(Debug, Clone)]
pub struct FnType {
    params: Vec<TypeAst>,
    return_type: Option<Box<TypeAst>>,
}
