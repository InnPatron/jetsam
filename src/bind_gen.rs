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

    fn insert(&mut self, key: String, typ: Type) {
        self.map.insert(key, typ);
    }

    fn get(&self, key: &str) -> Option<&Type> {
        self.map.get(key)
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

    Ok(module_info)
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

        ModuleDecl::ExportDecl(ExportDecl {
            span,
            decl,
        }) => {
            let decl_item = process_decl(context, module_info, decl, span)?;
            let item_kind = decl_item.item_kind();
            let (name, typ) = decl_item.into_data();

            match item_kind {
                ItemKind::Value => {
                    module_info.export_value(name.clone(), typ.clone());
                    context.value_scope.insert(name, typ);
                }

                ItemKind::Type => {
                    module_info.export_type(name.clone(), typ.clone());
                    context.type_scope.insert(name, typ);
                }
                ItemKind::ValueType => {
                    module_info.export_value(name.clone(), typ.clone());
                    module_info.export_type(name.clone(), typ.clone());
                    context.value_scope.insert(name.clone(), typ.clone());
                    context.type_scope.insert(name, typ);
                }
            }

            Ok(())
        }

        ModuleDecl::ExportNamed(NamedExport {
            src,
            span,
            specifiers,
        }) => {

            let mut exporter: Box<FnMut(String, Option<String>) -> ()> = match src {

                // Open the source module and re-export an exported item
                Some(src) => {
                    let (dep_context, dep_module) =
                        open_from_src(context, &src, span)?;
                    let dep_module_info = process_module(dep_context, dep_module)?;

                    Box::new(move |original_key: String, as_key: Option<String>| -> () {
                        module_info.merge_export(
                                    &dep_module_info,
                                    original_key,
                                    as_key);
                    })
                }

                // Export an item from the current module
                None => {
                    Box::new(|original_key: String, as_key: Option<String>| -> () {

                        let as_key = as_key.unwrap_or(original_key.clone());

                        let value_type = context.value_scope
                            .get(&original_key)
                            .map(|v| v.clone());
                        let type_item = context.type_scope
                            .get(&original_key)
                            .map(|t| t.clone());

                        if value_type.is_none() && type_item.is_none() {
                            panic!("Invalid export. No item named {} in scope", &original_key);
                        }

                        if let Some(value_type) = value_type {
                            module_info.export_value(as_key.clone(), value_type);
                        }

                        if let Some(type_item) = type_item {
                            module_info.export_type(as_key, type_item);
                        }
                    })
                }
            };

            for specifier in specifiers.into_iter() {
                match specifier {
                    ExportSpecifier::Named(NamedExportSpecifier {
                        orig,
                        exported: exported_as,
                        ..
                    }) => {

                        let orig_key = orig.sym.to_string();
                        let exported_key = exported_as
                            .map(|x| x.sym.to_string());

                        exporter(orig_key, exported_key);
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

fn process_decl(
    context: &Context,
    module_info: &ModuleInfo,
    decl: Decl,
    span: Span,
    ) -> Result<Item, BindGenError> {

    match decl {
        Decl::Class(ClassDecl {
            ident,
            class: Class {
                span,
                body,
                type_params,
                ..
            },
            ..
        }) => {
            // TODO: Type parameters
            // TODO: Subtyping relations?
            // TODO: Implements?

            todo!();
        },

        Decl::Fn(FnDecl {
            ident,
            function: Function {
                span,
                params,
                is_generator,
                is_async,
                return_type,
                type_params,
                ..
            },
            ..
        }) => {
            // TODO: Type parameters

            let params = params
                .into_iter()
                .map(|p| {
                    type_from_pattern(context, module_info, p)
                })
            .collect::<Result<Vec<_>, _>>()?;

            let return_type = return_type
                .map(|ann| type_from_ann(context, module_info, ann))
                .transpose()?
                .map(|typ| Box::new(typ));

            let fn_type = Type::Fn {
                origin: context.module_path.display().to_string(),
                type_signature: FnType {
                    params,
                    return_type,
                }
            };

            Ok(Item::Fn {
                name: ident.sym.to_string(),
                typ: fn_type,
            })
        },

        Decl::Var(VarDecl {
            decls,
            ..
        }) => {
            todo!();
        },

        Decl::TsInterface(TsInterfaceDecl {
            id,
            span,
            body,
            type_params,
            ..
        }) => {
            // TODO: Type parameters
            // TODO: Implements?

            todo!();
        },

        Decl::TsTypeAlias(TsTypeAliasDecl {
            span,
            id,
            type_ann,
            type_params,
            ..
        }) => {
            // TODO: Type parameters

            let aliasing_type = bind_type(context, module_info, *type_ann)?;
            let name = id.sym.to_string();

            Ok(Item::TsTypeAlias{
                name,
                typ: aliasing_type,
            })
        },

        Decl::TsEnum(TsEnumDecl {
            span,
            id,
            ..
        }) => {
            // TODO: Care about inhabitants?

            todo!();
        },

        Decl::TsModule(..) => {
            todo!("TS modules not sup");
        },
    }
}

fn type_from_pattern(
    context: &Context,
    module_info: &ModuleInfo,
    pattern: Pat,
) -> Result<Type, BindGenError> {
    // TODO: Perform basic type inference?
    let ann: Option<_> = match pattern {
        Pat::Ident(ident) => ident.type_ann,
        Pat::Array(array_pat) => array_pat.type_ann,
        Pat::Rest(rest_pat) => rest_pat.type_ann,
        Pat::Object(object_pat) => object_pat.type_ann,
        Pat::Assign(assign_pat) => assign_pat.type_ann,
        Pat::Invalid(invalid) => todo!("Invalid pattern {:?}", invalid),
        Pat::Expr(expr) => todo!("What is an expr pattern"),
    };

    ann
        .map(|ann| type_from_ann(context, module_info, ann))
        .unwrap_or(Ok(Type::Primitive(PrimitiveType::Any)))
}

fn type_from_ann(
    context: &Context,
    module_info: &ModuleInfo,
    ann: TsTypeAnn,
) -> Result<Type, BindGenError> {
    let ann_span = ann.span;

    bind_type(context, module_info, *ann.type_ann)
}

fn bind_type(
    context: &Context,
    module_info: &ModuleInfo,
    typ: TsType,
) -> Result<Type, BindGenError> {

    match typ {
        TsType::TsKeywordType(TsKeywordType {
            span,
            kind,
        }) => {

            let prim_type = match kind {
                TsKeywordTypeKind::TsAnyKeyword => PrimitiveType::Any,
                TsKeywordTypeKind::TsUnknownKeyword => todo!("unknown keyword type"),
                TsKeywordTypeKind::TsNumberKeyword => PrimitiveType::Number,
                TsKeywordTypeKind::TsObjectKeyword => PrimitiveType::Object,
                TsKeywordTypeKind::TsBooleanKeyword => PrimitiveType::Boolean,
                TsKeywordTypeKind::TsBigIntKeyword => todo!("big int keyword type"),
                TsKeywordTypeKind::TsStringKeyword => PrimitiveType::String,
                TsKeywordTypeKind::TsSymbolKeyword => todo!("symbol keyword type"),
                TsKeywordTypeKind::TsVoidKeyword => PrimitiveType::Void,
                TsKeywordTypeKind::TsUndefinedKeyword => PrimitiveType::Undefined,
                TsKeywordTypeKind::TsNullKeyword => PrimitiveType::Null,
                TsKeywordTypeKind::TsNeverKeyword => PrimitiveType::Never,
            };

            Ok(Type::Primitive(prim_type))
        },

        TsType::TsThisType(TsThisType {
            span,
        }) => {
            todo!("What is TsThisType?");
        },

        TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsFnType(TsFnType {
            span,
            params,
            type_params,
            type_ann,
        })) => {
            // What is type_ann
            // Is type_ann the return type?
            todo!("ts fn");
        },

        TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsConstructorType(TsConstructorType {
            span,
            params,
            type_params,
            type_ann,
        })) => {
            // What is type_ann
            // Is type_ann the return type?
            todo!("ts constructor");
        },

        TsType::TsTypeRef(TsTypeRef {
            span,
            type_name,
            type_params,
        }) => {
            todo!();
        },

        TsType::TsTypeQuery(_TsTypeQuery) => {
            todo!("ts type query");
        },

        TsType::TsTypeLit(..) => {
            todo!("ts type literals not supported");
        },

        TsType::TsArrayType(TsArrayType {
            span,
            elem_type,
        }) => {
            let elem_type = Box::new(bind_type(context, module_info, *elem_type)?);
            Ok(Type::UnsizedArray(elem_type))
        },

        TsType::TsTupleType(TsTupleType {
            span,
            elem_types,
        }) => {
            // Tuple types are fixed-length arrays (at init)
            todo!("ts tuple type");
        },

        TsType::TsOptionalType(..) => {
            todo!("ts optional types not supported");
        },

        TsType::TsRestType(..) => {
            todo!("ts rest types not supported");
        },

        TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(TsUnionType {
            span,
            types,
        })) => {
            todo!("ts union type");
        },

        TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsIntersectionType(..)) => {
            todo!("ts intersection types not supported");
        },

        TsType::TsConditionalType(..) => {
            todo!("ts conditional types not supported");
        },

        TsType::TsInferType(..) => {
            todo!("ts infer type not supported");
        },

        TsType::TsParenthesizedType(TsParenthesizedType {
            span,
            type_ann,
        }) => {
            todo!("parenthesized type");
        },

        TsType::TsTypeOperator(_TsTypeOperator) => {
            todo!("type operators not supported");
        },

        TsType::TsIndexedAccessType(_TsIndexedAccessType) => {
            todo!("ts indexed access type not supported");
        },

        TsType::TsMappedType(_TsMappedType) => {
            todo!("ts mapped type not supported");
        },

        TsType::TsLitType(TsLitType {
            span,
            lit,
        }) => {
            todo!("ts lit type");
        },

        TsType::TsTypePredicate(_TsTypePredicate) => {
            todo!("ts type predicates not supported?");
        },

        TsType::TsImportType(_TsImportType) => {
            todo!("What is TsImportType?");
        },
    }
}
