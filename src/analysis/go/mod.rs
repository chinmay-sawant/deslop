mod fingerprint;
mod parser;

use std::path::Path;

use crate::analysis::{Language, LanguageBackend, ParsedFile};
use crate::heuristics::{evaluate_go_file, evaluate_go_repo};
use crate::index::RepositoryIndex;
use crate::model::Finding;

#[derive(Debug, Clone, Copy)]
pub(crate) struct GoAnalyzer;

impl LanguageBackend for GoAnalyzer {
    fn language(&self) -> Language {
        Language::Go
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["go"]
    }

    fn supports_path(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("go")
    }

    fn parse_file(&self, path: &Path, source: &str) -> crate::Result<ParsedFile> {
        parser::parse_file(path, source).map_err(crate::Error::from)
    }

    fn evaluate_file(&self, file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
        evaluate_go_file(file, index)
    }

    fn evaluate_repo(&self, files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
        evaluate_go_repo(files, index)
    }
}
