use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("directory walk failed for {path}: {source}")]
    Walk {
        path: PathBuf,
        #[source]
        source: ignore::Error,
    },
    #[error("failed to configure {language} parser: {message}")]
    ParserConfiguration { language: &'static str, message: String },
    #[error("tree-sitter returned no parse tree for {language}")]
    MissingParseTree { language: &'static str },
    #[error("input file {path} exceeded the {max_bytes}-byte limit ({size} bytes)")]
    InputTooLarge {
        path: PathBuf,
        size: u64,
        max_bytes: u64,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn io(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }

    pub(crate) fn walk(path: impl AsRef<Path>, source: ignore::Error) -> Self {
        Self::Walk {
            path: path.as_ref().to_path_buf(),
            source,
        }
    }

    pub(crate) fn parser_configuration(language: &'static str, message: impl Into<String>) -> Self {
        Self::ParserConfiguration {
            language,
            message: message.into(),
        }
    }

    pub(crate) fn missing_parse_tree(language: &'static str) -> Self {
        Self::MissingParseTree { language }
    }
}