use std::hash::Hash;
use std::collections::HashMap;
use std::path::PathBuf;

use swc_atoms::JsWord;
use swc_common::Span;
use swc_ecma_ast::Str;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonPath(PathBuf);

impl CanonPath {
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }
}

impl From<CanonPath> for PathBuf {
    fn from(p: CanonPath) -> PathBuf {
        p.0
    }
}

impl std::convert::TryFrom<PathBuf> for CanonPath {
    type Error = std::io::Error;

    fn try_from(p: PathBuf) -> Result<Self, Self::Error> {
        p.canonicalize().map(|p| CanonPath(p))
    }
}

impl<'a> std::convert::TryFrom<&'a std::path::Path> for CanonPath {
    type Error = std::io::Error;

    fn try_from(p: &std::path::Path) -> Result<Self, Self::Error> {
        p.canonicalize().map(|p| CanonPath(p))
    }
}
