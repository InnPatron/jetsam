use std::collections::HashMap;

use swc_common::Span;
use swc_ecma_ast::Str;

pub struct BindingModule {
    dependencies: Vec<Dependency>,
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
