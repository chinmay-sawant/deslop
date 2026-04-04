use std::path::Path;

use crate::Result;
use crate::index::RepositoryIndex;
use crate::model::Finding;

use super::config::AnalysisConfig;
use super::go;
use super::python;
use super::rust;
use super::types::ParsedFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Language {
    Go,
    Python,
    Rust,
}

pub(crate) trait LanguageBackend: Send + Sync {
    fn language(&self) -> Language;

    fn supported_extensions(&self) -> &'static [&'static str];

    fn supports_path(&self, path: &Path) -> bool;

    fn parse_file(&self, path: &Path, source: &str) -> Result<ParsedFile>;

    fn evaluate_file(
        &self,
        _file: &ParsedFile,
        _index: &RepositoryIndex,
        _analysis_config: &AnalysisConfig,
    ) -> Vec<Finding> {
        Vec::new()
    }

    fn evaluate_repo(
        &self,
        _files: &[&ParsedFile],
        _index: &RepositoryIndex,
        _analysis_config: &AnalysisConfig,
    ) -> Vec<Finding> {
        Vec::new()
    }
}

static GO_BACKEND: go::GoAnalyzer = go::GoAnalyzer;
static PYTHON_BACKEND: python::PythonAnalyzer = python::PythonAnalyzer;
static RUST_BACKEND: rust::RustAnalyzer = rust::RustAnalyzer;
static BACKENDS: [&dyn LanguageBackend; 3] = [&GO_BACKEND, &PYTHON_BACKEND, &RUST_BACKEND];

pub(crate) fn registered_backends() -> &'static [&'static dyn LanguageBackend] {
    &BACKENDS
}

pub(crate) fn backend_for_path(path: &Path) -> Option<&'static dyn LanguageBackend> {
    registered_backends()
        .iter()
        .find(|backend| backend.supports_path(path))
        .copied()
}

pub(crate) fn backend_for_language(language: Language) -> Option<&'static dyn LanguageBackend> {
    registered_backends()
        .iter()
        .find(|backend| backend.language() == language)
        .copied()
}

pub(crate) fn supported_extensions() -> Vec<&'static str> {
    let mut extensions = Vec::new();

    for &backend in registered_backends() {
        for extension in backend.supported_extensions() {
            if !extensions.contains(extension) {
                extensions.push(*extension);
            }
        }
    }

    extensions.sort_unstable();
    extensions
}
