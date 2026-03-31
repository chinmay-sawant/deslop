use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::analysis::{ParsedFile, backend_for_path};
use crate::io::canonicalize_within_root;
use crate::model::ParseFailure;
use crate::{DEFAULT_MAX_BYTES, read_to_string_limited};

use super::suppression::{SuppressionDirective, parse_suppression_directives};

pub(super) fn analyze_discovered_files(
    discovered_files: &[PathBuf],
) -> (
    Vec<ParsedFile>,
    Vec<ParseFailure>,
    BTreeMap<PathBuf, Vec<SuppressionDirective>>,
) {
    let mut parsed_files = Vec::new();
    let mut parse_failures = Vec::new();
    let mut suppressions = BTreeMap::new();
    let mut outcomes = discovered_files
        .par_iter()
        .map(|path| analyze_file(path))
        .collect::<Vec<_>>();
    outcomes.sort_by(|left, right| left.path().cmp(right.path()));

    for outcome in outcomes {
        match outcome {
            FileOutcome::Parsed {
                file,
                suppressions: file_suppressions,
            } => {
                suppressions.insert(file.path.clone(), file_suppressions);
                parsed_files.push(*file);
            }
            FileOutcome::Generated(_) => {}
            FileOutcome::Failed(failure) => parse_failures.push(failure),
        }
    }

    (parsed_files, parse_failures, suppressions)
}

pub(super) fn is_generated(source: &str) -> bool {
    source.lines().take(5).any(|line| {
        let normalized = line.trim();
        normalized.contains("Code generated") && normalized.contains("DO NOT EDIT")
    })
}

enum FileOutcome {
    Parsed {
        file: Box<ParsedFile>,
        suppressions: Vec<SuppressionDirective>,
    },
    Generated(PathBuf),
    Failed(ParseFailure),
}

impl FileOutcome {
    fn path(&self) -> &Path {
        match self {
            Self::Parsed { file, .. } => &file.path,
            Self::Generated(path) => path,
            Self::Failed(failure) => &failure.path,
        }
    }
}

fn analyze_file(path: &Path) -> FileOutcome {
    let path = match canonicalize_within_root(path.parent().unwrap_or(path), path) {
        Ok(path) => path,
        Err(error) => {
            return FileOutcome::Failed(ParseFailure {
                path: path.to_path_buf(),
                message: error.to_string(),
            });
        }
    };

    match read_to_string_limited(&path, DEFAULT_MAX_BYTES) {
        Ok(source) => {
            if is_generated(&source) {
                return FileOutcome::Generated(path.clone());
            }

            let suppressions = parse_suppression_directives(&source);

            let Some(analyzer) = backend_for_path(&path) else {
                return FileOutcome::Failed(ParseFailure {
                    path: path.clone(),
                    message: format!("no analyzer registered for {}", path.display()),
                });
            };

            match analyzer.parse_file(&path, &source) {
                Ok(file) => FileOutcome::Parsed {
                    file: Box::new(file),
                    suppressions,
                },
                Err(error) => FileOutcome::Failed(ParseFailure {
                    path: path.clone(),
                    message: error.to_string(),
                }),
            }
        }
        Err(error) => FileOutcome::Failed(ParseFailure {
            path,
            message: error.to_string(),
        }),
    }
}
