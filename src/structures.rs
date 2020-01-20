use std::hash::Hash;
use std::collections::HashMap;
use std::path::PathBuf;

use swc_common::Span;
use swc_ecma_ast::Str;

pub struct ModuleInfo {
    path: PathBuf,
    export_all: Vec<CanonPath>,
    exported_types: HashMap<String, Type>,
    exported_values: HashMap<String, Type>,
    dependencies: HashMap<String, CanonPath>,
}

impl ModuleInfo {
    pub fn new(path: PathBuf, dependencies: HashMap<String, CanonPath>) -> Self {
        ModuleInfo {
            exported_types: HashMap::new(),
            exported_values: HashMap::new(),
            export_all: Vec::new(),
            dependencies,
            path,
        }
    }

    pub fn get_dep_canon_path(&self, src: &str) -> CanonPath {
        self.dependencies.get(src).unwrap().clone()
    }

    pub fn path(&self) -> &std::path::Path {
        self.path.as_path()
    }

    pub fn export_all_modue(&mut self, module: CanonPath) {
        self.export_all.push(module);
    }

    pub fn exported_types(&self) -> impl Iterator<Item=(&str, &Type)> {
        self.exported_types
            .iter()
            .map(|(s, t)| (s.as_str(), t))
    }

    pub fn exported_values(&self) -> impl Iterator<Item=(&str, &Type)> {
        self.exported_values
            .iter()
            .map(|(s, t)| (s.as_str(), t))
    }

    pub fn get_exported_value(&self, key: &str) -> Option<&Type> {
        self.exported_values.get(key)
    }

    pub fn get_exported_type(&self, key: &str) -> Option<&Type> {
        self.exported_types.get(key)
    }

    pub fn export_value(&mut self, export_key: String, typ: Type) {
        self.exported_values.insert(export_key, typ);
    }

    pub fn export_type(&mut self, export_key: String, typ: Type) {
        self.exported_types.insert(export_key, typ);
    }

    pub fn merge_export(&mut self, other: &Self, other_key: String, as_key: Option<String>) {

        let exp_value_type: Option<Type> = other.exported_values
            .get(&other_key)
            .map(|id| id.clone());

        let exp_type: Option<Type> = other.exported_types
            .get(&other_key)
            .map(|id| id.clone());

        if exp_value_type.is_none() && exp_type.is_none() {
            panic!("Unknown export key {}", &other_key);
        }

        if let Some(exp_value_type) = exp_value_type {
            self.export_value(other_key.clone(), exp_value_type.clone());
        }

        if let Some(exp_type) = exp_type {
            self.export_type(other_key, exp_type);
        }
    }

    pub fn merge_all(&mut self, other: &Self) {

        // Merge exports
        for (export_key, typ) in other.exported_types.iter() {
            self.exported_types.insert(export_key.clone(), typ.clone());
        }

        for (export_key, value) in other.exported_values.iter() {
            self.exported_values.insert(export_key.clone(), value.clone());
        }
    }
}

pub enum ItemKind {
    Value,
    Type,
    ValueType,
}

pub enum Item {
    Class {
        name: String,
        typ: Type,
    },
    Fn {
        name: String,
        typ: Type,
    },
    Var{
        name: String,
        typ: Type,
    },
    TsInterface{
        name: String,
        typ: Type,
    },
    TsTypeAlias{
        name: String,
        typ: Type,
    },
    TsEnum{
        name: String,
        typ: Type,
    },
    TsModule {
        name: String,
        typ: Type,
    },
}

impl Item {

    pub fn item_kind(&self) -> ItemKind {
        match self {
            Item::Class { .. } => ItemKind::ValueType,
            Item::Fn { .. } => ItemKind::Value,
            Item::Var { .. } => ItemKind::Value,
            Item::TsInterface { .. } => ItemKind::Type,
            Item::TsTypeAlias { .. } => ItemKind::Type,
            Item::TsEnum { .. } => ItemKind::Type,      // TODO: TsEnum is ValueType?
            Item::TsModule { .. } => todo!("Item TsModule?"),
        }
    }

    pub fn into_data(self) -> (String, Type) {

        match self {
            Item::Class { name, typ } => (name, typ),
            Item::Fn { name, typ } => (name, typ),
            Item::Var { name, typ } => (name, typ),
            Item::TsInterface { name, typ } => (name, typ),
            Item::TsTypeAlias { name, typ } => (name, typ),
            Item::TsEnum { name, typ } => (name, typ),
            Item::TsModule { .. } => todo!("Item TsModule"),
        }
    }
}

#[derive(Debug)]
pub enum Nebulous<T> {
    Floating {
        module_path: CanonPath,
        item_name: String,
    },

    Rooted(T),
}

impl<T> Nebulous<T> {
    pub fn is_floating(&self) -> bool {
        if let Nebulous::Floating { .. } = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Value {
    name: String,
    typ: Type
}

#[derive(Debug, Clone)]
pub enum Type {
    Fn {
        origin: String,
        type_signature: FnType,
    },
    Class {
        name: String,
        origin: String,
        constructor: Box<Type>,
        fields: HashMap<String, Type>,
    },
    Interface {
        name: String,
        origin: String,
        fields: HashMap<String, Type>,
    },
    Alias {
        name: String,
        aliasing_type: Box<Type>,
    },
    UnsizedArray(Box<Type>),
    Array(Box<Type>, usize),
    Union,
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
    Undefined,
    Null,
}

#[derive(Debug, Clone)]
pub struct FnType {
    pub params: Vec<Type>,
    pub return_type: Option<Box<Type>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonPath(PathBuf);

impl CanonPath {
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }
}

impl From<CanonPath> for PathBuf {
    fn from(p: CanonPath) -> PathBuf {
        p.0
    }
}

impl std::convert::TryFrom<PathBuf> for CanonPath {
    type Error = std::io::Error;

    fn try_from(p: PathBuf) -> Result<Self, Self::Error> {
        p.canonicalize().map(|p| CanonPath(p))
    }
}

impl<'a> std::convert::TryFrom<&'a std::path::Path> for CanonPath {
    type Error = std::io::Error;

    fn try_from(p: &std::path::Path) -> Result<Self, Self::Error> {
        p.canonicalize().map(|p| CanonPath(p))
    }
}
