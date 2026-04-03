mod build;
mod resolve;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::analysis::{DeclaredSymbol, Language};
use crate::model::IndexSummary;

pub(crate) use build::build_repository_index;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PackageKey {
    language: Language,
    package_name: String,
    directory: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct PackageIndex {
    pub language: Language,
    pub package_name: String,
    pub directory: PathBuf,
    pub functions: BTreeSet<String>,
    pub contextless_wrapper_functions: BTreeSet<String>,
    pub methods_by_receiver: BTreeMap<String, BTreeSet<String>>,
    pub symbols: Vec<DeclaredSymbol>,
    pub import_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) enum ImportResolution<'a> {
    Resolved(&'a PackageIndex),
    Ambiguous(Vec<&'a PackageIndex>),
    Unresolved,
}

#[derive(Debug, Clone)]
pub(crate) struct RepositoryIndex {
    root: PathBuf,
    packages: BTreeMap<PackageKey, PackageIndex>,
}

impl RepositoryIndex {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn package_for_file(
        &self,
        language: Language,
        file_path: &Path,
        package_name: &str,
    ) -> Option<&PackageIndex> {
        resolve::package_for_file(self, language, file_path, package_name)
    }

    pub fn resolve_import_path(
        &self,
        language: Language,
        import_path: &str,
    ) -> ImportResolution<'_> {
        resolve::resolve_import_path(self, language, import_path)
    }

    pub fn resolve_rust_import(
        &self,
        current_file_path: &Path,
        import_path: &str,
    ) -> ImportResolution<'_> {
        resolve::resolve_rust_import(self, current_file_path, import_path)
    }

    pub fn summary(&self) -> IndexSummary {
        resolve::summary(self)
    }
}

impl PackageIndex {
    pub fn directory_display(&self) -> String {
        if self.directory.as_os_str().is_empty() {
            ".".to_string()
        } else {
            self.directory.display().to_string()
        }
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains(name)
    }

    pub fn has_contextless_wrapper_function(&self, name: &str) -> bool {
        self.contextless_wrapper_functions.contains(name)
    }

    pub fn has_method(&self, receiver: &str, name: &str) -> bool {
        self.methods_by_receiver
            .get(receiver)
            .is_some_and(|methods| methods.contains(name))
    }

    pub fn has_symbol(&self, name: &str) -> bool {
        self.symbols.iter().any(|symbol| symbol.name == name)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{ImportResolution, build_repository_index};
    use crate::analysis::{
        DeclaredSymbol, GoFileData, Language, LanguageFileData, ParsedFile, ParsedFunction,
        PythonFileData, RustFileData,
    };
    use crate::model::{FunctionFingerprint, SymbolKind};

    fn sample_file(
        language: Language,
        path: &str,
        package_name: &str,
        function_names: &[&str],
    ) -> ParsedFile {
        ParsedFile {
            language,
            path: PathBuf::from(path),
            package_name: Some(package_name.to_string()),
            is_test_file: false,
            syntax_error: false,
            line_count: 1,
            byte_size: 10,
            pkg_strings: Vec::new(),
            comments: Vec::new(),
            functions: function_names
                .iter()
                .map(|name| ParsedFunction {
                    fingerprint: FunctionFingerprint {
                        name: (*name).to_string(),
                        kind: "function".to_string(),
                        receiver_type: None,
                        start_line: 1,
                        end_line: 1,
                        line_count: 1,
                        comment_lines: 0,
                        code_lines: 1,
                        comment_to_code_ratio: 0.0,
                        complexity_score: 1,
                        symmetry_score: 0.0,
                        boilerplate_err_guards: 0,
                        contains_any_type: false,
                        contains_empty_interface: false,
                        type_assertion_count: 0,
                        call_count: 0,
                    },
                    signature_text: String::new(),
                    body_start_line: 1,
                    calls: Vec::new(),
                    is_test_function: false,
                    local_binding_names: Vec::new(),
                    doc_comment: None,
                    body_text: String::new(),
                    local_strings: Vec::new(),
                    test_summary: None,
                    go: None,
                    python: None,
                    rust: None,
                })
                .collect(),
            imports: Vec::new(),
            symbols: function_names
                .iter()
                .map(|name| DeclaredSymbol {
                    name: (*name).to_string(),
                    kind: SymbolKind::Function,
                    receiver_type: None,
                    receiver_is_pointer: None,
                    line: 1,
                })
                .collect(),
            top_level_bindings: Vec::new(),
            module_scope_calls: Vec::new(),
            lang: match language {
                Language::Go => LanguageFileData::Go(GoFileData::default()),
                Language::Python => LanguageFileData::Python(PythonFileData::default()),
                Language::Rust => LanguageFileData::Rust(RustFileData::default()),
            },
        }
    }

    #[test]
    fn builds_package_lookup() {
        let files = vec![sample_file(
            Language::Go,
            "/repo/utils/sample.go",
            "utils",
            &["Trim"],
        )];

        let index = build_repository_index(Path::new("/repo"), &files);
        assert!(
            index
                .package_for_file(Language::Go, Path::new("/repo/utils/sample.go"), "utils")
                .is_some_and(|package| package.has_function("Trim"))
        );
    }

    #[test]
    fn test_pkg_separation() {
        let files = vec![
            sample_file(
                Language::Go,
                "/repo/pkg/render/main.go",
                "render",
                &["Normalize"],
            ),
            sample_file(
                Language::Go,
                "/repo/internal/render/main.go",
                "render",
                &["Sanitize"],
            ),
        ];

        let index = build_repository_index(Path::new("/repo"), &files);

        assert!(
            index
                .package_for_file(
                    Language::Go,
                    Path::new("/repo/pkg/render/main.go"),
                    "render"
                )
                .is_some_and(|package| package.has_function("Normalize")
                    && !package.has_function("Sanitize"))
        );
        assert!(
            index
                .package_for_file(
                    Language::Go,
                    Path::new("/repo/internal/render/main.go"),
                    "render"
                )
                .is_some_and(|package| package.has_function("Sanitize")
                    && !package.has_function("Normalize"))
        );
    }

    #[test]
    fn test_import_suffix() {
        let files = vec![
            sample_file(
                Language::Go,
                "/repo/pkg/render/main.go",
                "render",
                &["Normalize"],
            ),
            sample_file(
                Language::Go,
                "/repo/internal/render/main.go",
                "render",
                &["Sanitize"],
            ),
        ];

        let index = build_repository_index(Path::new("/repo"), &files);

        match index.resolve_import_path(Language::Go, "github.com/acme/project/pkg/render") {
            ImportResolution::Resolved(package) => {
                assert_eq!(package.directory, PathBuf::from("pkg/render"));
                assert!(package.has_function("Normalize"));
                assert!(!package.has_function("Sanitize"));
            }
            other => panic!("expected resolved import, got {other:?}"),
        }
    }

    #[test]
    fn test_mixed_lang() {
        let files = vec![
            sample_file(
                Language::Go,
                "/repo/pkg/render/main.go",
                "render",
                &["Normalize"],
            ),
            sample_file(
                Language::Rust,
                "/repo/pkg/render/lib.rs",
                "render",
                &["NormalizeRust"],
            ),
            sample_file(
                Language::Python,
                "/repo/pkg/render/__init__.py",
                "render",
                &["normalize_python"],
            ),
        ];

        let index = build_repository_index(Path::new("/repo"), &files);

        assert!(
            index
                .package_for_file(
                    Language::Go,
                    Path::new("/repo/pkg/render/main.go"),
                    "render"
                )
                .is_some_and(|package| package.has_function("Normalize")
                    && !package.has_function("NormalizeRust"))
        );
        assert!(
            index
                .package_for_file(
                    Language::Rust,
                    Path::new("/repo/pkg/render/lib.rs"),
                    "render"
                )
                .is_some_and(|package| package.has_function("NormalizeRust")
                    && !package.has_function("Normalize"))
        );
        assert!(
            index
                .package_for_file(
                    Language::Python,
                    Path::new("/repo/pkg/render/__init__.py"),
                    "render"
                )
                .is_some_and(|package| package.has_function("normalize_python")
                    && !package.has_function("Normalize")
                    && !package.has_function("NormalizeRust"))
        );

        match index.resolve_import_path(Language::Go, "github.com/acme/project/pkg/render") {
            ImportResolution::Resolved(package) => {
                assert_eq!(package.language, Language::Go);
                assert!(package.has_function("Normalize"));
                assert!(!package.has_function("NormalizeRust"));
                assert!(!package.has_function("normalize_python"));
            }
            other => panic!("expected go package resolution, got {other:?}"),
        }
    }

    #[test]
    fn test_rust_imports() {
        let files = vec![
            sample_file(
                Language::Rust,
                "/repo/src/config/mod.rs",
                "config",
                &["shared"],
            ),
            sample_file(
                Language::Rust,
                "/repo/src/config/render.rs",
                "render",
                &["normalize"],
            ),
            sample_file(
                Language::Rust,
                "/repo/src/config/sub/helpers.rs",
                "helpers",
                &["load"],
            ),
        ];

        let index = build_repository_index(Path::new("/repo"), &files);

        match index.resolve_rust_import(Path::new("/repo/src/lib.rs"), "crate::config::render") {
            ImportResolution::Resolved(package) => {
                assert_eq!(package.directory, PathBuf::from("src/config"));
                assert!(package.has_function("normalize"));
            }
            other => panic!("expected crate import to resolve, got {other:?}"),
        }

        match index.resolve_rust_import(Path::new("/repo/src/config/mod.rs"), "self::render") {
            ImportResolution::Resolved(package) => {
                assert_eq!(package.directory, PathBuf::from("src/config"));
                assert!(package.has_function("normalize"));
            }
            other => panic!("expected self import to resolve, got {other:?}"),
        }

        match index.resolve_rust_import(
            Path::new("/repo/src/config/sub/helpers.rs"),
            "super::super::render",
        ) {
            ImportResolution::Resolved(package) => {
                assert_eq!(package.directory, PathBuf::from("src/config"));
                assert!(package.has_function("normalize"));
            }
            other => panic!("expected super import to resolve, got {other:?}"),
        }
    }
}
