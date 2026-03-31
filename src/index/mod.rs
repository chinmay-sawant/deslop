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

    pub fn resolve_import_path(
        &self,
        language: Language,
        import_path: &str,
    ) -> ImportResolution<'_> {
        let mut candidates = self
            .packages
            .values()
            .filter(|package| match language {
                Language::Python => {
                    package.language == language
                        && python_import_matches_module(import_path, package)
                }
                _ => {
                    package.language == language
                        && import_matches_dir(import_path, &package.directory)
                }
            })
            .collect::<Vec<_>>();

        match candidates.len() {
            0 => ImportResolution::Unresolved,
            1 => ImportResolution::Resolved(candidates.remove(0)),
            _ => ImportResolution::Ambiguous(candidates),
        }
    }

    pub fn resolve_rust_import(
        &self,
        current_file_path: &Path,
        import_path: &str,
    ) -> ImportResolution<'_> {
        let Some((crate_root, current_module_segments)) =
            rust_module_context(&self.root, current_file_path)
        else {
            return ImportResolution::Unresolved;
        };
        let Some(target_segments) = normalize_rust_path(import_path, &current_module_segments)
        else {
            return ImportResolution::Unresolved;
        };
        if target_segments.is_empty() {
            let candidates = self
                .packages
                .values()
                .filter(|package| {
                    package.language == Language::Rust && package.directory == crate_root
                })
                .collect::<Vec<_>>();

            return match candidates.len() {
                0 => ImportResolution::Unresolved,
                1 => ImportResolution::Resolved(candidates[0]),
                _ => ImportResolution::Ambiguous(candidates),
            };
        }

        let Some(module_name) = target_segments.last() else {
            return ImportResolution::Unresolved;
        };
        let file_module_directory = rust_file_mod_dir(&crate_root, &target_segments);
        let mod_module_directory = rust_mod_mod_dir(&crate_root, &target_segments);
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

fn python_import_matches_module(import_path: &str, package: &PackageIndex) -> bool {
    let import_segments = import_path
        .split('.')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if import_segments.is_empty() {
        return false;
    }

    let directory_segments = package
        .directory
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let full_import_path = import_segments
        .iter()
        .map(|segment| (*segment).to_string())
        .collect::<Vec<_>>();

    for candidate_index in [
        import_segments.len().saturating_sub(1),
        import_segments.len().saturating_sub(2),
    ] {
        let Some(candidate_name) = import_segments.get(candidate_index).copied() else {
            continue;
        };
        if candidate_name == "*" || candidate_name != package.package_name {
            continue;
        }

        let prefix_without_module = import_segments
            .get(..candidate_index)
            .into_iter()
            .flatten()
            .map(|segment| (*segment).to_string())
            .collect::<Vec<_>>();
        if directory_segments.ends_with(&prefix_without_module)
            || directory_segments.ends_with(&full_import_path)
        {
            return true;
        }
    }

    false
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

pub(crate) fn build_repository_index(root: &Path, files: &[ParsedFile]) -> RepositoryIndex {
    let mut packages = BTreeMap::new();

    for file in files {
        let ParsedFile {
            language,
            path,
            package_name,
            imports,
            symbols,
            ..
        } = file;
        let language = *language;
        let package_name = package_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let directory = package_directory(root, path);
        let import_count = imports.len();
        let key = PackageKey {
            language,
            package_name: package_name.clone(),
            directory: directory.clone(),
        };
        let package_entry = packages.entry(key).or_insert_with(|| PackageIndex {
            language,
            package_name,
            directory,
            functions: BTreeSet::new(),
            contextless_wrapper_functions: BTreeSet::new(),
            methods_by_receiver: BTreeMap::new(),
            symbols: Vec::new(),
            import_count: 0,
        });

        package_entry.import_count += import_count;

        for function in &file.functions {
            if function.fingerprint.receiver_type.is_none()
                && is_contextless_wrapper_candidate(file, function)
            {
                package_entry
                    .contextless_wrapper_functions
                    .insert(function.fingerprint.name.clone());
            }
        }

        for symbol in symbols {
            insert_symbol(package_entry, symbol);
        }
    }

    RepositoryIndex {
        root: root.to_path_buf(),
        packages,
    }
}

fn insert_symbol(package_entry: &mut PackageIndex, symbol: &DeclaredSymbol) {
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
                    .or_default()
                    .insert(symbol.name.clone());
            }
        }
        _ => {}
    }
}

fn is_contextless_wrapper_candidate(
    file: &ParsedFile,
    function: &crate::analysis::ParsedFunction,
) -> bool {
    if function.has_context_parameter {
        return false;
    }

    let import_aliases = file
        .imports
        .iter()
        .map(|import| (import.alias.as_str(), import.path.as_str()))
        .collect::<BTreeMap<_, _>>();

    function.calls.iter().any(|call| {
        let Some(receiver) = call.receiver.as_deref() else {
            return false;
        };
        let Some(import_path) = import_aliases.get(receiver) else {
            return false;
        };

        matches!(*import_path, "net/http")
            && matches!(
                call.name.as_str(),
                "Get" | "Head" | "Post" | "PostForm" | "NewRequest"
            )
            || matches!(*import_path, "os/exec") && call.name == "Command"
            || matches!(*import_path, "net") && matches!(call.name.as_str(), "Dial" | "DialTimeout")
    }) || function.db_query_calls.iter().any(|query_call| {
        matches!(
            query_call.method_name.as_str(),
            "Query" | "QueryRow" | "Exec" | "Get" | "Select"
        )
    })
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

fn import_matches_dir(import_path: &str, directory: &Path) -> bool {
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

    import_segments
        .get(import_segments.len() - directory_segments.len()..)
        .into_iter()
        .flatten()
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
        components
            .get(1..components.len() - 1)
            .map(|segments| segments.to_vec())
            .unwrap_or_default()
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

fn normalize_rust_path(
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
            let super_count = segments
                .iter()
                .take_while(|segment| segment == &&"super".to_string())
                .count();
            if super_count > current_module_segments.len() {
                return None;
            }

            let mut resolved = current_module_segments
                .get(..current_module_segments.len() - super_count)
                .map(|segments| segments.to_vec())
                .unwrap_or_default();
            resolved.extend(segments.into_iter().skip(super_count));
            Some(resolved)
        }
        _ => None,
    }
}

fn rust_file_mod_dir(crate_root: &Path, target_segments: &[String]) -> PathBuf {
    if target_segments.len() <= 1 {
        return crate_root.to_path_buf();
    }

    let mut directory = crate_root.to_path_buf();
    for segment in target_segments.iter().take(target_segments.len() - 1) {
        directory.push(segment);
    }
    directory
}

fn rust_mod_mod_dir(crate_root: &Path, target_segments: &[String]) -> PathBuf {
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
            line_count: 1,
            byte_size: 10,
            pkg_strings: Vec::new(),
            comments: Vec::new(),
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
                    signature_text: String::new(),
                    body_start_line: 1,
                    calls: Vec::new(),
                    exception_handlers: Vec::new(),
                    has_context_parameter: false,
                    is_test_function: false,
                    local_binding_names: Vec::new(),
                    doc_comment: None,
                    body_text: String::new(),
                    local_strings: Vec::new(),
                    normalized_body: String::new(),
                    validation_signature: None,
                    exception_block_signatures: Vec::new(),
                    test_summary: None,
                    safety_comment_lines: Vec::new(),
                    unsafe_lines: Vec::new(),
                    dropped_errors: Vec::new(),
                    panic_errors: Vec::new(),
                    errorf_calls: Vec::new(),
                    context_factory_calls: Vec::new(),
                    goroutines: Vec::new(),
                    loop_goroutines: Vec::new(),
                    unmanaged_goroutines: Vec::new(),
                    sleep_loops: Vec::new(),
                    busy_wait_lines: Vec::new(),
                    mutex_loops: Vec::new(),
                    alloc_loops: Vec::new(),
                    fmt_loops: Vec::new(),
                    reflect_loops: Vec::new(),
                    concat_loops: Vec::new(),
                    json_loops: Vec::new(),
                    db_query_calls: Vec::new(),
                    gorm_query_chains: Vec::new(),
                    parse_input_calls: Vec::new(),
                    gin_calls: Vec::new(),
                    none_comparison_lines: Vec::new(),
                    side_effect_comprehension_lines: Vec::new(),
                    redundant_return_none_lines: Vec::new(),
                    list_materialization_lines: Vec::new(),
                    deque_operation_lines: Vec::new(),
                    temp_collection_lines: Vec::new(),
                    recursive_call_lines: Vec::new(),
                    list_membership_loop_lines: Vec::new(),
                    repeated_len_loop_lines: Vec::new(),
                    builtin_candidate_lines: Vec::new(),
                    missing_context_manager_lines: Vec::new(),
                    has_complete_type_hints: false,
                    has_varargs: false,
                    has_kwargs: false,
                    is_async: false,
                    await_points: Vec::new(),
                    macro_calls: Vec::new(),
                    spawn_calls: Vec::new(),
                    lock_calls: Vec::new(),
                    permit_acquires: Vec::new(),
                    futures_created: Vec::new(),
                    blocking_calls: Vec::new(),
                    select_macro_lines: Vec::new(),
                    drop_impl: false,
                    write_loops: Vec::new(),
                    line_iteration_loops: Vec::new(),
                    default_hasher_lines: Vec::new(),
                    boxed_container_lines: Vec::new(),
                    unsafe_soundness: Vec::new(),
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
            class_summaries: Vec::new(),
            package_vars: Vec::new(),
            interfaces: Vec::new(),
            go_structs: Vec::new(),
            module_scope_calls: Vec::new(),
            top_level_bindings: Vec::new(),
            python_models: Vec::new(),
            rust_statics: Vec::new(),
            rust_enums: Vec::new(),
            structs: Vec::new(),
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
