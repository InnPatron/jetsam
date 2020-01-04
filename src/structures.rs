use swc_common::Span;
use swc_ecma_ast::Str;

pub struct BindingModule {
    dependencies: Vec<Dependency>,
}

/// Dependency source path of the **input project, NOT the generated code**
pub struct Dependency(pub Str);

pub enum TypeAst {
    Fn(Str, FnType),
    Binding(BindingType),
    Array(Box<TypeAst>),
    Primitive(PrimitiveType),
}

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
pub struct BindingType {
    name: Str,
    origin: Str,
}

pub struct FnType {
    params: Vec<TypeAst>,
    return_type: Option<Box<TypeAst>>,
}
