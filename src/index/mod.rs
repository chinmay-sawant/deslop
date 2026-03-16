use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{DeclaredSymbol, ParsedFile};
use crate::model::{IndexSummary, SymbolKind};

#[derive(Debug, Clone)]
pub(crate) struct PackageIndex {
    pub package_name: String,
    pub functions: BTreeSet<String>,
    pub methods_by_receiver: BTreeMap<String, BTreeSet<String>>,
    pub symbols: Vec<DeclaredSymbol>,
    pub import_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct RepositoryIndex {
    packages: BTreeMap<String, PackageIndex>,
}

impl RepositoryIndex {
    pub fn package(&self, package_name: &str) -> Option<&PackageIndex> {
        self.packages.get(package_name)
    }

    pub fn resolve_import_alias(&self, alias: &str) -> Option<&PackageIndex> {
        self.packages.get(alias)
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
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains(name)
    }

    pub fn has_method(&self, receiver: &str, name: &str) -> bool {
        self.methods_by_receiver
            .get(receiver)
            .is_some_and(|methods| methods.contains(name))
    }
}

pub(crate) fn build_repository_index(files: &[ParsedFile]) -> RepositoryIndex {
    let mut packages = BTreeMap::new();

    for file in files {
        let package_name = file
            .package_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let package_entry = packages.entry(package_name.clone()).or_insert_with(|| PackageIndex {
            package_name,
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

    RepositoryIndex { packages }
}

#[cfg(test)]
mod tests {
    use super::build_repository_index;
    use crate::analysis::{DeclaredSymbol, ParsedFile, ParsedFunction};
    use crate::model::{FunctionFingerprint, SymbolKind};

    #[test]
    fn builds_package_lookup() {
        let files = vec![ParsedFile {
            path: "sample.go".into(),
            package_name: Some("utils".to_string()),
            syntax_error: false,
            byte_size: 10,
            functions: vec![ParsedFunction {
                fingerprint: FunctionFingerprint {
                    name: "Trim".to_string(),
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
            }],
            imports: Vec::new(),
            symbols: vec![DeclaredSymbol {
                name: "Trim".to_string(),
                kind: SymbolKind::Function,
                receiver_type: None,
                line: 1,
            }],
        }];

        let index = build_repository_index(&files);
        assert!(index.package("utils").is_some_and(|package| package.has_function("Trim")));
    }
}