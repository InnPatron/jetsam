use swc_atoms::JsWord;

use crate::ts::{ TsFlavorBuilder, TsFlavor };

use super::structures::CanonPath;
use super::typify_graph::{ ModuleGraph, ModuleNode };
use super::type_structs::{ Type, FnType, ClassType };

macro_rules! basic_scan {
    ($builder: expr => $field: ident) => {{
        $builder.$field(true);
    }}

}

pub fn detect(graph: &ModuleGraph) -> TsFlavor {

    let mut builder = TsFlavorBuilder::empty();
    todo!("Detect input TS flavor");

    builder.build()
        .expect("TS detection failed")
}

fn scan_type<'a, 'b, 'c>(
    builder: &'a mut TsFlavorBuilder,
    graph: &'b ModuleGraph,
    typ: &'b Type,
    parent_types: &'c mut Vec<(&'b JsWord, &'b CanonPath)> ) {
    match typ {

        Type::Named {
            ref name,
            ref source,
        } => {
            if parent_types.contains(&(name, source)) {
                builder.recursive_type(true);
            }
        }

        Type::Fn(FnType {
            ref params,
            ref return_type,
        }) => {

            for param_type in params {
                scan_type(builder, graph, param_type, parent_types);
            }

            scan_type(builder, graph, &*return_type, parent_types);

            builder.fn_type(true);
        }

        Type::Interface {
            ref name,
            ref origin,
            ref fields,
            ..
        } => {
            parent_types.push((name, origin));
            for field_type in fields.values() {
                scan_type(builder, graph, field_type, parent_types);
            }
            parent_types.pop();

            builder.interfaces(true);
        }

        Type::Literal {
            ref fields,
        } => {
            for field_type in fields.values() {
                scan_type(builder, graph, field_type, parent_types);
            }

            builder.type_literal(true);
        }

        Type::UnsizedArray(ref elem_type) => {
            scan_type(builder, graph, elem_type, parent_types);
            builder.array_type(true);
        }

        Type::Number    => basic_scan!(builder => number_type),
        Type::Boolean   => basic_scan!(builder => boolean_type),
        Type::String    => basic_scan!(builder => string_type),
        Type::Void      => basic_scan!(builder => void_type),
        Type::Any       => basic_scan!(builder => any_type),
        Type::Object    => basic_scan!(builder => object_type),
        Type::Undefined => basic_scan!(builder => undefined_type),
        Type::Null      => basic_scan!(builder => null_type),
        Type::Never     => basic_scan!(builder => never_type),


        t => todo!("Scan {:?}", t),
    };
}
