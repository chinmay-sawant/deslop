mod evaluate;
mod file_analysis;
mod reporting;
mod suppression;
mod walker;

use std::time::Instant;

use crate::analysis::{AnalysisConfig, supported_extensions};
use crate::index::build_repository_index;
use crate::model::{ScanOptions, ScanReport, TimingBreakdown};
use crate::scan::walker::discover_source_files;
use crate::{Result, load_repository_config};

#[cfg(test)]
use self::evaluate::apply_repository_config;
use self::evaluate::evaluate_findings;
use self::file_analysis::analyze_discovered_files;
#[cfg(test)]
use self::file_analysis::is_generated;
use self::reporting::file_reports;
#[cfg(test)]
use self::suppression::{
    SuppressionDirective, next_code_line, parse_rule_ids, parse_suppression_directives,
};

pub fn scan_repository(options: &ScanOptions) -> Result<ScanReport> {
    scan_repository_with_go_semantic(options, false)
}

pub fn scan_repository_with_go_semantic(
    options: &ScanOptions,
    enable_go_semantic: bool,
) -> Result<ScanReport> {
    let total_start = Instant::now();
    let canonical_root = options
        .root
        .canonicalize()
        .map_err(|error| crate::Error::io(&options.root, error))?;
    let repo_config = load_repository_config(&canonical_root)?;

    let discover_start = Instant::now();
    let supported_extensions = supported_extensions();
    let discovered_files = discover_source_files(
        &canonical_root,
        options.respect_ignore,
        &supported_extensions,
    )?;
    let discover_ms = discover_start.elapsed().as_millis();

    let parse_start = Instant::now();
    let (parsed_files, parse_failures, suppressions) = analyze_discovered_files(&discovered_files);
    let parse_ms = parse_start.elapsed().as_millis();

    let index_start = Instant::now();
    let index = build_repository_index(&canonical_root, &parsed_files);
    let index_summary = index.summary();
    let index_ms = index_start.elapsed().as_millis();

    let analysis_config = AnalysisConfig {
        enable_go_semantic: repo_config.go_semantic_experimental || enable_go_semantic,
    };

    let heuristics_start = Instant::now();
    let findings = evaluate_findings(
        &parsed_files,
        &index,
        &suppressions,
        &repo_config,
        &canonical_root,
        &analysis_config,
    );
    let heuristics_ms = heuristics_start.elapsed().as_millis();

    let files_analyzed = parsed_files.len();
    let functions_found = parsed_files.iter().map(|file| file.functions.len()).sum();
    let files = file_reports(&parsed_files);

    Ok(ScanReport {
        root: canonical_root,
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
#[cfg(test)]
mod tests;
