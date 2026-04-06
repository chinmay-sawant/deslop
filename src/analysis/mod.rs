mod backend;
mod config;
mod error;
mod go;
mod python;
pub(crate) mod rust;
mod types;

use std::path::Path;

pub(crate) use backend::{
    Language, LanguageBackend, backend_for_language, backend_for_path, registered_backends,
    supported_extensions,
};
pub(crate) use config::AnalysisConfig;
pub use error::Error;
pub(crate) use error::Result as AnalysisResult;

// Shared / cross-language types
pub(crate) use types::{
    BlockFingerprint, CallSite, CommentSummary, DeclaredSymbol, FormattedErrorCall, ImportSpec,
    LanguageFileData, NamedLiteral, ParsedFile, ParsedFunction, TestFunctionSummary,
    TopLevelBindingSummary, TopLevelCallSummary,
};
// Go-specific evidence types
pub(crate) use types::{
    ContextFactoryCall, DbQueryCall, GinCallSummary, GoFieldSummary, GoFileData,
    GoFunctionEvidence, GoStructSummary, GormChainStep, GormQueryChain, InterfaceSummary,
    PackageVarSummary, ParseInputCall, StructTag,
};
// Python-specific evidence types
pub(crate) use types::{
    ClassSummary, ExceptionHandler, PythonFieldSummary, PythonFileData, PythonFunctionEvidence,
    PythonModelSummary,
};
// Rust-specific evidence types
pub(crate) use types::{
    FieldSummary, MacroCall, RuntimeCall, RustAttributeSummary, RustEnumSummary, RustFileData,
    RustFunctionEvidence, RustModuleDeclaration, RustStaticSummary, StructSummary, UnsafePattern,
    UnsafePatternKind,
};

pub(crate) fn parse_source_file(path: &Path, source: &str) -> crate::Result<ParsedFile> {
    let backend =
        backend_for_path(path).ok_or_else(|| crate::Error::unsupported_parser_path(path))?;
    backend.parse_file(path, source)
}

pub fn validate_source(path: &Path, source: &str) -> crate::Result<()> {
    let _ = parse_source_file(path, source)?;
    Ok(())
}

pub fn syntax_error_for_source(path: &Path, source: &str) -> crate::Result<bool> {
    let parsed = parse_source_file(path, source)?;
    Ok(parsed.syntax_error)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        Language, backend_for_path, parse_source_file, registered_backends, supported_extensions,
    };

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

    #[test]
    fn test_parse_source_rejects_unknown_extensions() {
        let error = parse_source_file(Path::new("input.txt"), "package main")
            .expect_err("unknown extensions should fail");

        assert!(matches!(error, crate::Error::UnsupportedParserPath { .. }));
    }

    #[test]
    fn every_backend_has_at_least_one_supported_extension() {
        for &backend in registered_backends() {
            assert!(
                !backend.supported_extensions().is_empty(),
                "backend for {:?} should declare at least one file extension",
                backend.language()
            );
        }
    }
}
