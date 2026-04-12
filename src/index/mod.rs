mod build;
mod resolve;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::analysis::{DeclaredSymbol, ImportSpec, Language};
use crate::model::IndexSummary;

pub(crate) use build::build_repository_index;
pub(crate) use resolve::RustModuleFileResolution;

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
    rust_package_names_by_file: BTreeMap<PathBuf, String>,
    rust_imports_by_file: BTreeMap<PathBuf, Vec<ImportSpec>>,
    rust_include_neighbors: BTreeMap<PathBuf, Vec<PathBuf>>,
    rust_child_modules: BTreeMap<PathBuf, BTreeMap<String, Vec<PathBuf>>>,
    rust_parent_modules: BTreeMap<PathBuf, Vec<PathBuf>>,
    rust_crate_roots: BTreeMap<PathBuf, Vec<PathBuf>>,
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

    pub(crate) fn resolve_rust_module_file(
        &self,
        current_file_path: &Path,
        import_path: &str,
    ) -> RustModuleFileResolution {
        resolve::resolve_rust_module_file(self, current_file_path, import_path)
    }

    pub(crate) fn package_for_rust_file(&self, file_path: &Path) -> Option<&PackageIndex> {
        resolve::package_for_rust_file(self, file_path)
    }

    pub(crate) fn rust_imports_for_file(&self, file_path: &Path) -> &[ImportSpec] {
        self.rust_imports_by_file
            .get(file_path)
            .map_or(&[], Vec::as_slice)
    }

    pub(crate) fn rust_file_uses_textual_includes(&self, file_path: &Path) -> bool {
        resolve::rust_file_uses_textual_includes(self, file_path)
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
mod tests;
