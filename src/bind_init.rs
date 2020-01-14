use std::collections::HashMap;
use std::path::PathBuf;

use swc_ecma_parser::Session;
use swc_ecma_ast::Module;

use super::structures::CanonPath;
use super::error::BindGenError;

pub struct ModuleData {
    pub path: CanonPath,
    pub module_ast: Module,
}


pub fn init<'a>(root_module_path: PathBuf, session: Session<'a>) -> Result<(), BindGenError> {
    let mut module_cache: HashMap<CanonPath, Module> = HashMap::new();

    todo!();
}

