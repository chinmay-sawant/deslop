mod evaluate;
mod parser;

pub(crate) const EVALUATE_BINDING_LOCATION: &str = evaluate::BINDING_LOCATION;
pub(crate) const FINDINGS_BINDING_LOCATION: &str = evaluate::findings::BINDING_LOCATION;

use std::path::Path;

use crate::analysis::{AnalysisConfig, Language, LanguageBackend, ParsedFile};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use self::evaluate::evaluate_rust_findings;
#[cfg(test)]
use self::evaluate::{alias_lookup, call_matches_import, import_matches_item};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RustAnalyzer;

impl LanguageBackend for RustAnalyzer {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn supports_path(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("rs")
    }

    fn parse_file(&self, path: &Path, source: &str) -> crate::Result<ParsedFile> {
        parser::parse_file(path, source).map_err(crate::Error::from)
    }

    fn evaluate_file(
        &self,
        file: &ParsedFile,
        index: &RepositoryIndex,
        _analysis_config: &AnalysisConfig,
    ) -> Vec<Finding> {
        evaluate_rust_findings(file, index)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{alias_lookup, call_matches_import, evaluate_rust_findings, import_matches_item};
    use crate::analysis::rust::parser;
    use crate::index::build_repository_index;

    fn parse_file(path: &str, source: &str) -> crate::analysis::ParsedFile {
        let parsed = parser::parse_file(Path::new(path), source);
        assert!(parsed.is_ok(), "rust source should parse");
        match parsed {
            Ok(file) => file,
            Err(_) => unreachable!("asserted above"),
        }
    }

    #[test]
    fn grouped_imported_function_matches_current_module_index() {
        let current = parse_file(
            "/repo/src/analysis/go/mod.rs",
            r#"
use crate::heuristics::{evaluate_go_file, evaluate_go_repo};

fn run() {
    evaluate_go_file();
    evaluate_go_repo();
}
"#,
        );
        let heuristics = parse_file(
            "/repo/src/heuristics/mod.rs",
            r#"
pub fn evaluate_go_file() {}
pub fn evaluate_go_repo() {}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), heuristics]);
        let import_aliases = alias_lookup(&current.imports);
        let import_spec = import_aliases.get("evaluate_go_file");

        assert!(import_spec.is_some(), "grouped import should be indexed");
        let import_spec = match import_spec {
            Some(import_spec) => import_spec,
            None => unreachable!("asserted above"),
        };

        assert!(call_matches_import(&index, &current.path, import_spec));
        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| finding.rule_id == "hallucinated_import_call")
        );
    }

    #[test]
    fn self_group_imported_function_matches_current_module_index() {
        let current = parse_file(
            "/repo/src/analysis/go/parser/mod.rs",
            r#"
use self::general::{collect_calls, find_package_name};

fn run() {
    collect_calls();
    find_package_name();
}
"#,
        );
        let general = parse_file(
            "/repo/src/analysis/go/parser/general.rs",
            r#"
pub(super) fn collect_calls() {}
pub(super) fn find_package_name() {}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), general]);
        let import_aliases = alias_lookup(&current.imports);

        assert!(call_matches_import(
            &index,
            &current.path,
            match import_aliases.get("collect_calls") {
                Some(import_spec) => import_spec,
                None => unreachable!("collect_calls import should exist"),
            }
        ));
        assert!(call_matches_import(
            &index,
            &current.path,
            match import_aliases.get("find_package_name") {
                Some(import_spec) => import_spec,
                None => unreachable!("find_package_name import should exist"),
            }
        ));
        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| finding.rule_id == "hallucinated_import_call")
        );
    }

    #[test]
    fn imported_type_receiver_is_not_treated_as_missing_module() {
        let current = parse_file(
            "/repo/src/analysis/go/parser/mod.rs",
            r#"
use crate::analysis::Error;

fn run() {
    let _ = Error::parser_configuration();
}
"#,
        );
        let analysis = parse_file(
            "/repo/src/analysis/mod.rs",
            r#"
pub use error::Error;
mod error;
"#,
        );
        let error = parse_file(
            "/repo/src/analysis/error.rs",
            r#"
pub struct Error;

impl Error {
    pub fn parser_configuration() -> Self {
        Self
    }
}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), analysis, error]);
        let import_aliases = alias_lookup(&current.imports);
        let import_spec = import_aliases.get("Error");

        assert!(import_spec.is_some(), "type import should be indexed");
        let import_spec = match import_spec {
            Some(import_spec) => import_spec,
            None => unreachable!("asserted above"),
        };

        assert!(import_matches_item(&index, &current.path, import_spec));
        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| {
                    finding.rule_id == "hallucinated_import_call"
                        && finding.message.contains("Error::parser_configuration")
                })
        );
    }

    #[test]
    fn rust_prelude_drop_is_not_flagged_as_local_hallucination() {
        let current = parse_file(
            "/repo/src/lib.rs",
            r#"
pub fn release(value: String) {
    drop(value);
}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), std::slice::from_ref(&current));

        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| {
                    finding.rule_id == "hallucinated_local_call" && finding.message.contains("drop")
                })
        );
    }

    #[test]
    fn actual_go_module_imported_heuristic_call_is_resolved() {
        let current = parse_file("/repo/src/analysis/go/mod.rs", include_str!("../go/mod.rs"));
        let heuristics = parse_file(
            "/repo/src/heuristics/mod.rs",
            include_str!("../../heuristics/mod.rs"),
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), heuristics]);
        let import_aliases = alias_lookup(&current.imports);
        let import_spec = import_aliases.get("evaluate_go_file");

        assert!(
            import_spec.is_some(),
            "evaluate_go_file import should exist"
        );
        let import_spec = match import_spec {
            Some(import_spec) => import_spec,
            None => unreachable!("asserted above"),
        };

        assert!(call_matches_import(&index, &current.path, import_spec));
    }

    #[test]
    fn actual_go_parser_error_import_is_indexed_as_item() {
        let current = parse_file(
            "/repo/src/analysis/go/parser/mod.rs",
            include_str!("../go/parser/mod.rs"),
        );
        let analysis = parse_file("/repo/src/analysis/mod.rs", include_str!("../mod.rs"));
        let error = parse_file("/repo/src/analysis/error.rs", include_str!("../error.rs"));

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), analysis, error]);
        let import_aliases = alias_lookup(&current.imports);
        let import_spec = import_aliases.get("Error");

        assert!(import_spec.is_some(), "Error import should exist");
        let import_spec = match import_spec {
            Some(import_spec) => import_spec,
            None => unreachable!("asserted above"),
        };

        assert!(import_matches_item(&index, &current.path, import_spec));
    }

    #[test]
    fn nested_test_module_imports_do_not_leak_into_outer_function_checks() {
        let current = parse_file(
            "/repo/src/analysis/python/mod.rs",
            r#"
mod parser;

fn run() {
    let _ = parser::parse_file();
}

#[cfg(test)]
mod tests {
    use super::parser;
}
"#,
        );
        let parser_module = parse_file(
            "/repo/src/analysis/python/parser/mod.rs",
            r#"
pub fn parse_file() -> Result<(), ()> {
    Ok(())
}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), parser_module]);

        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| {
                    finding.rule_id == "hallucinated_import_call"
                        && finding.message.contains("parser::parse_file")
                })
        );
    }

    #[test]
    fn nested_test_module_super_import_matches_local_child_module() {
        let current = parse_file(
            "/repo/src/analysis/python/mod.rs",
            r#"
mod parser;

#[cfg(test)]
mod tests {
    use super::parser;

    fn parse_file() {
        let _ = parser::parse_file();
    }
}
"#,
        );
        let parser_module = parse_file(
            "/repo/src/analysis/python/parser/mod.rs",
            r#"
pub fn parse_file() -> Result<(), ()> {
    Ok(())
}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), parser_module]);
        let import_aliases = alias_lookup(&current.imports);
        let import_spec = match import_aliases.get("parser") {
            Some(import_spec) => import_spec,
            None => unreachable!("parser import should exist"),
        };

        assert!(import_matches_item(&index, &current.path, import_spec));
        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| {
                    finding.rule_id == "hallucinated_import_call"
                        && finding.message.contains("parser::parse_file")
                })
        );
    }

    #[test]
    fn function_scoped_import_before_call_is_not_flagged_as_local() {
        let current = parse_file(
            "/repo/src/io.rs",
            r#"
#[cfg(unix)]
fn create_link(target: &str, link: &str) {
    use std::os::unix::fs::symlink as create_fs_link;

    assert!(create_fs_link(target, link).is_ok());
}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), std::slice::from_ref(&current));

        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| {
                    finding.rule_id == "hallucinated_local_call"
                        && finding.message.contains("create_fs_link")
                })
        );
    }

    #[test]
    fn expression_receivers_are_not_treated_as_import_paths() {
        let current = parse_file(
            "/repo/src/lib.rs",
            r#"
mod parser {
    pub fn parse_file() -> Result<(), ()> {
        Ok(())
    }
}

pub fn run() {
    let _ = parser::parse_file().map_err(|_| ());
}
"#,
        );

        let index = build_repository_index(Path::new("/repo"), std::slice::from_ref(&current));

        assert!(
            !evaluate_rust_findings(&current, &index)
                .iter()
                .any(|finding| {
                    finding.rule_id == "hallucinated_import_call"
                        && finding.message.contains("parser::parse_file")
                })
        );
    }
}
