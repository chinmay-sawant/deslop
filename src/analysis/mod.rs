mod go;
mod types;

use std::path::Path;

use anyhow::Result;

pub(crate) use types::{
    CallSite, DeclaredSymbol, FormattedErrorCall, ImportSpec, ParsedFile, ParsedFunction,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Language {
    Go,
    Python,
}

pub(crate) trait Analyzer: Send + Sync {
    fn supports_path(&self, path: &Path) -> bool;

    fn parse_file(&self, path: &Path, source: &str) -> Result<ParsedFile>;
}

fn language_for_path(path: &Path) -> Option<Language> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("go") => Some(Language::Go),
        Some("py") => Some(Language::Python),
        _ => None,
    }
}

pub(crate) fn analyzer_for_path(path: &Path) -> Option<&'static dyn Analyzer> {
    static GO_ANALYZER: go::GoAnalyzer = go::GoAnalyzer;

    match language_for_path(path) {
        Some(Language::Go) => GO_ANALYZER.supports_path(path).then_some(&GO_ANALYZER),
        Some(Language::Python) | None => None,
    }
}
