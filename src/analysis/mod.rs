mod go;
mod python;
mod rust;
mod types;

use std::path::Path;

use anyhow::Result;

use crate::index::RepositoryIndex;
use crate::model::Finding;

pub(crate) use types::{
    CallSite, ContextFactoryCall, DbQueryCall, DeclaredSymbol, FormattedErrorCall, ImportSpec,
    NamedLiteral, ParsedFile, ParsedFunction, StructTag, TestFunctionSummary,
};

#[allow(dead_code)]
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

    fn evaluate_file(&self, _file: &ParsedFile, _index: &RepositoryIndex) -> Vec<Finding> {
        Vec::new()
    }

    fn evaluate_repo(&self, _files: &[&ParsedFile], _index: &RepositoryIndex) -> Vec<Finding> {
        Vec::new()
    }
}

pub(crate) fn registered_backends() -> [&'static dyn LanguageBackend; 3] {
    static GO_BACKEND: go::GoAnalyzer = go::GoAnalyzer;
    static PYTHON_BACKEND: python::PythonAnalyzer = python::PythonAnalyzer;
    static RUST_BACKEND: rust::RustAnalyzer = rust::RustAnalyzer;

    [&GO_BACKEND, &PYTHON_BACKEND, &RUST_BACKEND]
}

pub(crate) fn backend_for_path(path: &Path) -> Option<&'static dyn LanguageBackend> {
    registered_backends()
        .into_iter()
        .find(|backend| backend.supports_path(path))
}

pub(crate) fn backend_for_language(language: Language) -> Option<&'static dyn LanguageBackend> {
    registered_backends()
        .into_iter()
        .find(|backend| backend.language() == language)
}

pub(crate) fn supported_extensions() -> Vec<&'static str> {
    let mut extensions = Vec::new();

    for backend in registered_backends() {
        for extension in backend.supported_extensions() {
            if !extensions.contains(extension) {
                extensions.push(*extension);
            }
        }
    }

    extensions.sort_unstable();
    extensions
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Language, backend_for_path, supported_extensions};

    #[test]
    fn test_python_backend() {
        let backend = backend_for_path(Path::new("app.py"))
            .expect("python files should resolve to a backend");

        assert_eq!(backend.language(), Language::Python);
    }

    #[test]
    fn test_rust_backend() {
        let backend = backend_for_path(Path::new("src/main.rs"))
            .expect("rust files should resolve to a backend");

        assert_eq!(backend.language(), Language::Rust);
    }

    #[test]
    fn test_extensions() {
        assert_eq!(supported_extensions(), vec!["go", "py", "rs"]);
    }
}
