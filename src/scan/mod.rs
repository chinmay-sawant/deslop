mod walker;

use std::fs;
use std::time::Instant;

use anyhow::{Context, Result};
use rayon::prelude::*;

use crate::analysis::{ParsedFile, backend_for_language, backend_for_path, supported_extensions};
use crate::heuristics::evaluate_shared_findings;
use crate::index::build_repository_index;
use crate::model::{Finding, ParseFailure, ScanOptions, ScanReport, TimingBreakdown};
use crate::scan::walker::discover_source_files;

pub fn scan_repository(options: &ScanOptions) -> Result<ScanReport> {
    let total_start = Instant::now();

    let discover_start = Instant::now();
    let supported_extensions = supported_extensions();
    let discovered_files = discover_source_files(
        &options.root,
        options.respect_ignore,
        &supported_extensions,
    )
        .with_context(|| format!("failed to walk {}", options.root.display()))?;
    let discover_ms = discover_start.elapsed().as_millis();

    let parse_start = Instant::now();
    let mut parsed_files = Vec::new();
    let mut parse_failures = Vec::new();
    let mut outcomes = discovered_files
        .par_iter()
        .map(|path| analyze_file(path))
        .collect::<Vec<_>>();
    outcomes.sort_by(|left, right| left.path().cmp(right.path()));

    for outcome in outcomes {
        match outcome {
            FileOutcome::Parsed(file) => parsed_files.push(file),
            FileOutcome::Generated(_) => {}
            FileOutcome::Failed(failure) => parse_failures.push(failure),
        }
    }
    let parse_ms = parse_start.elapsed().as_millis();

    let index_start = Instant::now();
    let index = build_repository_index(&options.root, &parsed_files);
    let index_summary = index.summary();
    let index_ms = index_start.elapsed().as_millis();

    let heuristics_start = Instant::now();
    let findings = evaluate_findings(&parsed_files, &index);
    let heuristics_ms = heuristics_start.elapsed().as_millis();

    let files_analyzed = parsed_files.len();
    let functions_found = parsed_files.iter().map(|file| file.functions.len()).sum();
    let files = parsed_files.iter().map(ParsedFile::to_report).collect();

    Ok(ScanReport {
        root: options.root.clone(),
        files_discovered: discovered_files.len(),
        files_analyzed,
        functions_found,
        files,
        findings,
        index_summary,
        parse_failures,
        timings: TimingBreakdown {
            discover_ms,
            parse_ms,
            index_ms,
            heuristics_ms,
            total_ms: total_start.elapsed().as_millis(),
        },
    })
}

fn evaluate_findings(files: &[ParsedFile], index: &crate::index::RepositoryIndex) -> Vec<Finding> {
    let mut findings = evaluate_shared_findings(files, index);

    for file in files {
        if let Some(backend) = backend_for_language(file.language) {
            findings.extend(backend.evaluate_file_findings(file, index));
        }
    }

    for backend in crate::analysis::registered_backends() {
        let backend_files = files
            .iter()
            .filter(|file| file.language == backend.language())
            .collect::<Vec<_>>();
        findings.extend(backend.evaluate_repository_findings(&backend_files, index));
    }

    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    findings
}

enum FileOutcome {
    Parsed(ParsedFile),
    Generated(std::path::PathBuf),
    Failed(ParseFailure),
}

impl FileOutcome {
    fn path(&self) -> &std::path::Path {
        match self {
            Self::Parsed(file) => &file.path,
            Self::Generated(path) => path,
            Self::Failed(failure) => &failure.path,
        }
    }
}

fn analyze_file(path: &std::path::Path) -> FileOutcome {
    match fs::read_to_string(path) {
        Ok(source) => {
            if is_generated_source(&source) {
                return FileOutcome::Generated(path.to_path_buf());
            }

            let Some(analyzer) = backend_for_path(path) else {
                return FileOutcome::Failed(ParseFailure {
                    path: path.to_path_buf(),
                    message: format!("no analyzer registered for {}", path.display()),
                });
            };

            match analyzer.parse_file(path, &source) {
                Ok(file) => FileOutcome::Parsed(file),
                Err(error) => FileOutcome::Failed(ParseFailure {
                    path: path.to_path_buf(),
                    message: error.to_string(),
                }),
            }
        }
        Err(error) => FileOutcome::Failed(ParseFailure {
            path: path.to_path_buf(),
            message: error.to_string(),
        }),
    }
}

fn is_generated_source(source: &str) -> bool {
    source.lines().take(5).any(|line| {
        let normalized = line.trim();
        normalized.contains("Code generated") && normalized.contains("DO NOT EDIT")
    })
}

#[cfg(test)]
mod tests {
    use super::is_generated_source;

    #[test]
    fn detects_generated_files() {
        let generated = "// Code generated by mockery. DO NOT EDIT.\npackage sample\n";
        assert!(is_generated_source(generated));
    }
}
