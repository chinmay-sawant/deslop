mod parser;

pub(crate) const EVALUATE_BINDING_LOCATION: &str =
    crate::heuristics::rust::EVALUATE_BINDING_LOCATION;
pub(crate) const FINDINGS_BINDING_LOCATION: &str =
    crate::analysis::rust::findings::BINDING_LOCATION;

use std::path::Path;

use crate::analysis::{Language, LanguageBackend, ParsedFile};

#[cfg(test)]
pub(crate) use crate::heuristics::rust::{alias_lookup, call_matches_import, import_matches_item};

pub(crate) mod findings;

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
}

#[cfg(test)]
fn evaluate_rust_findings(
    file: &ParsedFile,
    index: &crate::index::RepositoryIndex,
) -> Vec<crate::model::Finding> {
    crate::heuristics::evaluate_file(file, index, &crate::analysis::AnalysisConfig::default())
}

#[cfg(test)]
mod tests;
