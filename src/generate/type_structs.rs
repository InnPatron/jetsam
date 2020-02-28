use std::collections::HashMap;

use swc_atoms::JsWord;

use super::structures::CanonPath;

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
