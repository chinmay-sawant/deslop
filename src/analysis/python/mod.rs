mod parser;

use std::path::Path;

use crate::analysis::{Language, LanguageBackend, ParsedFile};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PythonAnalyzer;

impl LanguageBackend for PythonAnalyzer {
    fn language(&self) -> Language {
        Language::Python
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["py"]
    }

    fn supports_path(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("py")
    }

    fn parse_file(&self, path: &Path, source: &str) -> crate::Result<ParsedFile> {
        parser::parse_file(path, source).map_err(crate::Error::from)
    }
}

#[cfg(test)]
mod tests;
