use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use swc_common::{BytePos, Span, SyntaxContext};
use swc_ecma_ast::*;

use super::error::*;
use super::structures::CanonPath;

// TODO: Fix dependency resolution to match Node (and Typescript Node import option)
//    Helpful sources: https://www.typescriptlang.org/docs/handbook/module-resolution.html#node
pub fn locate_dependency(
    original: &Path,
    dependency: &Path,
) -> Result<Option<CanonPath>, BindGenError> {
    if dependency.is_relative() {
        let mut current_path = original.to_owned();
        current_path.pop();
        let mut current_path = current_path.join(dependency);
        prepare_path(&mut current_path);

        CanonPath::try_from(current_path.clone())
            .map_err(|e| BindGenError {
                module_path: current_path,
                kind: e.into(),
                span: Span::new(BytePos(0), BytePos(0), SyntaxContext::empty()),
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

pub fn get_decl_ident(decl: &Decl) -> &Ident {
    match decl {
        Decl::Class(ClassDecl { ref ident, .. }) => ident,
        Decl::Fn(FnDecl { ref ident, .. }) => ident,
        Decl::Var(..) => panic!("get_decl_ident() does not work on var"),
        Decl::TsInterface(TsInterfaceDecl { ref id, .. }) => id,
        Decl::TsTypeAlias(TsTypeAliasDecl { ref id, .. }) => id,
        Decl::TsEnum(TsEnumDecl { ref id, .. }) => id,
        Decl::TsModule(..) => panic!("get_decl_ident() does not work on TsModule"),
    }
}
