use std::collections::{HashMap, HashSet};

use swc_ecma_ast::*;
use swc_atoms::JsWord;
use swc_common::Span;

use super::bind_init::{ModuleData, ParsedModuleCache as ModuleCache};
use super::structures::{Type, CanonPath, FnType, ClassType};
use super::error::*;
use super::bind_graph_init::{
    ModuleGraph as UTModuleGraph,
    ModuleNode as UTModuleNode,
    Import,
    Export,
    ScopeKind,
};

pub fn typify(cache: &ModuleCache, ut_graph: UTModuleGraph) -> Result<ModuleGraph, BindGenError> {
    let mut graph = ModuleGraph {
        nodes: HashMap::new(),
        export_edges: ut_graph.export_edges,
        import_edges: ut_graph.import_edges,
    };

    for (_, module_data) in cache.iter() {
        NodeInitSession::init(&mut graph, cache, module_data)?;
    }

    Ok(graph)
}

pub struct ModuleNode {
    pub path: CanonPath,
    pub rooted_export_types: HashMap<JsWord, Type>,
    pub rooted_export_values: HashMap<JsWord, Type>,
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

struct NodeInitSession<'a> {
    path: &'a CanonPath,
    dependency_map: &'a HashMap<String, CanonPath>,
    cache: &'a ModuleCache,
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
            cache,

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

        let module_node = ModuleNode {
            path: module_data.path.clone(),
            rooted_export_types,
            rooted_export_values,
        };

        g.nodes.insert(module_data.path.clone(), module_node);

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
            Some(ref src) => Ok(()),

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
                                    } => (),

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
                                    } => (),

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
            Decl::Class(ref decl) => {
                let typ = self.gen_class_type(decl)?;
                (vec![(decl.ident.sym.clone(), typ)], ScopeKind::ValueType)
            },

            Decl::Fn(ref decl) => {
                let typ = self.gen_fn_type(&decl.function)?;
                (vec![(decl.ident.sym.clone(), typ)], ScopeKind::Value)
            },

            Decl::Var(VarDecl {
                decls,
                ..
            }) => {
                let mut symbols = Vec::new();

                for decl in decls.iter() {
                    match decl.name {
                        Pat::Ident(ref ident) => {
                            let typ = ident.type_ann
                                .as_ref()
                                .map(|ann| self.type_from_ann(ann))
                                .transpose()?
                                .unwrap_or(Type::Any);
                            symbols.push((ident.sym.clone(), typ));
                        },

                        _ => todo!("Handle all patterns"),
                    }
                }

                (symbols, ScopeKind::Value)
            },

            Decl::TsInterface(ref decl) => {
                let typ = self.gen_interface_type(decl)?;
                (vec![(decl.id.sym.clone(), typ)], ScopeKind::Type)
            },

            Decl::TsTypeAlias(ref alias) => {
                let typ = self.bind_type(&*alias.type_ann)?;
                (vec![(alias.id.sym.clone(), typ)], ScopeKind::Type)
            },

            Decl::TsEnum(TsEnumDecl {
                id,
                ..
            }) => {
                let typ = Type::Opaque {
                    name: id.sym.clone(),
                    origin: self.path.clone(),
                };
                (vec![(id.sym.clone(), typ)], ScopeKind::Type)
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

    fn gen_fn_type(
        &self,
        function: &Function
    ) -> Result<Type, BindGenError> {
        let mut params: Vec<Type> = Vec::new();

        let return_type = function.return_type
            .as_ref()
            .map(|ann| self.type_from_ann(ann))
            .transpose()?
            .unwrap_or(Type::Any);

        for param in function.params.iter() {
            let ann = ann_from_pat(param);

            let typ = ann
                .map(|ann| self.type_from_ann(ann))
                .transpose()?
                .unwrap_or(Type::Any);

            params.push(typ);
        }

        Ok(Type::Fn(FnType {
            params,
            return_type: Box::new(return_type),
        }))

    }

    fn gen_class_type(
        &self,
        decl: &ClassDecl
    ) -> Result<Type, BindGenError> {

        // TODO: Type parameters
        let mut members: HashMap<JsWord, Type> = HashMap::new();
        let mut constructors: Vec<FnType> = Vec::new();

        for class_member in decl.class.body.iter() {
            match class_member {
                ClassMember::ClassProp(ref prop) => {
                    match *prop.key {
                        Expr::Ident(ref ident) => {
                            let typ = ident.type_ann
                                .as_ref()
                                .map(|ann| self.type_from_ann(ann))
                                .transpose()?
                                .unwrap_or(Type::Any);

                            members.insert(ident.sym.clone(), typ);
                        }

                        _ => todo!("Unsupported key expr (not ident)"),
                    }
                }

                ClassMember::Constructor(ref constructor) => {
                    let mut params = Vec::new();
                    for param in constructor.params.iter() {
                        let ann = match param {
                            PatOrTsParamProp::Pat(ref pat) => ann_from_pat(pat),
                            PatOrTsParamProp::TsParamProp(..) => todo!("TsParamProp"),
                        };

                        let param_typ = ann
                            .map(|ann| self.type_from_ann(ann))
                            .transpose()?
                            .unwrap_or(Type::Any);
                        params.push(param_typ);
                    }
                    constructors.push(FnType {
                        params,
                        return_type: Box::new(Type::Any),
                    });
                }

                ClassMember::Method(ref method) => {
                    // TODO: Self parameter
                    let typ = self.gen_fn_type(&method.function)?;
                    let key = match method.key {
                        PropName::Ident(ref ident) => ident.sym.clone(),

                        ref x => todo!("Unsupported prop name kind: {:?}", x),
                    };

                    members.insert(key, typ);
                }

                x => todo!("Handle ClassMember: {:?}", x),
            }

        }

        Ok(Type::Class(ClassType {
            name: decl.ident.sym.clone(),
            origin: self.path.clone(),
            constructors,
            members,
        }))
    }

    fn gen_type_element<F>(
        &self,
        element: &TsTypeElement,
        mut f: F
    ) -> Result<(), BindGenError>
    where F: FnMut(JsWord, Type) -> () {
        match element {
            TsTypeElement::TsPropertySignature(ref signature) => {
                let ident = ident_from_key(&*signature.key);
                let typ = ident.type_ann
                    .as_ref()
                    .map(|ann| self.type_from_ann(ann))
                    .transpose()?
                    .unwrap_or(Type::Any);

                f(ident.sym.clone(), typ);
                Ok(())
            }

            // TODO: Log that TsIndexSignature was skipped
            TsTypeElement::TsIndexSignature(..) => Ok(()),

            TsTypeElement::TsMethodSignature(ref signature) => {
                let ident = ident_from_key(&*signature.key);
                let return_type = signature.type_ann
                    .as_ref()
                    .map(|ann| self.type_from_ann(ann))
                    .transpose()?
                    .unwrap_or(Type::Any);

                let params = signature.params.iter()
                    .map(|fn_param| {
                        let ann = ann_from_fn_param(fn_param);
                        Ok(ann
                            .map(|ann| self.type_from_ann(ann))
                            .transpose()?
                            .unwrap_or(Type::Any))
                    })
                .collect::<Result<Vec<Type>, _>>()?;

                let typ = Type::Fn(FnType {
                    params,
                    return_type: Box::new(return_type),
                });

                f(ident.sym.clone(), typ);

                Ok(())
            }

            ref x => todo!("[{:?}]Handle TsTypeElement: {:?}", self.path.as_path().display(), x),
        }
    }

    fn gen_interface_type(
        &self,
        decl: &TsInterfaceDecl
    ) -> Result<Type, BindGenError> {

        // TODO: Type parameters
        let mut fields: HashMap<JsWord, Type> = HashMap::new();

        for ts_type_element in decl.body.body.iter() {
            self.gen_type_element(ts_type_element,
                |sym, typ| {
                    fields.insert(sym, typ);

                })?;
        }

        Ok(Type::Interface {
            name: decl.id.sym.clone(),
            origin: self.path.clone(),
            fields,
        })
    }

    fn type_from_ann(
        &self,
        ann: &TsTypeAnn,
    ) -> Result<Type, BindGenError> {
        let ann_span = ann.span;

        self.bind_type(&ann.type_ann)
    }

    fn bind_type(
        &self,
        typ: &TsType,
    ) -> Result<Type, BindGenError> {

        match typ {
            TsType::TsKeywordType(TsKeywordType {
                ref span,
                ref kind,
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
                ref span,
            }) => {
                // TODO: What is TsThisType?
                //   `this` type is used for class members and refers to the class

                Ok(Type::Any)
            },

            TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsFnType(TsFnType {
                ref span,
                ref params,
                ref type_params,
                ref type_ann,
            })) => {
                // TODO: Is type_ann the return type?

                let mut new_params = Vec::new();
                for param in params {
                    let ann = ann_from_fn_param(param);

                    let typ = ann
                        .map(|ann| self.type_from_ann(ann))
                        .transpose()?
                        .unwrap_or(Type::Any);

                    new_params.push(typ);
                }

                Ok(Type::Fn(FnType {
                    params: new_params,
                    return_type: Box::new(Type::Any),
                }))
            },

            TsType::TsFnOrConstructorType(TsFnOrConstructorType::TsConstructorType(TsConstructorType {
                ref span,
                ref params,
                ref type_params,
                ref type_ann,
            })) => {
                // What is type_ann
                // Is type_ann the return type?
                todo!("ts constructor");
            },

            TsType::TsTypeRef(TsTypeRef {
                ref span,
                ref type_name,
                ref type_params,
            }) => {
                // todo!("{}:{:?}", module_info.path().display(), type_name);
                // TODO: TsTypeRef

                Ok(Type::Any)
            },

            TsType::TsTypeQuery(_TsTypeQuery) => {
                todo!("ts type query");
            },

            TsType::TsTypeLit(ref lit) => {

                let mut fields = HashMap::new();
                for type_element in lit.members.iter() {
                    self.gen_type_element(type_element,
                        |sym, typ| { fields.insert(sym, typ); })?;
                }

                Ok(Type::Literal {
                    fields,
                })
            },

            TsType::TsArrayType(TsArrayType {
                ref span,
                ref elem_type,
            }) => {
                let elem_type = Box::new(self.bind_type(elem_type)?);
                Ok(Type::UnsizedArray(elem_type))
            },

            TsType::TsTupleType(TsTupleType {
                ref span,
                ref elem_types,
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
                ref span,
                ref types,
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
                ref span,
                ref type_ann,
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
                ref span,
                ref lit,
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

fn ann_from_pat(p: &Pat) -> Option<&TsTypeAnn> {
    match p {
        Pat::Ident(ref pat) => pat.type_ann.as_ref(),
        Pat::Array(ref pat) => pat.type_ann.as_ref(),
        Pat::Rest(ref pat) => pat.type_ann.as_ref(),
        Pat::Object(ref pat) => pat.type_ann.as_ref(),
        Pat::Assign(ref pat) => pat.type_ann.as_ref(),
        Pat::Invalid(..) => None,
        Pat::Expr(..) => None,
    }
}

fn ann_from_fn_param(p: &TsFnParam) -> Option<&TsTypeAnn> {
    match p {
        TsFnParam::Ident(ref pat) => pat.type_ann.as_ref(),
        TsFnParam::Array(ref pat) => pat.type_ann.as_ref(),
        TsFnParam::Object(ref pat) => pat.type_ann.as_ref(),
        TsFnParam::Rest(ref pat) => pat.type_ann.as_ref(),
    }
}

fn ident_from_key(key: &Expr) -> &Ident {
    match key {
        Expr::Ident(ref ident) => ident,

        _ => todo!("Unsupported key expr (not ident)"),
    }
}