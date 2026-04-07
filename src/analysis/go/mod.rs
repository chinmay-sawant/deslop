mod fingerprint;
mod parser;

use std::path::Path;

use crate::analysis::{Language, LanguageBackend, ParsedFile};

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
}
