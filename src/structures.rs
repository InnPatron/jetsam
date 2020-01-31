use std::hash::Hash;
use std::collections::HashMap;
use std::path::PathBuf;

use swc_atoms::JsWord;
use swc_common::Span;
use swc_ecma_ast::Str;

/// NOTE: The current way of tracking re-export all
///   does NOT work if there are conflicting re-exports.
///   ORDER MATTERS FOR ALL EXPORTS BUT THAT IS TOO DIFFICULT
///     TO HANDLE IN GENERAL.
///   PLANK WILL FAIL TO CORRECTLY GENERATE MODULES WHICH RELY ON EXPORT ORDER
///     FOR A CORRECT INTERFACE.
pub struct ModuleInfo {
    path: PathBuf,
    dependencies: HashMap<String, CanonPath>,
    pub export_all: Vec<CanonPath>,
    pub exported_types: HashMap<String, Nebulous<Type>>,
    pub exported_values: HashMap<String, Nebulous<Value>>,
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

    pub fn export_value(&mut self, name: String, to_insert: Nebulous<Value>) {
        use std::collections::hash_map::Entry;

        match self.exported_values.entry(name) {
            Entry::Occupied(ref mut occupied) => {
                if to_insert.is_floating() == false {
                    // Rooted values should not be overwritten at the module scope
                    assert!(occupied.get().is_floating());

                    occupied.insert(to_insert);
                }
            }

            Entry::Vacant(vacant) => {
                vacant.insert(to_insert);
            }
        }

        // self.exported_values.insert(name, to_insert);
    }

    pub fn export_type(&mut self, name: String, to_insert: Nebulous<Type>) {
        use std::collections::hash_map::Entry;

        match self.exported_types.entry(name) {
            Entry::Occupied(ref mut occupied) => {
                if to_insert.is_floating() == false {
                    // Rooted values should not be overwritten at the module scope
                    assert!(occupied.get().is_floating());

                    occupied.insert(to_insert);
                }
            }

            Entry::Vacant(vacant) => {
                vacant.insert(to_insert);
            }
        }

        // self.exported_types.insert(name, to_insert);
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

#[derive(Debug, Clone)]
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
    pub name: String,
    pub typ: Type
}

#[derive(Debug, Clone)]
pub enum Type {
    Named {
        name: JsWord,
        source: CanonPath,
    },
    Fn(FnType),
    Class(ClassType),
    Interface {
        name: JsWord,
        origin: CanonPath,
        fields: HashMap<JsWord, Type>,
    },
    Literal {
        fields: HashMap<JsWord, Type>,
    },
    Alias {
        name: JsWord,
        aliasing_type: Box<Type>,
    },
    Opaque {
        name: JsWord,
        origin: CanonPath,
    },
    UnsizedArray(Box<Type>),
    Array(Box<Type>, usize),
    Union,
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
    pub return_type: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct ClassType {
    pub name: JsWord,
    pub origin: CanonPath,
    pub constructors: Vec<FnType>,
    pub members: HashMap<JsWord, Type>,
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
