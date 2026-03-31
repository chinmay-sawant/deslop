mod backend;
mod config;
mod error;
mod go;
mod python;
mod rust;
mod types;

pub(crate) use backend::{
    Language, LanguageBackend, backend_for_language, backend_for_path, registered_backends,
    supported_extensions,
};
pub(crate) use config::AnalysisConfig;
pub use error::Error;
pub(crate) use error::Result as AnalysisResult;

pub(crate) use types::{
    BlockFingerprint, CallSite, ClassSummary, CommentSummary, ContextFactoryCall, DbQueryCall,
    DeclaredSymbol, ExceptionHandler, FieldSummary, FormattedErrorCall, GinCallSummary,
    GoFieldSummary, GoStructSummary, GormChainStep, GormQueryChain, ImportSpec, InterfaceSummary,
    MacroCall, NamedLiteral, PackageVarSummary, ParseInputCall, ParsedFile, ParsedFunction,
    PythonFieldSummary, PythonModelSummary, RuntimeCall, RustEnumSummary, RustStaticSummary,
    StructSummary, StructTag, TestFunctionSummary, TopLevelBindingSummary, TopLevelCallSummary,
    UnsafePattern, UnsafePatternKind,
};

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
