use crate::ts::{ TsFlavorBuilder, TsFlavor };

use super::typify_graph::{ ModuleGraph, ModuleNode };
use super::type_structs::Type;

pub fn detect(graph: &ModuleGraph) -> TsFlavor {

    let mut builder = TsFlavorBuilder::empty();
    todo!("Detect input TS flavor");

    builder.build()
        .expect("TS detection failed")
}
