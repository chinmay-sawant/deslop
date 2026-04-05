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
    RustAttributeSummary, RustEnumSummary, RustFunctionEvidence, RustFunctionEvidenceView,
    RustModuleDeclaration, RustStaticSummary, StructSummary,
};

/// Common file-level data shared across all languages.
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
    pub functions: Vec<ParsedFunction>,
    pub imports: Vec<ImportSpec>,
    pub symbols: Vec<DeclaredSymbol>,
    pub top_level_bindings: Vec<TopLevelBindingSummary>,
    pub module_scope_calls: Vec<TopLevelCallSummary>,
    /// Language-specific file-level data. Exactly one variant is populated.
    pub lang: LanguageFileData,
}

/// Language-specific file-level data. Prevents storing empty vectors for
/// languages that don't use a given field.
#[derive(Debug, Clone)]
pub(crate) enum LanguageFileData {
    Go(GoFileData),
    Python(PythonFileData),
    Rust(RustFileData),
}

/// Go-specific file-level data.
#[derive(Debug, Clone, Default)]
pub(crate) struct GoFileData {
    pub struct_tags: Vec<StructTag>,
    pub package_vars: Vec<PackageVarSummary>,
    pub interfaces: Vec<InterfaceSummary>,
    pub go_structs: Vec<GoStructSummary>,
}

/// Python-specific file-level data.
#[derive(Debug, Clone, Default)]
pub(crate) struct PythonFileData {
    pub class_summaries: Vec<ClassSummary>,
    pub python_models: Vec<PythonModelSummary>,
}

/// Rust-specific file-level data.
#[derive(Debug, Clone, Default)]
pub(crate) struct RustFileData {
    pub rust_statics: Vec<RustStaticSummary>,
    pub rust_enums: Vec<RustEnumSummary>,
    pub structs: Vec<StructSummary>,
    pub attributes: Vec<RustAttributeSummary>,
    pub module_declarations: Vec<RustModuleDeclaration>,
}

impl ParsedFile {
    /// Access Go-specific file data. Returns `None` if the file is not Go.
    pub(crate) fn go_data(&self) -> Option<&GoFileData> {
        match &self.lang {
            LanguageFileData::Go(data) => Some(data),
            _ => None,
        }
    }

    /// Access Python-specific file data. Returns `None` if the file is not Python.
    pub(crate) fn python_data(&self) -> Option<&PythonFileData> {
        match &self.lang {
            LanguageFileData::Python(data) => Some(data),
            _ => None,
        }
    }

    /// Access Rust-specific file data. Returns `None` if the file is not Rust.
    pub(crate) fn rust_data(&self) -> Option<&RustFileData> {
        match &self.lang {
            LanguageFileData::Rust(data) => Some(data),
            _ => None,
        }
    }

    // ── Backward-compatible accessors ──
    // These delegate to the language envelope, returning empty slices for mismatches.

    pub(crate) fn struct_tags(&self) -> &[StructTag] {
        self.go_data().map_or(&[], |d| &d.struct_tags)
    }

    pub(crate) fn package_vars(&self) -> &[PackageVarSummary] {
        self.go_data().map_or(&[], |d| &d.package_vars)
    }

    pub(crate) fn interfaces(&self) -> &[InterfaceSummary] {
        self.go_data().map_or(&[], |d| &d.interfaces)
    }

    pub(crate) fn go_structs(&self) -> &[GoStructSummary] {
        self.go_data().map_or(&[], |d| &d.go_structs)
    }

    pub(crate) fn class_summaries(&self) -> &[ClassSummary] {
        self.python_data().map_or(&[], |d| &d.class_summaries)
    }

    pub(crate) fn python_models(&self) -> &[PythonModelSummary] {
        self.python_data().map_or(&[], |d| &d.python_models)
    }

    pub(crate) fn rust_statics(&self) -> &[RustStaticSummary] {
        self.rust_data().map_or(&[], |d| &d.rust_statics)
    }

    pub(crate) fn rust_enums(&self) -> &[RustEnumSummary] {
        self.rust_data().map_or(&[], |d| &d.rust_enums)
    }

    pub(crate) fn structs(&self) -> &[StructSummary] {
        self.rust_data().map_or(&[], |d| &d.structs)
    }

    pub(crate) fn rust_attributes(&self) -> &[RustAttributeSummary] {
        self.rust_data().map_or(&[], |d| &d.attributes)
    }

    pub(crate) fn rust_module_declarations(&self) -> &[RustModuleDeclaration] {
        self.rust_data().map_or(&[], |d| &d.module_declarations)
    }
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
