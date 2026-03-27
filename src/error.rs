use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Analysis(#[from] crate::AnalysisError),
    #[error(transparent)]
    Config(#[from] crate::ConfigError),
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
    #[error("input file {path} exceeded the {max_bytes}-byte limit ({size} bytes)")]
    InputTooLarge {
        path: PathBuf,
        size: u64,
        max_bytes: u64,
    },
    #[error("refusing to follow symlinked path {path}")]
    SymlinkRejected { path: PathBuf },
    #[error("path {path} resolves outside scan root {root}")]
    PathOutsideRoot { path: PathBuf, root: PathBuf },
    #[error("byte count conversion overflowed for {path}: {value}")]
    ByteCountOverflow { path: PathBuf, value: usize },
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

    pub(crate) fn byte_count_overflow(path: impl AsRef<Path>, value: usize) -> Self {
        Self::ByteCountOverflow {
            path: path.as_ref().to_path_buf(),
            value,
        }
    }

    pub(crate) fn symlink_rejected(path: impl AsRef<Path>) -> Self {
        Self::SymlinkRejected {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub(crate) fn path_outside_root(
        path: impl AsRef<Path>,
        root: impl AsRef<Path>,
    ) -> Self {
        Self::PathOutsideRoot {
            path: path.as_ref().to_path_buf(),
            root: root.as_ref().to_path_buf(),
        }
    }
}