use std::collections::HashMap;

use swc_ecma_ast::*;
use swc_common::Span;

use super::bind_init;
use super::error::*;
use super::structures::*;

pub struct ModuleGraph {
    pub module_graph: HashMap<CanonPath, ModuleInfo>,
}

struct Context {
    value_scope: Scope<Nebulous<Value>>,
    type_scope: Scope<Nebulous<Type>>,
}

struct Scope<T> {
    map: HashMap<String, T>,
}

impl<T> Scope<Nebulous<T>> {
    fn new() -> Self {
        Scope {
            map: HashMap::new(),
        }
    }

    fn insert(&mut self, key: String, to_insert: Nebulous<T>) {
        use std::collections::hash_map::Entry;

        match self.map.entry(key) {
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
    }

    fn get(&self, key: &str) -> Option<&Nebulous<T>> {
        self.map.get(key)
    }
}

pub fn build_module_graph(
    module_cache: bind_init::ParsedModuleCache
) -> Result<ModuleGraph, BindGenError> {

    let mut module_graph: HashMap<CanonPath, ModuleInfo> = HashMap::new();
    for (canon_path, mut module) in module_cache.0.into_iter() {

        let mut context = Context {
            value_scope: Scope::new(),
            type_scope: Scope::new(),
        };

        let mut module_info = ModuleInfo::new(canon_path.clone().into(), module.dependencies);

        hoist_imports(&mut module.module_ast);

        let mut bind_gen_session = BindGenSession;
        bind_gen_session.process_module(&mut context, &mut module_info, module.module_ast)?;

        module_graph.insert(canon_path, module_info);
    }

    Ok(ModuleGraph {
        module_graph
    })
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

struct BindGenSession;

impl BindGenSession {

    fn process_module(
        &mut self,
        context: &mut Context,
        module_info: &mut ModuleInfo,
        module: Module
    ) -> Result<(), BindGenError> {
        for item in module.body {
            self.process_module_item(context, module_info, item)?;
        }

        Ok(())
    }

    fn process_module_item(
        &mut self,
        context: &mut Context,
        module_info: &mut ModuleInfo,
        item: ModuleItem,
        ) -> Result<(), BindGenError> {


        match item {
            ModuleItem::ModuleDecl(decl) => self.process_module_decl(context, module_info, decl),

            ModuleItem::Stmt(stmt) => todo!("ModuleItem::Stmt"),
        }
    }

    fn process_module_decl(
        &mut self,
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
                for specifier in specifiers {
                    match specifier {
                        ImportSpecifier::Specific(ImportSpecific {
                            span,
                            local,
                            imported,
                        }) => {
                            // context.value_scope.insert(
                            let import_canon_path =
                                module_info.get_dep_canon_path(&src.value.to_string());
                            let item_name = imported
                                .map(|ident| ident.sym.to_string())
                                .unwrap_or_else(|| local.sym.to_string());
                            let import_name = local.sym.to_string();
                            context.value_scope.insert(import_name, Nebulous::Floating {
                                item_name,
                                module_path: import_canon_path,
                            });
                        }

                        ImportSpecifier::Default(def) => {
                            return Err(BindGenError {
                                module_path: module_info.path().to_owned(),
                                kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultImport),
                                span: def.span,
                            });
                        }

                        ImportSpecifier::Namespace(namespace) => {
                            return Err(BindGenError {
                                module_path: module_info.path().to_owned(),
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
                let decl_item = BindGenSession::process_decl(context, module_info, decl, span)?;
                let item_kind = decl_item.item_kind();
                let (name, typ) = decl_item.into_data();

                match item_kind {
                    ItemKind::Value => {
                        module_info.export_value(name.clone(), Nebulous::Rooted(Value {
                            name: name.clone(),
                            typ: typ.clone(),
                        }));
                        context.value_scope.insert(name.clone(), Nebulous::Rooted(Value {
                            name,
                            typ,
                        }));
                    }

                    ItemKind::Type => {
                        module_info.export_type(name.clone(), Nebulous::Rooted(typ.clone()));
                        context.type_scope.insert(name, Nebulous::Rooted(typ));
                    }

                    ItemKind::ValueType => {
                        module_info.export_value(name.clone(), Nebulous::Rooted(Value {
                            name: name.clone(),
                            typ: typ.clone(),
                        }));
                        module_info.export_type(name.clone(), Nebulous::Rooted(typ.clone()));

                        context.value_scope.insert(name.clone(), Nebulous::Rooted(Value {
                            name: name.clone(),
                            typ: typ.clone(),
                        }));
                        context.type_scope.insert(name, Nebulous::Rooted(typ));
                    }
                }

                Ok(())
            }

            ModuleDecl::ExportNamed(NamedExport {
                src,
                span,
                specifiers,
            }) => {

                let module_path = module_info.path().to_owned();
                let mut exporter: Box<FnMut(String, Option<String>) -> ()> = match src {

                    // Open the source module and re-export an exported item
                    Some(src) => {

                        Box::new(move |original_key: String, as_key: Option<String>| -> () {
                            let as_key = as_key.unwrap_or(original_key.clone());
                            let export_canon_path =
                                module_info.get_dep_canon_path(&src.value.to_string());

                            module_info.export_value(as_key.clone(), Nebulous::Floating {
                                module_path: export_canon_path.clone(),
                                item_name: original_key.clone(),
                            });

                            module_info.export_type(as_key, Nebulous::Floating {
                                module_path: export_canon_path,
                                item_name: original_key,
                            });
                        })
                    }

                    // Export an item from the current module
                    None => {
                        Box::new(|original_key: String, as_key: Option<String>| -> () {

                            let as_key = as_key.unwrap_or(original_key.clone());

                            let value_item = context.value_scope
                                .get(&original_key)
                                .map(|v| v.clone());
                            let type_item = context.type_scope
                                .get(&original_key)
                                .map(|t| t.clone());

                            if value_item.is_none() && type_item.is_none() {
                                panic!("Invalid export. No item named {} in scope", &original_key);
                            }

                            if let Some(value_item) = value_item {
                                module_info.export_value(as_key.clone(), value_item);
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
                                module_path,
                                kind: BindGenErrorKind::UnsupportedFeature(
                                          UnsupportedFeature::NamespaceExport),
                                span,
                            });
                        }

                        ExportSpecifier::Default(..) => {
                            return Err(BindGenError {
                                module_path,
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

                // Mark module as re-exporting all of another module
                // NOTE: The current way of tracking re-export all
                //   does NOT work if there are conflicting re-exports.
                //   ORDER MATTERS FOR ALL EXPORTS BUT THAT IS TOO DIFFICULT
                //     TO HANDLE IN GENERAL.
                //   PLANK WILL FAIL TO CORRECTLY GENERATE MODULES WHICH RELY ON EXPORT ORDER
                //     FOR A CORRECT INTERFACE.
                let dep_canon_path = module_info.get_dep_canon_path(&src.value.to_string());
                module_info.export_all.push(dep_canon_path);

                Ok(())
            }

            ModuleDecl::ExportDefaultDecl(..)
                => unreachable!("Caught in init"),

            ModuleDecl::ExportDefaultExpr(..)
                => unreachable!("Caught in init"),

            ModuleDecl::TsImportEquals(..)
                => unreachable!("Caught in init"),

            ModuleDecl::TsExportAssignment(..)
                => unreachable!("Caught in init"),

            ModuleDecl::TsNamespaceExport(..)
                => unreachable!("Caught in init"),
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

                let name = ident.sym.to_string();
                let origin = module_info.path().display().to_string();

                // TODO: Constructor type generation
                let constructor = Box::new(Type::Fn {
                    origin: origin.clone(),
                    type_signature: FnType {
                        params: Vec::new(),
                        return_type: None,
                    },
                });

                // TODO: Class fields
                let fields = HashMap::new();

                Ok(Item::Class {
                    name: name.clone(),
                    typ: Type::Class {
                        name,
                        origin,
                        constructor,
                        fields,
                    },
                })
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
                        BindGenSession::type_from_pattern(context, module_info, p)
                    })
                .collect::<Result<Vec<_>, _>>()?;

                let return_type = return_type
                    .map(|ann| BindGenSession::type_from_ann(context, module_info, ann))
                    .transpose()?
                    .map(|typ| Box::new(typ));

                let fn_type = Type::Fn {
                    origin: module_info.path().display().to_string(),
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

                // TODO: Handle patterns
                for var_decl in decls {
                    match var_decl.name {
                        Pat::Ident(ident) => {
                            return Ok(Item::Var {
                                name: ident.sym.to_string(),
                                typ: Type::Primitive(PrimitiveType::Any),
                            });
                        }

                        _ => todo!("variable patterns"),
                    }
                }

                unreachable!();
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

                let name = id.sym.to_string();
                let origin = module_info.path().display().to_string();

                // TODO: Interface fields
                let fields = HashMap::new();

                Ok(Item::TsInterface{
                    name: name.clone(),
                    typ: Type::Interface {
                        name,
                        origin,
                        fields,
                    },
                })
            },

            Decl::TsTypeAlias(TsTypeAliasDecl {
                span,
                id,
                type_ann,
                type_params,
                ..
            }) => {
                // TODO: Type parameters

                let aliasing_type = BindGenSession::bind_type(context, module_info, *type_ann)?;
                let name = id.sym.to_string();

                Ok(Item::TsTypeAlias{
                    name: name.clone(),
                    typ: Type::Alias {
                        name,
                        aliasing_type: Box::new(aliasing_type),
                    }
                })
            },

            Decl::TsEnum(TsEnumDecl {
                span,
                id,
                ..
            }) => {
                // TODO: Care about inhabitants?

                let name = id.sym.to_string();
                Ok(Item::TsEnum{
                    name,
                    typ: Type::Primitive(PrimitiveType::Any),
                })
            },

            Decl::TsModule(m) => {
                todo!("TS modules not suppported: {}::{:?}", module_info.path().display(), m.id);
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
            .map(|ann| BindGenSession::type_from_ann(context, module_info, ann))
            .unwrap_or(Ok(Type::Primitive(PrimitiveType::Any)))
    }

    fn type_from_ann(
        context: &Context,
        module_info: &ModuleInfo,
        ann: TsTypeAnn,
    ) -> Result<Type, BindGenError> {
        let ann_span = ann.span;

        BindGenSession::bind_type(context, module_info, *ann.type_ann)
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
                // todo!("{}:{:?}", module_info.path().display(), type_name);
                // TODO: TsTypeRef

                Ok(Type::Primitive(PrimitiveType::Any))
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
                let elem_type = Box::new(BindGenSession::bind_type(context, module_info, *elem_type)?);
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
                // TODO: How to bind union types?
                // Keep opaque for now
                Ok(Type::Union)
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
}
