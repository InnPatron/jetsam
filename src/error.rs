use std::error::Error;
use std::fmt::Display;

use swc_common::Span;

#[derive(Debug, Clone)]
pub struct BindGenError {
    pub kind: BindGenErrorKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum BindGenErrorKind {
    UnsupportedFeature(UnsupportedFeature),
}

#[derive(Debug, Clone)]
pub enum UnsupportedFeature {
    TsImportEquals,
    TsExportAssignment,
    TsNamespaceExport,
}
