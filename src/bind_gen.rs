use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::sync::Arc;

use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, FilePathMapping, SourceMap, Span
};
use swc_ecma_parser::{lexer::Lexer, Parser, Session, SourceFileInput, Syntax, TsConfig, JscTarget};
use swc_ecma_ast::*;
use super::structures::*;
use super::error::*;

// Marker types
struct ValueMarker;
struct TypeMarker;

pub struct Context<'a> {
    module_path: PathBuf,
    value_scope: Scope<ValueMarker>,
    type_scope: Scope<TypeMarker>,
    source_map: Arc<SourceMap>,
    session: Session<'a>,
}

impl<'a> Context<'a> {
    pub fn new(mut module_path: PathBuf, handler: &'a Handler, source_map: Arc<SourceMap>) -> Self {
        let session = Session {
            handler,
        };

        Context::prepare_path(&mut module_path);

        Context {
            module_path,
            value_scope: Scope::new(),
            type_scope: Scope::new(),
            session,
            source_map,
        }
    }

    fn prepare_path(path: &mut PathBuf) {
        if path.file_name().is_none() {
            panic!("Module path must contain a file");
        }

        if path.extension().is_none() {
            path.set_extension("d.ts");
        }
    }

    fn fork(&self, relative_path_to_module: PathBuf) -> Self {

        let module_path = {
            let mut current_path = self.module_path.clone();
            current_path.pop();
            let mut current_path = current_path.join(relative_path_to_module);

            Context::prepare_path(&mut current_path);
            current_path
        };

        Context {
            module_path,
            value_scope: Scope::new(),
            type_scope: Scope::new(),
            source_map: self.source_map.clone(),
            session: self.session,
        }
    }
}

struct Scope<T> {
    map: HashMap<String, Type>,
    marker: std::marker::PhantomData<T>,
}

impl<T> Scope<T> {
    fn new() -> Self {
        Scope {
            map: HashMap::new(),
            marker: std::marker::PhantomData,
        }
    }
}

impl Scope<ValueMarker> {
    fn try_import(&mut self, module: &ModuleInfo, export_key: String, as_key: Option<String>) -> bool {
        module.get_exported_value(&export_key)
            .map(|typ| {
                let key = as_key.unwrap_or(export_key.clone());
                self.map.insert(key, typ.clone());
                true
            })
        .unwrap_or(false)
    }
}

impl Scope<TypeMarker> {
    fn try_import(&mut self, module: &ModuleInfo, export_key: String, as_key: Option<String>) -> bool {
        module.get_exported_type(&export_key)
            .map(|typ| {
                let key = as_key.unwrap_or(export_key.clone());
                self.map.insert(key, typ.clone());
                true
            })
        .unwrap_or(false)
    }
}

fn open_from_src<'a, 'b>(
    original_context: &'a Context<'b>,
    src: &Str,
    span: Span
    ) -> Result<(Context<'b>, Module), BindGenError> {

    let path = PathBuf::from(src.value.to_string());
    let context = original_context.fork(path);

    let module = open_module(&context, Some(span))?;

    Ok((context, module))
}

pub fn open_module(context: &Context,
    span: Option<Span>,
    ) -> Result<Module, BindGenError> {
    use swc_common::{BytePos, SyntaxContext};

    let span = span
        .unwrap_or(Span::new(BytePos(0), BytePos(0), SyntaxContext::empty()));

    let file_handle = context.source_map
        .load_file(context.module_path.as_path())
        .map_err(|io_err| {
            BindGenError {
                kind: BindGenErrorKind::IoError(context.module_path.clone(), io_err),
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

pub fn process_module(mut context: Context, mut module: Module) -> Result<ModuleInfo, BindGenError> {

    let mut module_info = ModuleInfo::new(context.module_path.clone());

    hoist_imports(&mut module);
    for module_item in module.body {
        let result = process_module_item(&mut context, &mut module_info, module_item)?;
    }
    todo!();
}

fn hoist_imports(module: &mut Module) {
    use std::mem;

    let capacity = module.body.len();

    let mut module_items = Vec::with_capacity(capacity);
    mem::swap(&mut module_items, &mut module.body);

    let mut other_buffer = Vec::with_capacity(capacity);

    for module_item in module_items {
        match module_item {
            import @ ModuleItem::ModuleDecl(ModuleDecl::Import(..))
                => module.body.push(import),

            other => other_buffer.push(other),
        }
    }

    module.body.append(&mut other_buffer);
}

fn process_module_item(
    context: &mut Context,
    module_info: &mut ModuleInfo,
    item: ModuleItem,
    ) -> Result<(), BindGenError> {


    match item {
        ModuleItem::ModuleDecl(decl) => process_module_decl(context, module_info, decl),

        ModuleItem::Stmt(stmt) => todo!(),
    }
}

fn process_module_decl(
    context: &mut Context,
    module_info: &mut ModuleInfo,
    decl: ModuleDecl
    ) -> Result<(), BindGenError> {

    // TODO: Collect span info?
    match decl {

        ModuleDecl::Import(ImportDecl {
            src,
            span,
            specifiers,
        }) => {
            // TODO: Module cache
            let (dep_context, dep_module) =
                open_from_src(context, &src, span)?;

            let dep_module_info = process_module(dep_context, dep_module)?;

            for specifier in specifiers {
                match specifier {
                    ImportSpecifier::Specific(ImportSpecific {
                        span,
                        local,
                        imported,
                    }) => {
                        context.value_scope.try_import(
                            &dep_module_info,
                            local.sym.to_string(),
                            imported.as_ref().map(|ident| ident.sym.to_string())
                        );

                        context.type_scope.try_import(
                            &dep_module_info,
                            local.sym.to_string(),
                            imported.map(|ident| ident.sym.to_string())
                        );
                    }

                    ImportSpecifier::Default(def) => {
                        return Err(BindGenError {
                            kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultImport),
                            span: def.span,
                        });
                    }

                    ImportSpecifier::Namespace(namespace) => {
                        return Err(BindGenError {
                            kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultImport),
                            span: namespace.span,
                        });
                    }
                }
            }

            Ok(())
        }

        // TODO: Collect items for re-export
        ModuleDecl::ExportDecl(ExportDecl { .. }) => todo!(),

        ModuleDecl::ExportNamed(NamedExport {
            src,
            span,
            specifiers,
        }) => {

            match src {
                Some(src) => {
                    let (dep_context, dep_module) =
                        open_from_src(context, &src, span)?;
                    let dep_module_info = process_module(dep_context, dep_module)?;

                    for specifier in specifiers.into_iter() {
                        match specifier {
                            ExportSpecifier::Named(NamedExportSpecifier {
                                orig,
                                exported: exported_as,
                                ..
                            }) => {

                                let exported_key = exported_as.map(|x| x.sym.to_string());
                                let orig_key = orig.sym.to_string();
                                module_info.merge_export(
                                            &dep_module_info,
                                            orig_key,
                                            exported_key);
                            },

                            ExportSpecifier::Namespace(NamespaceExportSpecifier {
                                span,
                                ..
                            }) => {
                                return Err(BindGenError {
                                    kind: BindGenErrorKind::UnsupportedFeature(
                                              UnsupportedFeature::NamespaceExport),
                                    span,
                                });
                            }

                            ExportSpecifier::Default(..) => {
                                return Err(BindGenError {
                                    kind: BindGenErrorKind::UnsupportedFeature(
                                              UnsupportedFeature::DefaultExport),
                                    span,
                                });
                            }
                        }
                    }

                    Ok(())
                }

                None => todo!(),
            }
        }

        ModuleDecl::ExportAll(ExportAll {
            src,
            span,
            ..
        }) => {
            let (dep_context, dep_module) =
                open_from_src(context, &src, span)?;
            let dep_module_info = process_module(dep_context, dep_module)?;

            // Take all exports and merge into the current module
            module_info.merge_all(&dep_module_info);

            Ok(())
        }

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
