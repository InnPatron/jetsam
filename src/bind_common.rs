use std::path::{Path, PathBuf};
use std::convert::TryFrom;

use swc_common::{BytePos, SyntaxContext, Span};

use super::structures::CanonPath;
use super::error::*;

// TODO: Fix dependency resolution to match Node (and Typescript Node import option)
//    Helpful sources: https://www.typescriptlang.org/docs/handbook/module-resolution.html#node
pub fn locate_dependency(original: &Path, dependency: &Path) -> Result<Option<CanonPath>, BindGenError> {

    if dependency.is_relative() {
        let mut current_path = original.to_owned();
        current_path.pop();
        let mut current_path = current_path.join(dependency);
        prepare_path(&mut current_path);

        CanonPath::try_from(current_path)
            .map_err(|e| {
                BindGenError {
                    module_path: original.to_owned(),
                    kind: e.into(),
                    span: Span::new(BytePos(0), BytePos(0), SyntaxContext::empty()),
                }
            })
        .map(|path| Some(path))

    } else {
        todo!("Other module resolution");
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
