use std::collections::HashMap;
use std::path::{PathBuf, Path};

use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, FilePathMapping, SourceMap, Span
};
use swc_ecma_parser::{lexer::Lexer, Parser, Session, SourceFileInput, Syntax, TsConfig, JscTarget};
use swc_ecma_ast::*;
use super::structures::*;
use super::error::*;

pub struct Context<'a> {
    pub module_path: PathBuf,
    pub scope: Scope,
    pub typing_env: TypeEnv,
    pub session: Session<'a>,
}

impl<'a> Context<'a> {
    pub fn new(module_path: PathBuf, handler: &Handler) -> Self {
        let session = Session {
            handler,
        };

        Context {
            module_path,
            scope: Scope::new(),
            typing_env: TypeEnv::new(),
            session,
        }
    }

    fn fork(&self, new_module: PathBuf) -> Self {
        Context {
            module_path: new_module,
            scope: Scope::new(),
            typing_env: TypeEnv::new(),
            session: self.session,
        }
    }
}

pub struct Scope {
    map: HashMap<String, ()>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            map: HashMap::new(),
        }
    }
}

pub struct TypeEnv {
    map: HashMap<String, ()>,
}

impl TypeEnv {
    fn new() -> Self {
        TypeEnv {
            map: HashMap::new(),
        }
    }
}

pub fn open_module(context: &Context, source_map: SourceMap, module_path: &Path, span: Option<Span>)
    -> Result<Module, BindGenError> {
    use swc_common::{BytePos, SyntaxContext};

    let span = span
        .unwrap_or(Span::new(BytePos(0), BytePos(0), SyntaxContext::empty()));

    let file_handle = source_map
        .load_file(module_path)
        .map_err(|io_err| {
            BindGenError {
                kind: BindGenErrorKind::IoError(module_path.to_path_buf(), io_err),
                span: span.clone(),
            }
        })?;

    let session = context.session;
    let lexer = Lexer::new(
        session,
        Syntax::Typescript(TsConfig {
            tsx: false,
            decorators: false,
            dynamic_import: false,
        }),
        JscTarget::Es2018,
        SourceFileInput::from(&*file_handle),
        None,
    );

    let mut parser = Parser::new_from(session, lexer);

    parser
        .parse_module()
        .map_err(|mut e| {
            e.emit();

            BindGenError {
                kind: BindGenErrorKind::ParserError,
                span: span.clone(),
            }
        })
}

pub fn process_module(mut context: Context, module: Module) -> Result<BindingModule, BindGenError> {

    let mut depedencies: Vec<Dependency> = Vec::new();
    for module_item in module.body {
        let result = process_module_item(&mut context, &mut depedencies, module_item)?;
    }
    todo!();
}

fn process_module_item(
    context: &mut Context,
    dependencies: &mut Vec<Dependency>,
    item: ModuleItem,
    ) -> Result<(), BindGenError> {


    match item {
        ModuleItem::ModuleDecl(decl) => {
            let dependency = module_item_dependency(context, &decl);
            let _module =
            todo!();
        },

        ModuleItem::Stmt(stmt) => todo!(),
    }

    todo!();
}

fn module_item_dependency(
    context: &mut Context,
    decl: &ModuleDecl
    ) -> Result<Option<Dependency>, BindGenError> {

    // TODO: Collect span info?
    match decl {

        // TODO: Collect import names for later?
        ModuleDecl::Import(ImportDecl {
            ref src,
            ..
        }) => Ok(Some(Dependency(src.clone()))),

        // TODO: Collect items for re-export
        ModuleDecl::ExportDecl(ExportDecl { .. }) => todo!(),

        // TODO: Collect items for re-export
        ModuleDecl::ExportNamed(NamedExport {
            ref src,
            ..
        }) => Ok(src.as_ref().map(|src| Dependency(src.clone()))),

        ModuleDecl::ExportAll(ExportAll {
            ref src,
            ..
        }) => Ok(Some(Dependency(src.clone()))),

        ModuleDecl::ExportDefaultDecl(ExportDefaultDecl { ref span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultExport),
                span: span.clone(),
            })
        }

        ModuleDecl::ExportDefaultExpr(ExportDefaultExpr { ref span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultExport),
                span: span.clone(),
            })
        }

        ModuleDecl::TsImportEquals(TsImportEqualsDecl { ref span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsImportEquals),
                span: span.clone(),
            })
        }

        ModuleDecl::TsExportAssignment(TsExportAssignment { ref span, .. }) => {
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsExportAssignment),
                span: span.clone(),
            })
        }

        ModuleDecl::TsNamespaceExport(TsNamespaceExportDecl { ref span, .. }) => {

            // TODO: Handle TsNamespaceExport?
            //   What is TsNamespaceExport??
            Err(BindGenError {
                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsNamespaceExport),
                span: span.clone(),
            })
        }
    }
}
