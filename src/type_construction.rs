use std::collections::HashMap;

use swc_atoms::JsWord;
use swc_ecma_ast::*;

use super::bind_init::ModuleData;

use super::type_structs::*;
use super::structures::{ ItemState, Scope, CanonPath };
use super::error::*;

///
/// Assumes type_scope is fully initialized (via init_type_scop::init())
///
pub fn construct_variable_types(
    current_module: &CanonPath,
    type_scope: &Scope<ItemState>,
    decl: &VarDecl
) -> Result<(), BindGenError> {
    todo!("Variable type construction");
}

///
/// Assumes type_scope is fully initialized (via init_type_scop::init())
///
pub fn construct_fn_type(
    current_module: &CanonPath,
    type_scope: &Scope<ItemState>,
    function: &Function
) -> Result<Type, BindGenError> {

    let session = Session {
        path: current_module,
        scope: type_scope,
        self_id: None,
    };

    session.gen_fn_type(function)
}

///
/// Assumes type_scope is fully initialized (via init_type_scop::init())
///
pub fn construct_type(
    current_module: &CanonPath,
    type_scope: &Scope<ItemState>,
    decl: &Decl
) -> Result<Type, BindGenError> {

    let self_id = get_type_name(decl);

    let session = Session {
        path: current_module,
        scope: type_scope,
        self_id,
    };

    match decl {
        Decl::Class(ref decl) => session.gen_class_type(decl),

        Decl::TsInterface(ref decl) => session.gen_interface_type(decl),

        Decl::TsTypeAlias(ref alias) => session.bind_type(&*alias.type_ann),

        Decl::TsEnum(TsEnumDecl {
            id,
            ..
        }) => {
            let typ = Type::Opaque {
                name: id.sym.clone(),
                origin: session.path.clone(),
            };

            Ok(typ)
        },

        _ => unreachable!(),
    }
}

fn get_type_name(decl: &Decl) -> Option<&JsWord> {
    match decl {
        Decl::Class(ref decl)       => Some(&decl.ident.sym),
        Decl::TsInterface(ref decl) => Some(&decl.id.sym),
        Decl::TsTypeAlias(ref decl) => Some(&decl.id.sym),
        Decl::TsEnum(ref decl)      => Some(&decl.id.sym),

        _                           => None,
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

struct Session<'a> {
    path: &'a CanonPath,
    self_id: Option<&'a JsWord>,
    scope: &'a Scope<ItemState>,
}

impl<'a> Session<'a> {

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
