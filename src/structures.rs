use std::collections::HashMap;
use std::path::PathBuf;

use swc_common::Span;
use swc_ecma_ast::Str;

pub struct BindingModule {
    dependencies: Vec<Dependency>,
}

pub struct ModuleInfo {
    path: PathBuf,
    exports: HashMap<String, OwnedItem>,
}

impl ModuleInfo {
    pub fn new(path: PathBuf) -> Self {
        ModuleInfo {
            exports: HashMap::new(),
            path,
        }
    }

    pub fn insert(&mut self, key: String, item: OwnedItem) {
        if self.exports.insert(key.clone(), item).is_some() {
            panic!("Duplicate exported key (\"{}\") in `{}`. Should not occur if passing static analysis.",
                key,
                self.path.display());
        }
    }

    pub fn merge(&mut self, other: Self) {
        for (exported_key, owned_item) in other.exports.into_iter() {
            if self.exports.insert(exported_key.clone(), owned_item).is_some() {
                panic!("Duplicate exported key (\"{}\") from `{}` conflicting with key in `{}`.\
                    Should not occur if passing static analysis.",
                    exported_key,
                    other.path.display(),
                    self.path.display(),
                );
            }
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
