use std::path::{Path, PathBuf};

use crate::analysis::Language;
use crate::model::IndexSummary;

use super::build::package_directory;
use super::{ImportResolution, PackageIndex, PackageKey, RepositoryIndex};

#[derive(Debug, Clone)]
pub(crate) enum RustModuleFileResolution {
    Resolved(PathBuf),
    Ambiguous(Vec<PathBuf>),
    Unresolved,
}

pub(crate) fn package_for_file<'a>(
    index: &'a RepositoryIndex,
    language: Language,
    file_path: &Path,
    package_name: &str,
) -> Option<&'a PackageIndex> {
    let key = PackageKey {
        language,
        package_name: package_name.to_string(),
        directory: package_directory(&index.root, file_path),
    };

    index.packages.get(&key)
}

pub(crate) fn package_for_rust_file<'a>(
    index: &'a RepositoryIndex,
    file_path: &Path,
) -> Option<&'a PackageIndex> {
    let package_name = index.rust_package_names_by_file.get(file_path)?;
    package_for_file(index, Language::Rust, file_path, package_name)
}

pub(crate) fn resolve_import_path<'a>(
    index: &'a RepositoryIndex,
    language: Language,
    import_path: &str,
) -> ImportResolution<'a> {
    let mut candidates = index
        .packages
        .values()
        .filter(|package| match language {
            Language::Python => {
                package.language == language && python_import_matches_module(import_path, package)
            }
            _ => {
                package.language == language && import_matches_dir(import_path, &package.directory)
            }
        })
        .collect::<Vec<_>>();

    match candidates.len() {
        0 => ImportResolution::Unresolved,
        1 => ImportResolution::Resolved(candidates.remove(0)),
        _ => ImportResolution::Ambiguous(candidates),
    }
}

pub(crate) fn resolve_rust_import<'a>(
    index: &'a RepositoryIndex,
    current_file_path: &Path,
    import_path: &str,
) -> ImportResolution<'a> {
    match resolve_rust_module_file(index, current_file_path, import_path) {
        RustModuleFileResolution::Resolved(file_path) => {
            if let Some(package) = package_for_rust_file(index, &file_path) {
                return ImportResolution::Resolved(package);
            }
        }
        RustModuleFileResolution::Ambiguous(file_paths) => {
            let mut candidates = file_paths
                .into_iter()
                .filter_map(|file_path| package_for_rust_file(index, &file_path))
                .collect::<Vec<_>>();
            candidates.sort_by(|left, right| {
                left.directory
                    .cmp(&right.directory)
                    .then(left.package_name.cmp(&right.package_name))
            });
            candidates.dedup_by(|left, right| {
                left.directory == right.directory && left.package_name == right.package_name
            });
            return match candidates.len() {
                0 => ImportResolution::Unresolved,
                1 => ImportResolution::Resolved(candidates[0]),
                _ => ImportResolution::Ambiguous(candidates),
            };
        }
        RustModuleFileResolution::Unresolved => {}
    }

    legacy_resolve_rust_import(index, current_file_path, import_path)
}

pub(crate) fn resolve_rust_module_file(
    index: &RepositoryIndex,
    current_file_path: &Path,
    import_path: &str,
) -> RustModuleFileResolution {
    let segments = import_path
        .split("::")
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let Some(head) = segments.first().copied() else {
        return RustModuleFileResolution::Unresolved;
    };

    let mut current_files = match head {
        "crate" => index
            .rust_crate_roots
            .get(current_file_path)
            .cloned()
            .unwrap_or_default(),
        "self" => vec![current_file_path.to_path_buf()],
        "super" => {
            let super_count = segments
                .iter()
                .take_while(|segment| **segment == "super")
                .count();
            let mut parents = vec![current_file_path.to_path_buf()];
            for _ in 0..super_count {
                let mut next = Vec::new();
                for parent in &parents {
                    if let Some(candidates) = index.rust_parent_modules.get(parent) {
                        next.extend(candidates.iter().cloned());
                    }
                }
                if next.is_empty() {
                    return RustModuleFileResolution::Unresolved;
                }
                next.sort();
                next.dedup();
                parents = next;
            }
            parents
        }
        _ => Vec::new(),
    };

    if current_files.is_empty() {
        return RustModuleFileResolution::Unresolved;
    }

    let start_index = if head == "super" {
        segments
            .iter()
            .take_while(|segment| **segment == "super")
            .count()
    } else {
        1
    };
    for segment in segments.iter().skip(start_index) {
        let mut next_files = Vec::new();
        for current in &current_files {
            if let Some(children) = index.rust_child_modules.get(current)
                && let Some(candidates) = children.get(*segment)
            {
                next_files.extend(candidates.iter().cloned());
            }
        }
        if next_files.is_empty() {
            return RustModuleFileResolution::Unresolved;
        }
        next_files.sort();
        next_files.dedup();
        current_files = next_files;
    }

    match current_files.len() {
        0 => RustModuleFileResolution::Unresolved,
        1 => RustModuleFileResolution::Resolved(current_files.remove(0)),
        _ => RustModuleFileResolution::Ambiguous(current_files),
    }
}

fn legacy_resolve_rust_import<'a>(
    index: &'a RepositoryIndex,
    current_file_path: &Path,
    import_path: &str,
) -> ImportResolution<'a> {
    let Some((crate_root, current_module_segments)) =
        rust_module_context(&index.root, current_file_path)
    else {
        return ImportResolution::Unresolved;
    };
    let Some(target_segments) = normalize_rust_path(import_path, &current_module_segments) else {
        return ImportResolution::Unresolved;
    };
    if target_segments.is_empty() {
        let candidates = index
            .packages
            .values()
            .filter(|package| package.language == Language::Rust && package.directory == crate_root)
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
    let mut candidates = index
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

pub(crate) fn summary(index: &RepositoryIndex) -> IndexSummary {
    let package_count = index.packages.len();
    let symbol_count = index
        .packages
        .values()
        .map(|package| package.symbols.len())
        .sum();
    let import_count = index
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
