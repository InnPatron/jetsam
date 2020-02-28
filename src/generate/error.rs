use std::io::Error as IoError;
use std::path::PathBuf;

use swc_common::Span;
use serde_json::error::Error as JsonError;

#[derive(Debug)]
pub struct BindGenError {
    pub kind: BindGenErrorKind,
    pub module_path: PathBuf,
    pub span: Span,
}

#[derive(Debug)]
pub enum BindGenErrorKind {
    UnsupportedFeature(UnsupportedFeature),
    IoError(IoError),
    ParserError,
}

impl From<IoError> for BindGenErrorKind {
    fn from(v: IoError) -> Self {
        BindGenErrorKind::IoError(v)
    }
}

#[derive(Debug)]
pub enum UnsupportedFeature {
    NamespaceImport,
    DefaultImport,
    NamespaceExport,
    DefaultExport,
    TsImportEquals,
    TsExportAssignment,
    TsNamespaceExport,
}

#[derive(Debug)]
pub enum EmitError {
    IoError(PathBuf, IoError),
    JsonError(PathBuf, JsonError),
}
