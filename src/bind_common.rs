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

        CanonPath::try_from(current_path.clone())
            .map_err(|e| {
                BindGenError {
                    module_path: current_path,
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

    match path.extension() {
        Some(os_str_ext) => {
            if os_str_ext != "d.ts" {
                let mut ext = os_str_ext.to_os_string();
                ext.push(".d.ts");
                path.set_extension(ext);
            }
        }

        None => {
            path.set_extension("d.ts");
        }
    }
}
