use std::hash::Hash;
use std::collections::HashMap;
use std::path::PathBuf;

use swc_common::Span;
use swc_ecma_ast::Str;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ValueId(pub u64);

pub struct ModuleInfo {
    path: PathBuf,
    owned_values: HashMap<ValueId, Type>,
    owned_types: HashMap<TypeId, Type>,
    exported_types: HashMap<String, TypeId>,
    exported_values: HashMap<String, ValueId>,
}

impl ModuleInfo {
    pub fn new(path: PathBuf) -> Self {
        ModuleInfo {
            owned_values: HashMap::new(),
            owned_types: HashMap::new(),
            exported_types: HashMap::new(),
            exported_values: HashMap::new(),

            path,
        }
    }

    pub fn register_value(&mut self, id: ValueId, value_type: Type) {
        if self.owned_values.insert(id, value_type).is_some() {
            panic!("Overriding value id '{:?}'", id);
        }
    }

    pub fn register_type(&mut self, id: TypeId, typ: Type) {
        if self.owned_types.insert(id, typ).is_some() {
            panic!("Overriding type id '{:?}'", id);
        }
    }

    pub fn export_value(&mut self, export_key: String, id: ValueId) {
        self.exported_values.insert(export_key, id);
    }

    pub fn export_type(&mut self, export_key: String, id: TypeId) {
        self.exported_types.insert(export_key, id);
    }

    pub fn merge_export(&mut self, other: &Self, other_key: String, as_key: Option<String>) {

        let value_item: Option<ValueId> = other.exported_values
            .get(&other_key)
            .map(|id| id.clone());

        let type_item: Option<TypeId> = other.exported_types
            .get(&other_key)
            .map(|id| id.clone());

        if let Some(value_id) = value_item {
            let value_type = other
                .owned_values
                .get(&value_id)
                .expect("source missing an exported value");

            self.register_value(value_id, value_type.clone());

            if let Err(bad_id) = ModuleInfo::merge_item(
                &mut self.owned_types,
                &other.owned_types,
                value_type) {
                panic!("Overriding type id '{:?}'", bad_id);
            }
        }

        if let Some(type_id) = type_item {
            let typ = other
                .owned_types
                .get(&type_id)
                .expect("source missing an exported type");

            self.register_type(type_id, typ.clone());

            if let Err(bad_id) = ModuleInfo::merge_item(
                &mut self.owned_types,
                &other.owned_types,
                typ) {
                panic!("Overriding type id '{:?}'", bad_id);
            }
        }
    }

    fn merge_item(deposit: &mut HashMap<TypeId, Type>, source: &HashMap<TypeId, Type>, root: &Type)
        -> Result<(), TypeId> {

        fn add_to_work_stack(stack: &mut Vec<TypeId>, typ: &Type) {
            match typ {
                Type::Fn { ref type_signature, .. } => {
                    type_signature.params.iter()
                        .for_each(|typ| add_to_work_stack(stack, typ));

                    type_signature.return_type
                        .iter()
                        .map(|typ| add_to_work_stack(stack, typ));
                }

                Type::Class {
                    ref constructor,
                    ref fields,
                    ..
                } => {
                    add_to_work_stack(stack, &constructor);
                    fields.iter()
                        .for_each(|(_, typ)| add_to_work_stack(stack, typ));
                }

                Type::Interface {
                    ref fields,
                    ..
                } => {
                    fields.iter()
                        .for_each(|(_, typ)| add_to_work_stack(stack, typ));
                }

                Type::Array(ref element_type, _) => {
                    add_to_work_stack(stack, element_type);
                }

                Type::TypeId(ref type_id) => {
                    stack.push(type_id.clone());
                }

                Type::Primitive(..) => (),
            }
        }

        let mut work_stack = Vec::new();

        add_to_work_stack(&mut work_stack, root);

        while work_stack.is_empty() == false {
            let next_id = work_stack.pop().unwrap();

            if deposit.contains_key(&next_id) {
                continue;
            }

            let to_insert = source.get(&next_id)
                .expect("Missing value for id in source module");

            add_to_work_stack(&mut work_stack, to_insert);

            if deposit.insert(next_id, to_insert.clone()).is_some() {
                return Err(next_id);
            }
        }

        Ok(())
    }

    pub fn merge_all(&mut self, other: &Self) {
        // Merge owned values
        for (value_id, value_type) in other.owned_values.iter() {
            self.owned_values.insert(value_id.clone(), value_type.clone());
        }

        // Merge owned types
        for (type_id, typ) in other.owned_types.iter() {
            self.owned_types.insert(type_id.clone(), typ.clone());
        }

        // Merge exports
        for (export_key, typ) in other.exported_types.iter() {
            self.exported_types.insert(export_key.clone(), typ.clone());
        }

        for (export_key, value) in other.exported_values.iter() {
            self.exported_values.insert(export_key.clone(), value.clone());
        }
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Fn {
        origin: Str,
        type_signature: FnType,
    },
    Class {
        name: Str,
        origin: Str,
        constructor: Box<Type>,
        fields: HashMap<String, Type>,
    },
    Interface {
        origin: Str,
        fields: HashMap<String, Type>,
    },
    Array(Box<Type>, usize),
    Primitive(PrimitiveType),
    TypeId(TypeId),
}

#[derive(Debug, Clone)]
pub enum PrimitiveType {
    Boolean,
    Number,
    String,
    Void,
    Object,
    Any,
    Never,
}

#[derive(Debug, Clone)]
pub struct FnType {
    params: Vec<Type>,
    return_type: Option<Box<Type>>,
}
