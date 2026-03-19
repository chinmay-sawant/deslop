mod fingerprint;
mod parser;

use std::path::Path;

use anyhow::Result;

use crate::analysis::{Analyzer, ParsedFile};

#[derive(Debug, Clone, Copy)]
pub(crate) struct GoAnalyzer;

impl Analyzer for GoAnalyzer {
    fn supports_path(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("go")
    }

    fn parse_file(&self, path: &Path, source: &str) -> Result<ParsedFile> {
        parser::parse_file(path, source)
    }
}
