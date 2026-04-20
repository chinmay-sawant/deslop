use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::analysis::{
    DeclaredSymbol, Language, ParsedFile, ParsedFunction, RustModuleDeclaration,
};
use crate::model::SymbolKind;

use super::{PackageIndex, PackageKey, RepositoryIndex};

type RustChildModules = BTreeMap<PathBuf, BTreeMap<String, Vec<PathBuf>>>;
type RustParentModules = BTreeMap<PathBuf, Vec<PathBuf>>;
type RustCrateRoots = BTreeMap<PathBuf, Vec<PathBuf>>;
type RustIncludeNeighbors = BTreeMap<PathBuf, Vec<PathBuf>>;

pub(crate) fn build_repository_index(root: &Path, files: &[ParsedFile]) -> RepositoryIndex {
    let mut packages = BTreeMap::new();
    let mut rust_package_names_by_file = BTreeMap::new();
    let mut rust_imports_by_file = BTreeMap::new();

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
        if language == Language::Rust {
            rust_package_names_by_file.insert(path.clone(), package_name.clone());
            rust_imports_by_file.insert(path.clone(), imports.clone());
        }
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

    let rust_include_neighbors = build_rust_include_graph(files);
    let (rust_child_modules, rust_parent_modules, rust_crate_roots) =
        build_rust_module_graph(files);

    RepositoryIndex {
        root: root.to_path_buf(),
        packages,
        rust_package_names_by_file,
        rust_imports_by_file,
        rust_include_neighbors,
        rust_child_modules,
        rust_parent_modules,
        rust_crate_roots,
    }
}

pub(super) fn package_directory(root: &Path, file_path: &Path) -> PathBuf {
    let Some(parent) = file_path.parent() else {
        return PathBuf::new();
    };

    if root.as_os_str().is_empty() {
        return parent.to_path_buf();
    }

    parent
        .strip_prefix(root)
        .map_or_else(|_| parent.to_path_buf(), Path::to_path_buf)
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

fn build_rust_module_graph(
    files: &[ParsedFile],
) -> (RustChildModules, RustParentModules, RustCrateRoots) {
    let rust_files = files
        .iter()
        .filter(|file| file.language == Language::Rust)
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let mut child_modules = RustChildModules::new();
    let mut parent_modules = RustParentModules::new();

    for file in files.iter().filter(|file| file.language == Language::Rust) {
        for declaration in file.rust_module_declarations() {
            let resolved_paths = resolve_module_declaration_paths(&file.path, declaration);
            for resolved_path in resolved_paths {
                if !rust_files.contains(&resolved_path) {
                    continue;
                }
                child_modules
                    .entry(file.path.clone())
                    .or_default()
                    .entry(declaration.name.clone())
                    .or_default()
                    .push(resolved_path.clone());
                parent_modules
                    .entry(resolved_path)
                    .or_default()
                    .push(file.path.clone());
            }
        }
    }

    for children in child_modules.values_mut() {
        for paths in children.values_mut() {
            paths.sort();
            paths.dedup();
        }
    }
    for parents in parent_modules.values_mut() {
        parents.sort();
        parents.dedup();
    }

    let child_files = parent_modules.keys().cloned().collect::<BTreeSet<_>>();
    let roots = rust_files
        .iter()
        .filter(|path| !child_files.contains(*path))
        .cloned()
        .collect::<Vec<_>>();

    let mut crate_roots = RustCrateRoots::new();
    for root in &roots {
        let mut visited = BTreeSet::new();
        assign_crate_root(root, root, &child_modules, &mut crate_roots, &mut visited);
    }
    for roots in crate_roots.values_mut() {
        roots.sort();
        roots.dedup();
    }

    (child_modules, parent_modules, crate_roots)
}

fn build_rust_include_graph(files: &[ParsedFile]) -> RustIncludeNeighbors {
    let rust_files = files
        .iter()
        .filter(|file| file.language == Language::Rust)
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    let mut neighbors = RustIncludeNeighbors::new();

    for file in files.iter().filter(|file| file.language == Language::Rust) {
        for declaration in file.rust_include_declarations() {
            let resolved_path = resolve_include_declaration_path(&file.path, &declaration.path);
            if !rust_files.contains(&resolved_path) {
                continue;
            }

            neighbors
                .entry(file.path.clone())
                .or_default()
                .push(resolved_path.clone());
            neighbors
                .entry(resolved_path)
                .or_default()
                .push(file.path.clone());
        }
    }

    for linked_files in neighbors.values_mut() {
        linked_files.sort();
        linked_files.dedup();
    }

    neighbors
}

fn assign_crate_root(
    root: &Path,
    current: &Path,
    child_modules: &BTreeMap<PathBuf, BTreeMap<String, Vec<PathBuf>>>,
    crate_roots: &mut BTreeMap<PathBuf, Vec<PathBuf>>,
    visited: &mut BTreeSet<PathBuf>,
) {
    if !visited.insert(current.to_path_buf()) {
        return;
    }

    crate_roots
        .entry(current.to_path_buf())
        .or_default()
        .push(root.to_path_buf());

    if let Some(children) = child_modules.get(current) {
        for paths in children.values() {
            for path in paths {
                assign_crate_root(root, path, child_modules, crate_roots, visited);
            }
        }
    }
}

fn resolve_module_declaration_paths(
    parent_file_path: &Path,
    declaration: &RustModuleDeclaration,
) -> Vec<PathBuf> {
    let Some(parent_dir) = parent_file_path.parent() else {
        return Vec::new();
    };

    if let Some(path_override) = &declaration.path_override {
        return vec![normalize_path(&parent_dir.join(path_override))];
    }

    let mut candidates = Vec::new();
    if is_directory_root_module(parent_file_path) {
        candidates.push(normalize_path(
            &parent_dir.join(format!("{}.rs", declaration.name)),
        ));
        candidates.push(normalize_path(
            &parent_dir.join(&declaration.name).join("mod.rs"),
        ));
        return candidates;
    }

    let Some(stem) = parent_file_path.file_stem().and_then(|stem| stem.to_str()) else {
        return Vec::new();
    };
    let module_dir = parent_dir.join(stem);
    candidates.push(normalize_path(
        &module_dir.join(format!("{}.rs", declaration.name)),
    ));
    candidates.push(normalize_path(
        &module_dir.join(&declaration.name).join("mod.rs"),
    ));
    candidates
}

fn resolve_include_declaration_path(parent_file_path: &Path, include_path: &str) -> PathBuf {
    let Some(parent_dir) = parent_file_path.parent() else {
        return PathBuf::from(include_path);
    };

    normalize_path(&parent_dir.join(include_path))
}

fn is_directory_root_module(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("mod.rs" | "lib.rs" | "main.rs")
    )
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }

    normalized
}

fn is_contextless_wrapper_candidate(file: &ParsedFile, function: &ParsedFunction) -> bool {
    if function.go_evidence().has_context_parameter {
        return false;
    }

    let go = function.go_evidence();
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
    }) || go.db_query_calls.iter().any(|query_call| {
        matches!(
            query_call.method_name.as_str(),
            "Query" | "QueryRow" | "Exec" | "Get" | "Select"
        )
    })
}
