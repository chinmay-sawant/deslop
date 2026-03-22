use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::analysis::{DeclaredSymbol, ParsedFile};
use crate::model::{IndexSummary, SymbolKind};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PackageKey {
    package_name: String,
    directory: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct PackageIndex {
    pub package_name: String,
    pub directory: PathBuf,
    pub functions: BTreeSet<String>,
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
    pub fn package_for_file(&self, file_path: &Path, package_name: &str) -> Option<&PackageIndex> {
        let key = PackageKey {
            package_name: package_name.to_string(),
            directory: package_directory(&self.root, file_path),
        };

        self.packages.get(&key)
    }

    pub fn resolve_import_path(&self, import_path: &str) -> ImportResolution<'_> {
        let mut candidates = self
            .packages
            .values()
            .filter(|package| import_path_matches_directory(import_path, &package.directory))
            .collect::<Vec<_>>();

        match candidates.len() {
            0 => ImportResolution::Unresolved,
            1 => ImportResolution::Resolved(candidates.remove(0)),
            _ => ImportResolution::Ambiguous(candidates),
        }
    }

    pub fn summary(&self) -> IndexSummary {
        let package_count = self.packages.len();
        let symbol_count = self
            .packages
            .values()
            .map(|package| package.symbols.len())
            .sum();
        let import_count = self
            .packages
            .values()
            .map(|package| package.import_count)
            .sum();

        IndexSummary {
            package_count,
            symbol_count,
            import_count,
        }
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

    pub fn has_method(&self, receiver: &str, name: &str) -> bool {
        self.methods_by_receiver
            .get(receiver)
            .is_some_and(|methods| methods.contains(name))
    }
}

pub(crate) fn build_repository_index(root: &Path, files: &[ParsedFile]) -> RepositoryIndex {
    let mut packages = BTreeMap::new();

    for file in files {
        let package_name = file
            .package_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let directory = package_directory(root, &file.path);
        let key = PackageKey {
            package_name: package_name.clone(),
            directory: directory.clone(),
        };
        let package_entry = packages.entry(key).or_insert_with(|| PackageIndex {
            package_name,
            directory,
            functions: BTreeSet::new(),
            methods_by_receiver: BTreeMap::new(),
            symbols: Vec::new(),
            import_count: 0,
        });

        package_entry.import_count += file.imports.len();

        for symbol in &file.symbols {
            package_entry.symbols.push(symbol.clone());
            match symbol.kind {
                SymbolKind::Function => {
                    package_entry.functions.insert(symbol.name.clone());
                }
                SymbolKind::Method => {
                    if let Some(receiver) = &symbol.receiver_type {
                        package_entry
                            .methods_by_receiver
                            .entry(receiver.clone())
                            .or_insert_with(BTreeSet::new)
                            .insert(symbol.name.clone());
                    }
                }
                _ => {}
            }
        }
    }

    RepositoryIndex {
        root: root.to_path_buf(),
        packages,
    }
}

fn package_directory(root: &Path, file_path: &Path) -> PathBuf {
    let Some(parent) = file_path.parent() else {
        return PathBuf::new();
    };

    if root.as_os_str().is_empty() {
        return parent.to_path_buf();
    }

    parent
        .strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| parent.to_path_buf())
}

fn import_path_matches_directory(import_path: &str, directory: &Path) -> bool {
    let directory_segments = directory
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    if directory_segments.is_empty() {
        return false;
    }

    let import_segments = import_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if directory_segments.len() > import_segments.len() {
        return false;
    }

    import_segments[import_segments.len() - directory_segments.len()..]
        .iter()
        .zip(directory_segments.iter())
        .all(|(left, right)| *left == right)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{ImportResolution, build_repository_index};
    use crate::analysis::{DeclaredSymbol, Language, ParsedFile, ParsedFunction};
    use crate::model::{FunctionFingerprint, SymbolKind};

    fn sample_file(path: &str, package_name: &str, function_names: &[&str]) -> ParsedFile {
        ParsedFile {
            language: Language::Go,
            path: PathBuf::from(path),
            package_name: Some(package_name.to_string()),
            is_test_file: false,
            syntax_error: false,
            byte_size: 10,
            package_string_literals: Vec::new(),
            struct_tags: Vec::new(),
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
                    calls: Vec::new(),
                    has_context_parameter: false,
                    doc_comment: None,
                    local_string_literals: Vec::new(),
                    test_summary: None,
                    dropped_error_lines: Vec::new(),
                    panic_on_error_lines: Vec::new(),
                    errorf_calls: Vec::new(),
                    context_factory_calls: Vec::new(),
                    goroutine_launch_lines: Vec::new(),
                    goroutine_in_loop_lines: Vec::new(),
                    goroutine_without_shutdown_lines: Vec::new(),
                    sleep_in_loop_lines: Vec::new(),
                    busy_wait_lines: Vec::new(),
                    mutex_lock_in_loop_lines: Vec::new(),
                    allocation_in_loop_lines: Vec::new(),
                    fmt_in_loop_lines: Vec::new(),
                    reflection_in_loop_lines: Vec::new(),
                    string_concat_in_loop_lines: Vec::new(),
                    json_marshal_in_loop_lines: Vec::new(),
                    db_query_calls: Vec::new(),
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
        }
    }

    #[test]
    fn builds_package_lookup() {
        let files = vec![sample_file("/repo/utils/sample.go", "utils", &["Trim"])];

        let index = build_repository_index(Path::new("/repo"), &files);
        assert!(
            index
                .package_for_file(Path::new("/repo/utils/sample.go"), "utils")
                .is_some_and(|package| package.has_function("Trim"))
        );
    }

    #[test]
    fn keeps_same_package_names_separate_by_directory() {
        let files = vec![
            sample_file("/repo/pkg/render/main.go", "render", &["Normalize"]),
            sample_file("/repo/internal/render/main.go", "render", &["Sanitize"]),
        ];

        let index = build_repository_index(Path::new("/repo"), &files);

        assert!(
            index
                .package_for_file(Path::new("/repo/pkg/render/main.go"), "render")
                .is_some_and(|package| package.has_function("Normalize")
                    && !package.has_function("Sanitize"))
        );
        assert!(
            index
                .package_for_file(Path::new("/repo/internal/render/main.go"), "render")
                .is_some_and(|package| package.has_function("Sanitize")
                    && !package.has_function("Normalize"))
        );
    }

    #[test]
    fn resolves_imports_by_directory_suffix_not_package_name_only() {
        let files = vec![
            sample_file("/repo/pkg/render/main.go", "render", &["Normalize"]),
            sample_file("/repo/internal/render/main.go", "render", &["Sanitize"]),
        ];

        let index = build_repository_index(Path::new("/repo"), &files);

        match index.resolve_import_path("github.com/acme/project/pkg/render") {
            ImportResolution::Resolved(package) => {
                assert_eq!(package.directory, PathBuf::from("pkg/render"));
                assert!(package.has_function("Normalize"));
                assert!(!package.has_function("Sanitize"));
            }
            other => panic!("expected resolved import, got {other:?}"),
        }
    }
}
