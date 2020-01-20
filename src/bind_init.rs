use std::collections::HashMap;
use std::path::PathBuf;
use std::convert::TryFrom;
use std::sync::Arc;

use swc_common::{BytePos, SyntaxContext, Span, SourceMap};
use swc_ecma_parser::{lexer::Lexer, Parser, Session, SourceFileInput, Syntax, TsConfig, JscTarget};
use swc_ecma_ast::{Str, Module};

use super::bind_common;
use super::structures::CanonPath;
use super::error::*;

pub struct ParsedModuleCache(pub HashMap<CanonPath, ModuleData>);

pub struct ModuleData {
    pub path: CanonPath,
    pub module_ast: Module,
    pub dependencies: HashMap<String, CanonPath>,
}

/// TODO: Take into account dependencies which may not be in the assumed location
///    because of pacakge managers.
///    i.e. need to take into account import { .. } from ".location/dependency"
///    Assuming the input is correct, emit a log warning of dependency instead of erroring
///
/// Starting from the root module, parse all Typescript '.d.ts' files in the project
///   and map to their canonical path.
pub fn init<'a>(
    source_map: Arc<SourceMap>,
    session: Session<'a>,
    root_module_path: PathBuf,
) -> Result<ParsedModuleCache, BindGenError> {

    let mut module_cache: HashMap<CanonPath, ModuleData> = HashMap::new();

    let root_module_path = CanonPath::try_from(root_module_path.clone())
        .map_err(|e| {
            BindGenError {
                kind: e.into(),
                module_path: root_module_path,
                span: Span::new(BytePos(0), BytePos(0), SyntaxContext::empty()),
            }
        })?;

    let mut work_stack: Vec<(CanonPath, Option<Span>)> = vec![(root_module_path, None)];

    while !work_stack.is_empty() {
        let (current_path, span) = work_stack
            .pop()
            .unwrap();

        if module_cache.contains_key(&current_path) {
            // Already initialized this module
            continue;
        }

        let span = span
            .unwrap_or(Span::new(BytePos(0), BytePos(0), SyntaxContext::empty()));

        let module_ast = open_module(
            &source_map,
            session,
            &current_path,
            span.clone(),
        )?;

        let dependencies = scan_dependencies(&current_path, &module_ast, span)?;
        work_stack.extend(dependencies.iter().map(|(k, (p, s))| (p.clone(), Some(s.clone()))));

        let module_data = ModuleData {
            path: current_path.clone(),
            module_ast,
            dependencies: dependencies.into_iter().map(|(k, (p, s))| (k, p)).collect(),
        };

        module_cache.insert(current_path.clone(), module_data);
    }

    Ok(ParsedModuleCache(module_cache))
}

fn scan_dependencies(
    module_path: &CanonPath,
    module_ast: &Module,
    original_span: Span,
) -> Result<HashMap<String, (CanonPath, Span)>, BindGenError> {
    use swc_ecma_ast::*;

    let handle_decl = |decl: &ModuleDecl| -> Result<Option<(String, CanonPath, Span)>, BindGenError> {
        let maybe_dep: Option<(&Str, &Span)> = match decl {
            ModuleDecl::Import(ImportDecl {
                ref src,
                ref span,
                ..
            }) => Some((src, span)),

            ModuleDecl::ExportDecl(..) => None,

            ModuleDecl::ExportNamed(NamedExport {
                ref src,
                ref span,
                ..
            }) => src.as_ref().map(|src| (src, span)),

            ModuleDecl::ExportDefaultDecl(ref export) => {
                return Err(BindGenError {
                    module_path: module_path.as_path().to_owned(),
                    kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultExport),
                    span: export.span.clone(),
                })
            }

            ModuleDecl::ExportDefaultExpr(ref export) => {
                return Err(BindGenError {
                    module_path: module_path.as_path().to_owned(),
                    kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultExport),
                    span: export.span.clone(),
                });
            },

            ModuleDecl::ExportAll(ExportAll {
                ref src,
                ref span,
            }) => Some((src, span)),

            ModuleDecl::TsImportEquals(TsImportEqualsDecl { ref span, .. }) => {
                return Err(BindGenError {
                    module_path: module_path.as_path().to_owned(),
                    kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsImportEquals),
                    span: span.clone(),
                });
            }

            ModuleDecl::TsExportAssignment(TsExportAssignment { ref span, .. }) => {
                return Err(BindGenError {
                    module_path: module_path.as_path().to_owned(),
                    kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsExportAssignment),
                    span: span.clone(),
                });
            }

            ModuleDecl::TsNamespaceExport(TsNamespaceExportDecl { ref span, .. }) => {

                // TODO: Handle TsNamespaceExport?
                //   What is TsNamespaceExport??
                return Err(BindGenError {
                    module_path: module_path.as_path().to_owned(),
                    kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::TsNamespaceExport),
                    span: span.clone(),
                });
            }
        };

        maybe_dep.map(|(src, span)| {
            let dep_buf = PathBuf::from(src.value.to_string());
            bind_common::locate_dependency(module_path.as_path(), &dep_buf)
                .map(|path_result| path_result.map(|path| (src.value.to_string(), path, span.clone())))
        })
            .transpose()
            .map(|opt| opt.flatten())
    };

    let mut dep_buf = HashMap::new();
    for module_item in module_ast.body.iter() {
        match module_item {
            ModuleItem::ModuleDecl(ref decl) => {
                if let Some((src, dep, span)) = handle_decl(decl)? {
                    dep_buf.insert(src, (dep, span));
                }
            }

            ModuleItem::Stmt(..) => (),
        }
    }

    Ok(dep_buf)
}

fn open_module<'a>(
    source_map: &Arc<SourceMap>,
    session: Session<'a>,
    path: &CanonPath,
    span: Span,
) -> Result<Module, BindGenError> {

    let file_handle = source_map
        .load_file(path.as_path())
        .map_err(|io_err| {
            BindGenError {
                kind: BindGenErrorKind::IoError(io_err),
                span: span.clone(),
                module_path: path.as_path().to_owned(),
            }
        })?;

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
                module_path: path.as_path().to_owned(),
            }
        })
}
