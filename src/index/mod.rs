use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::analysis::{DeclaredSymbol, Language, ParsedFile};
use crate::model::{IndexSummary, SymbolKind};

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
    pub fn package_for_file(
        &self,
        language: Language,
        file_path: &Path,
        package_name: &str,
    ) -> Option<&PackageIndex> {
        let key = PackageKey {
            language,
            package_name: package_name.to_string(),
            directory: package_directory(&self.root, file_path),
        };

        self.packages.get(&key)
    }

    pub fn resolve_import_path(&self, language: Language, import_path: &str) -> ImportResolution<'_> {
        let mut candidates = self
            .packages
            .values()
            .filter(|package| {
                package.language == language
                    && import_path_matches_directory(import_path, &package.directory)
            })
            .collect::<Vec<_>>();

        match candidates.len() {
            0 => ImportResolution::Unresolved,
            1 => ImportResolution::Resolved(candidates.remove(0)),
            _ => ImportResolution::Ambiguous(candidates),
        }
    }

    pub fn resolve_rust_module_import(
        &self,
        current_file_path: &Path,
        import_path: &str,
    ) -> ImportResolution<'_> {
        let Some((crate_root, current_module_segments)) =
            rust_module_context(&self.root, current_file_path)
        else {
            return ImportResolution::Unresolved;
        };
        let Some(target_segments) =
            normalize_rust_import_path(import_path, &current_module_segments)
        else {
            return ImportResolution::Unresolved;
        };
        let Some(module_name) = target_segments.last() else {
            return ImportResolution::Unresolved;
        };

        let file_module_directory = rust_file_module_directory(&crate_root, &target_segments);
        let mod_module_directory = rust_mod_module_directory(&crate_root, &target_segments);
        let mut candidates = self
            .packages
            .values()
            .filter(|package| {
                package.language == Language::Rust
                    && package.package_name == *module_name
                    && (package.directory == file_module_directory
                        || package.directory == mod_module_directory)
            })
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

    pub fn has_symbol(&self, name: &str) -> bool {
        self.symbols.iter().any(|symbol| symbol.name == name)
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
            language: file.language,
            package_name: package_name.clone(),
            directory: directory.clone(),
        };
        let package_entry = packages.entry(key).or_insert_with(|| PackageIndex {
            language: file.language,
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

fn rust_module_context(root: &Path, file_path: &Path) -> Option<(PathBuf, Vec<String>)> {
    let relative_path = file_path.strip_prefix(root).ok()?;
    let components = relative_path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let crate_root = components.first()?.as_str();

    if crate_root != "src" && crate_root != "tests" {
        return None;
    }

    let file_name = components.last()?.as_str();
    let directory_segments = if components.len() > 2 {
        components[1..components.len() - 1].to_vec()
    } else {
        Vec::new()
    };
    let mut module_segments = directory_segments;

    match file_name {
        "lib.rs" | "main.rs" | "mod.rs" => {}
        _ => {
            let stem = file_name.strip_suffix(".rs")?;
            if !stem.is_empty() {
                module_segments.push(stem.to_string());
            }
        }
    }

    Some((PathBuf::from(crate_root), module_segments))
}

fn normalize_rust_import_path(
    import_path: &str,
    current_module_segments: &[String],
) -> Option<Vec<String>> {
    let segments = import_path
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let head = segments.first()?.as_str();

    match head {
        "crate" => Some(segments.into_iter().skip(1).collect()),
        "self" => Some(
            current_module_segments
                .iter()
                .cloned()
                .chain(segments.into_iter().skip(1))
                .collect(),
        ),
        "super" => {
            let super_count = segments.iter().take_while(|segment| segment == &&"super".to_string()).count();
            if super_count > current_module_segments.len() {
                return None;
            }

            let mut resolved = current_module_segments[..current_module_segments.len() - super_count]
                .to_vec();
            resolved.extend(segments.into_iter().skip(super_count));
            Some(resolved)
        }
        _ => None,
    }
}

fn rust_file_module_directory(crate_root: &Path, target_segments: &[String]) -> PathBuf {
    if target_segments.len() <= 1 {
        return crate_root.to_path_buf();
    }

    let mut directory = crate_root.to_path_buf();
    for segment in &target_segments[..target_segments.len() - 1] {
        directory.push(segment);
    }
    directory
}

fn rust_mod_module_directory(crate_root: &Path, target_segments: &[String]) -> PathBuf {
    let mut directory = crate_root.to_path_buf();
    for segment in target_segments {
        directory.push(segment);
    }
    directory
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{ImportResolution, build_repository_index};
    use crate::analysis::{DeclaredSymbol, Language, ParsedFile, ParsedFunction};
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
                    is_test_function: false,
                    local_binding_names: Vec::new(),
                    doc_comment: None,
                    local_string_literals: Vec::new(),
                    test_summary: None,
                    safety_comment_lines: Vec::new(),
                    unsafe_lines: Vec::new(),
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
        let files = vec![sample_file(Language::Go, "/repo/utils/sample.go", "utils", &["Trim"])];

        let index = build_repository_index(Path::new("/repo"), &files);
        assert!(
            index
                .package_for_file(Language::Go, Path::new("/repo/utils/sample.go"), "utils")
                .is_some_and(|package| package.has_function("Trim"))
        );
    }

    #[test]
    fn keeps_same_package_names_separate_by_directory() {
        let files = vec![
            sample_file(Language::Go, "/repo/pkg/render/main.go", "render", &["Normalize"]),
            sample_file(Language::Go, "/repo/internal/render/main.go", "render", &["Sanitize"]),
        ];

        let index = build_repository_index(Path::new("/repo"), &files);

        assert!(
            index
                .package_for_file(Language::Go, Path::new("/repo/pkg/render/main.go"), "render")
                .is_some_and(|package| package.has_function("Normalize")
                    && !package.has_function("Sanitize"))
        );
        assert!(
            index
                .package_for_file(Language::Go, Path::new("/repo/internal/render/main.go"), "render")
                .is_some_and(|package| package.has_function("Sanitize")
                    && !package.has_function("Normalize"))
        );
    }

    #[test]
    fn resolves_imports_by_directory_suffix_not_package_name_only() {
        let files = vec![
            sample_file(Language::Go, "/repo/pkg/render/main.go", "render", &["Normalize"]),
            sample_file(Language::Go, "/repo/internal/render/main.go", "render", &["Sanitize"]),
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
    fn keeps_mixed_language_packages_separate_in_the_same_directory() {
        let files = vec![
            sample_file(Language::Go, "/repo/pkg/render/main.go", "render", &["Normalize"]),
            sample_file(Language::Rust, "/repo/pkg/render/lib.rs", "render", &["NormalizeRust"]),
        ];

        let index = build_repository_index(Path::new("/repo"), &files);

        assert!(index
            .package_for_file(Language::Go, Path::new("/repo/pkg/render/main.go"), "render")
            .is_some_and(|package| package.has_function("Normalize") && !package.has_function("NormalizeRust")));
        assert!(index
            .package_for_file(Language::Rust, Path::new("/repo/pkg/render/lib.rs"), "render")
            .is_some_and(|package| package.has_function("NormalizeRust") && !package.has_function("Normalize")));

        match index.resolve_import_path(Language::Go, "github.com/acme/project/pkg/render") {
            ImportResolution::Resolved(package) => {
                assert_eq!(package.language, Language::Go);
                assert!(package.has_function("Normalize"));
                assert!(!package.has_function("NormalizeRust"));
            }
            other => panic!("expected go package resolution, got {other:?}"),
        }
    }

    #[test]
    fn resolves_rust_module_imports_for_crate_self_and_super_paths() {
        let files = vec![
            sample_file(Language::Rust, "/repo/src/config/mod.rs", "config", &["shared"]),
            sample_file(Language::Rust, "/repo/src/config/render.rs", "render", &["normalize"]),
            sample_file(Language::Rust, "/repo/src/config/sub/helpers.rs", "helpers", &["load"]),
        ];

        let index = build_repository_index(Path::new("/repo"), &files);

        match index.resolve_rust_module_import(
            Path::new("/repo/src/lib.rs"),
            "crate::config::render",
        ) {
            ImportResolution::Resolved(package) => {
                assert_eq!(package.directory, PathBuf::from("src/config"));
                assert!(package.has_function("normalize"));
            }
            other => panic!("expected crate import to resolve, got {other:?}"),
        }

        match index.resolve_rust_module_import(
            Path::new("/repo/src/config/mod.rs"),
            "self::render",
        ) {
            ImportResolution::Resolved(package) => {
                assert_eq!(package.directory, PathBuf::from("src/config"));
                assert!(package.has_function("normalize"));
            }
            other => panic!("expected self import to resolve, got {other:?}"),
        }

        match index.resolve_rust_module_import(
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
