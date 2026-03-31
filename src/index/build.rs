use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::analysis::{DeclaredSymbol, ParsedFile, ParsedFunction};
use crate::model::SymbolKind;

use super::{PackageIndex, PackageKey, RepositoryIndex};

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

pub(super) fn package_directory(root: &Path, file_path: &Path) -> PathBuf {
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

fn is_contextless_wrapper_candidate(file: &ParsedFile, function: &ParsedFunction) -> bool {
    if function.has_context_parameter {
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
