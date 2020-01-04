use swc_common::Span;
use swc_ecma_ast::Str;

pub struct BindingModule {
    dependencies: Vec<Dependency>,
}

/// Dependency source path of the **input project, NOT the generated code**
pub struct Dependency(pub Str);
