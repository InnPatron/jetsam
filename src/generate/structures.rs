use std::hash::Hash;
use std::collections::HashMap;
use std::path::PathBuf;

use swc_atoms::JsWord;

use super::type_structs::Type;

pub struct Scope<T> {
    map: HashMap<JsWord, T>,
}

impl<T> Scope<T> {
    pub fn new() -> Self {
        Scope {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: JsWord, v: T) {
        self.map.insert(key, v);
    }

    pub fn get(&self, key: &JsWord) -> Option<&T> {
        self.map.get(key)
    }
}

#[derive(Clone)]
pub enum ItemStateT {
    Imported {
        source: CanonPath,
        src_key: JsWord,
        as_key: JsWord,
    },

    Rooted(Type),
}

#[derive(Clone)]
pub enum ItemState {
    Imported {
        source: CanonPath,
        src_key: JsWord,
        as_key: JsWord,
    },

    Rooted,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonPath(PathBuf);

impl CanonPath {
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }

    #[cfg(test)]
    pub fn mock(b: PathBuf) -> Self {
        CanonPath(b)
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
