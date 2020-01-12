use std::error::Error;
use std::io::Error as IoError;
use std::path::PathBuf;
use std::fmt::Display;

use swc_common::Span;

#[derive(Debug)]
pub struct BindGenError {
    pub kind: BindGenErrorKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum BindGenErrorKind {
    UnsupportedFeature(UnsupportedFeature),
    IoError(PathBuf, IoError),
    ParserError,
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
}
