use std::collections::{HashMap, HashSet};

use swc_ecma_ast::*;
use swc_atoms::JsWord;
use swc_common::Span;

use super::bind_init::{ModuleData, ParsedModuleCache as ModuleCache};
use super::structures::{Type, CanonPath};
use super::error::*;

pub fn init(cache: &ModuleCache) -> Result<ModuleGraph, BindGenError> {
    let mut graph = ModuleGraph {
        nodes: HashMap::new(),
        export_edges: HashMap::new(),
        import_edges: HashMap::new(),
    };

    for (_, module_data) in cache.0.iter() {
        NodeInitSession::init(&mut graph, cache, module_data)?;
    }

    Ok(graph)
}

pub struct ModuleNode {
    pub path: CanonPath,
    pub rooted_export_types: HashMap<JsWord, Type>,
    pub rooted_export_values: HashMap<JsWord, Type>,
}

pub enum Import {
    // Unused until TS 3.8
    NamedType {

    },
    Named {
        source: CanonPath,
        src_key: JsWord,
    },
}

pub enum Export {
    // Unused until TS 3.8
    NamedType {

    },
    Named {
        source: CanonPath,
        src_key: JsWord,
        export_key: JsWord,
    },
    All {
        source: CanonPath,
    },
}

/// ORDER OF EXPORTS MATTERS
/// ORDER OF IMPORTS MATTERS
///
/// Ordered by occurence in the AST
pub struct ModuleGraph {
    pub nodes: HashMap<CanonPath, ModuleNode>,
    pub export_edges: HashMap<CanonPath, Vec<Export>>,
    pub import_edges: HashMap<CanonPath, Vec<Import>>,
}

impl ModuleGraph {
    fn module_instantiated(&self, p: &CanonPath) -> bool {
        self.nodes.contains_key(p)
    }
}

#[derive(Clone)]
enum ItemState {
    Imported {
        source: CanonPath,
        src_key: JsWord,
        as_key: JsWord,
    },

    Rooted {
        typ: Type,
    },
}

#[derive(Copy, Clone)]
enum ScopeKind {
    Value,
    Type,
    ValueType,
}

struct NodeInitSession<'a> {
    path: &'a CanonPath,
    dependency_map: &'a HashMap<String, CanonPath>,
    import_edges: Vec<Import>,
    export_edges: Vec<Export>,
    rooted_values: HashMap<JsWord, Type>,
    rooted_types: HashMap<JsWord, Type>,

    value_scope: HashMap<JsWord, ItemState>,
    type_scope: HashMap<JsWord, ItemState>,
}

macro_rules! get_dep_src {
    ($self: expr, $src_str: expr) => {
        $self.dependency_map.get(&*$src_str.value).expect("Source path not found in dependency_map")
    }

}

impl<'a> NodeInitSession<'a> {

    fn init(
        g: &mut ModuleGraph,
        cache: &ModuleCache,
        module_data: &ModuleData
    ) -> Result<(), BindGenError> {
        let mut session = NodeInitSession {
            path: &module_data.path,
            dependency_map: &module_data.dependencies,
            import_edges: Vec::new(),
            export_edges: Vec::new(),
            rooted_values: HashMap::new(),
            rooted_types: HashMap::new(),

            value_scope: HashMap::new(),
            type_scope: HashMap::new(),
        };

        for item in module_data.module_ast.body.iter() {
            session.process_module_item(item)?;
        }

        let rooted_export_types = session.rooted_types;
        let rooted_export_values = session.rooted_values;
        let import_edges = session.import_edges;
        let export_edges = session.export_edges;

        let module_node = ModuleNode {
            path: module_data.path.clone(),
            rooted_export_types,
            rooted_export_values,
        };

        g.nodes.insert(module_data.path.clone(), module_node);

        g.export_edges.insert(module_data.path.clone(), export_edges);
        g.import_edges.insert(module_data.path.clone(), import_edges);

        Ok(())
    }

    fn scope_item(&mut self, name: JsWord, state: ItemState, kind: ScopeKind) {
        use std::collections::hash_map::Entry;

        match kind {
            ScopeKind::Value => {
                match self.value_scope.entry(name) {
                    Entry::Vacant(vacant) => { vacant.insert(state); },
                    Entry::Occupied(ref mut occupied) => (),
                }
            }

            ScopeKind::Type => {
                match self.type_scope.entry(name) {
                    Entry::Vacant(vacant) => { vacant.insert(state); },
                    Entry::Occupied(ref mut occupied) => (),
                }
            }

            ScopeKind::ValueType => {
                match self.type_scope.entry(name.clone()) {
                    Entry::Vacant(vacant) => { vacant.insert(state.clone()); },
                    Entry::Occupied(ref mut occupied) => (),
                }

                match self.value_scope.entry(name) {
                    Entry::Vacant(vacant) => { vacant.insert(state); },
                    Entry::Occupied(ref mut occupied) => (),
                }
            }
        }

    }

    fn process_module_item(&mut self, item: &ModuleItem) -> Result<(), BindGenError> {
        match item {

            ModuleItem::ModuleDecl(ref decl) => self.process_module_decl(decl),

            ModuleItem::Stmt(ref stmt) => self.process_stmt(stmt),
        }
    }

    fn process_stmt(&mut self, stmt: &Stmt) -> Result<(), BindGenError> {
        if let Stmt::Decl(ref decl) = stmt {
            self.process_decl(decl, false)?;
        }

        Ok(())
    }

    fn process_module_decl(&mut self, module_decl: &ModuleDecl) -> Result<(), BindGenError> {
        match module_decl {

            ModuleDecl::Import(ref import) => {
                let src_canon_path: &CanonPath =
                    get_dep_src!(self, import.src);

                for specifier in import.specifiers.iter() {
                    self.handle_import_specifier(src_canon_path, specifier)?;
                }

                Ok(())
            },

            ModuleDecl::ExportDecl(ExportDecl {
                ref decl,
                ..
            }) => self.process_decl(decl, true),

            ModuleDecl::ExportNamed(ref exp) => self.process_named_export(exp),

            ModuleDecl::ExportAll(ExportAll {
                ref src,
                ..
            }) => {
                let dep_canon_path = get_dep_src!(self, src);
                self.export_edges.push(Export::All {
                    source: dep_canon_path.clone(),
                });

                Ok(())
            }

            ModuleDecl::ExportDefaultDecl(..)
                => unreachable!("Caught in bind init"),

            ModuleDecl::ExportDefaultExpr(..)
                => unreachable!("Caught in bind init"),

            ModuleDecl::TsImportEquals(..)
                => unreachable!("Caught in bind init"),

            ModuleDecl::TsExportAssignment(..)
                => unreachable!("Caught in bind init"),

            ModuleDecl::TsNamespaceExport(..)
                => unreachable!("Caught in bind init"),
        }
    }

    fn prune_export_specifiers<'c, T>(&self, specifiers: T, exp_span: &Span)
        -> Result<Vec<&'c ExportSpecifier>, BindGenError>
            where T: Iterator<Item=&'c ExportSpecifier> {

        let mut buff = Vec::new();
        for spec in specifiers {
            match spec {
                ExportSpecifier::Named(..) => {
                    buff.push(spec);
                },

                ExportSpecifier::Namespace(NamespaceExportSpecifier {
                    ref span,
                    ..
                }) => {
                    return Err(BindGenError {
                        module_path: self.path.as_path().to_owned(),
                        kind: BindGenErrorKind::UnsupportedFeature(
                                  UnsupportedFeature::NamespaceExport),
                        span: span.clone(),
                    });
                }

                ExportSpecifier::Default(..) => {
                    return Err(BindGenError {
                        module_path: self.path.as_path().to_owned(),
                        kind: BindGenErrorKind::UnsupportedFeature(
                                  UnsupportedFeature::DefaultExport),
                        span: exp_span.clone(),
                    });
                }
            }
        }

        Ok(buff)
    }

    fn process_named_export(&mut self, exp: &NamedExport) -> Result<(), BindGenError> {
        let specifiers = self.prune_export_specifiers(exp.specifiers.iter(), &exp.span)?;

        match exp.src {
            Some(ref src) => {
                let src_canon_path: &CanonPath =
                    get_dep_src!(self, src);

                // Add export edges
                for spec in specifiers {

                    match spec {
                        ExportSpecifier::Named(NamedExportSpecifier {
                            ref orig,
                            exported: ref exported_as,
                            ..
                        }) => {

                            let orig_key = orig.sym.clone();
                            let export_key = exported_as
                                .as_ref()
                                .map(|x| x.sym.clone())
                                .unwrap_or(orig_key.clone());

                            self.export_edges.push(Export::Named {
                                source: src_canon_path.clone(),
                                src_key: orig_key,
                                export_key,
                            });
                        },

                        _ => unreachable!("Invalid specifier should be pruned"),
                    }
                }

                Ok(())
            }

            None => {
                for spec in specifiers {
                    match spec {
                        ExportSpecifier::Named(NamedExportSpecifier {
                            ref orig,
                            exported: ref exported_as,
                            ..
                        }) => {

                            let orig_key = orig.sym.clone();
                            let export_key = exported_as
                                .as_ref()
                                .map(|x| x.sym.clone())
                                .unwrap_or(orig_key.clone());

                            // Handle the named export if it refers to a rooted item or imported
                            //   item by adding an edge if it is an imported item
                            //   or by marking the item as rooted (under its export key)

                            // Handle value
                            if let Some(ref state) = self.value_scope.get(&orig_key) {
                                match state {
                                    ItemState::Imported {
                                        ref source,
                                        ref src_key,
                                        ref as_key,
                                    } => {
                                        self.export_edges.push(Export::Named {
                                            source: source.clone(),
                                            src_key: src_key.clone(),
                                            export_key: as_key.clone(),
                                        });
                                    }

                                    ItemState::Rooted {
                                        ref typ,
                                    } => {
                                        self.rooted_values.insert(export_key.clone(), typ.clone());
                                    }
                                }
                            }

                            // Handle value
                            if let Some(ref state) = self.type_scope.get(&orig_key) {
                                match state {
                                    ItemState::Imported {
                                        ref source,
                                        ref src_key,
                                        ref as_key,
                                    } => {
                                        self.export_edges.push(Export::Named {
                                            source: source.clone(),
                                            src_key: src_key.clone(),
                                            export_key: as_key.clone(),
                                        });
                                    }

                                    ItemState::Rooted {
                                        ref typ,
                                    }=> {
                                        self.rooted_types.insert(export_key, typ.clone());
                                    }
                                }
                            }
                        },

                        _ => unreachable!("Invalid specifier should be pruned"),
                    }
                }

                Ok(())
            }
        }
    }

    fn process_decl(&mut self, decl: &Decl, export: bool) -> Result<(), BindGenError> {
        let (symbol_type, scope_kind): (Vec<(JsWord, Type)>, ScopeKind) = match decl {
            Decl::Class(ClassDecl {
                ref ident,
                ..
            }) => {
                todo!("Generate type");
                (vec![(ident.sym.clone(), Type::Any)], ScopeKind::ValueType)
            },

            Decl::Fn(FnDecl {
                ident,
                ..
            }) => {
                todo!("Generate type");
                (vec![(ident.sym.clone(), Type::Any)], ScopeKind::Value)
            },

            Decl::Var(VarDecl {
                decls,
                ..
            }) => {
                todo!("Generate type");
                let mut symbols = Vec::new();
                decls.iter()
                    .for_each(|decl| {
                        match decl.name {
                            Pat::Ident(ref ident) => {
                                symbols.push((ident.sym.clone(), Type::Any));
                            },

                            _ => todo!("Handle all patterns"),
                        }
                    });

                (symbols, ScopeKind::Value)
            },

            Decl::TsInterface(TsInterfaceDecl {
                id,
                ..
            }) => {
                todo!("Generate type");
                (vec![(id.sym.clone(), Type::Any)], ScopeKind::Type)
            },

            Decl::TsTypeAlias(TsTypeAliasDecl {
                id,
                ..
            }) => {
                todo!("Generate type");
                (vec![(id.sym.clone(), Type::Any)], ScopeKind::Type)
            },

            Decl::TsEnum(TsEnumDecl {
                id,
                ..
            }) => {
                todo!("Generate type");
                (vec![(id.sym.clone(), Type::Any)], ScopeKind::Type)
            },

            Decl::TsModule(m) => {
                todo!("TS modules not suppported: {}::{:?}", self.path.as_path().display(), m.id);
            },
        };

        for (symbol, typ) in symbol_type.into_iter() {


            match scope_kind {
                ScopeKind::Value => {
                    if export {
                        self.rooted_values.insert(symbol.clone(), typ.clone());
                    }

                    self.scope_item(symbol, ItemState::Rooted {
                        typ,
                    }, scope_kind);
                }

                ScopeKind::Type => {
                    if export {
                        self.rooted_types.insert(symbol.clone(), typ.clone());
                    }
                    self.scope_item(symbol, ItemState::Rooted {
                        typ,
                    }, scope_kind);
                }

                ScopeKind::ValueType => {
                    if export {
                        self.rooted_types.insert(symbol.clone(), typ.clone());
                        self.rooted_values.insert(symbol.clone(), typ.clone());
                    }
                    self.scope_item(symbol, ItemState::Rooted {
                        typ,
                    }, scope_kind);
                }
            }
        }

        Ok(())
    }

    fn handle_import_specifier(&mut self, source: &CanonPath, spec: &ImportSpecifier)
        -> Result<(), BindGenError> {
        match spec {
            ImportSpecifier::Specific(ref specific) => {

                let src_key = specific
                    .imported
                    .as_ref()
                    .map(|export_key| export_key.sym.clone())
                    .unwrap_or(specific.local.sym.clone());

                let as_key = specific.local.sym.clone();

                self.import_edges.push(Import::Named {
                    source: source.clone(),
                    src_key: src_key.clone(),
                });

                let item = ItemState::Imported {
                    source: source.clone(),
                    src_key,
                    as_key,
                };

                let import_key = specific.local.sym.clone();
                self.scope_item(import_key, item, ScopeKind::ValueType);

                Ok(())
            }

            ImportSpecifier::Default(def) => {
                Err(BindGenError {
                    module_path: self.path.as_path().to_owned(),
                    kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultImport),
                    span: def.span,
                })
            }

            ImportSpecifier::Namespace(namespace) => {
                Err(BindGenError {
                    module_path: self.path.as_path().to_owned(),
                    kind: BindGenErrorKind::UnsupportedFeature(UnsupportedFeature::DefaultImport),
                    span: namespace.span,
                })
            }
        }
    }

    fn type_from_ann(
        &self,
        ann: TsTypeAnn,
    ) -> Result<Type, BindGenError> {
        let ann_span = ann.span;

        self.bind_type(*ann.type_ann)
    }

    fn bind_type(
        &self,
        typ: TsType,
    ) -> Result<Type, BindGenError> {

        match typ {
            TsType::TsKeywordType(TsKeywordType {
                span,
                kind,
            }) => {

                let prim_type = match kind {
                    TsKeywordTypeKind::TsAnyKeyword => Type::Any,
                    TsKeywordTypeKind::TsUnknownKeyword => todo!("unknown keyword type"),
                    TsKeywordTypeKind::TsNumberKeyword => Type::Number,
                    TsKeywordTypeKind::TsObjectKeyword => Type::Object,
                    TsKeywordTypeKind::TsBooleanKeyword => Type::Boolean,
                    TsKeywordTypeKind::TsBigIntKeyword => todo!("big int keyword type"),
                    TsKeywordTypeKind::TsStringKeyword => Type::String,
                    TsKeywordTypeKind::TsSymbolKeyword => todo!("symbol keyword type"),
                    TsKeywordTypeKind::TsVoidKeyword => Type::Void,
                    TsKeywordTypeKind::TsUndefinedKeyword => Type::Undefined,
                    TsKeywordTypeKind::TsNullKeyword => Type::Null,
                    TsKeywordTypeKind::TsNeverKeyword => Type::Never,
                };

                Ok(prim_type)
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

                Ok(Type::Any)
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
                let elem_type = Box::new(self.bind_type(*elem_type)?);
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
