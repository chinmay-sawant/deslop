use std::path::PathBuf;

use crate::analysis::Language;
use crate::model::FunctionFingerprint;

use super::common::{
    CallSite, CommentSummary, DeclaredSymbol, ImportSpec, NamedLiteral, TestFunctionSummary,
    TopLevelBindingSummary, TopLevelCallSummary,
};
use super::go::{
    GoFunctionEvidence, GoFunctionEvidenceView, GoStructSummary, InterfaceSummary,
    PackageVarSummary, StructTag,
};
use super::python::{
    ClassSummary, PythonFunctionEvidence, PythonFunctionEvidenceView, PythonModelSummary,
};
use super::rust::{
    RustEnumSummary, RustFunctionEvidence, RustFunctionEvidenceView, RustStaticSummary,
    StructSummary,
};

#[derive(Debug, Clone)]
pub(crate) struct ParsedFile {
    pub language: Language,
    pub path: PathBuf,
    pub package_name: Option<String>,
    pub is_test_file: bool,
    pub syntax_error: bool,
    pub line_count: usize,
    pub byte_size: usize,
    pub pkg_strings: Vec<NamedLiteral>,
    pub comments: Vec<CommentSummary>,
    pub struct_tags: Vec<StructTag>,
    pub functions: Vec<ParsedFunction>,
    pub imports: Vec<ImportSpec>,
    pub symbols: Vec<DeclaredSymbol>,
    pub class_summaries: Vec<ClassSummary>,
    pub package_vars: Vec<PackageVarSummary>,
    pub interfaces: Vec<InterfaceSummary>,
    pub go_structs: Vec<GoStructSummary>,
    pub module_scope_calls: Vec<TopLevelCallSummary>,
    pub top_level_bindings: Vec<TopLevelBindingSummary>,
    pub python_models: Vec<PythonModelSummary>,
    pub rust_statics: Vec<RustStaticSummary>,
    pub rust_enums: Vec<RustEnumSummary>,
    // Rust heuristics consume conservative struct summaries so they can stay syntax-driven.
    pub structs: Vec<StructSummary>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedFunction {
    pub fingerprint: FunctionFingerprint,
    pub signature_text: String,
    pub body_start_line: usize,
    pub calls: Vec<CallSite>,
    pub is_test_function: bool,
    pub local_binding_names: Vec<String>,
    pub doc_comment: Option<String>,
    pub body_text: String,
    pub local_strings: Vec<NamedLiteral>,
    pub test_summary: Option<TestFunctionSummary>,
    // Language-specific evidence — exactly one of these is populated per function.
    pub go: Option<GoFunctionEvidence>,
    pub python: Option<PythonFunctionEvidence>,
    pub rust: Option<RustFunctionEvidence>,
}

impl ParsedFunction {
    pub(crate) fn go_evidence(&self) -> GoFunctionEvidenceView<'_> {
        self.go
            .as_ref()
            .map(|ev| ev.as_view())
            .unwrap_or_else(GoFunctionEvidenceView::empty)
    }

    pub(crate) fn python_evidence(&self) -> PythonFunctionEvidenceView<'_> {
        self.python
            .as_ref()
            .map(|ev| ev.as_view())
            .unwrap_or_else(PythonFunctionEvidenceView::empty)
    }

    pub(crate) fn rust_evidence(&self) -> RustFunctionEvidenceView<'_> {
        self.rust
            .as_ref()
            .map(|ev| ev.as_view())
            .unwrap_or_else(RustFunctionEvidenceView::empty)
    }
}
