use std::path::PathBuf;

use anyhow::Result;

use crate::model::{BenchmarkReport, BenchmarkRun, StageStats};
use crate::{ScanOptions, scan_repository};

#[derive(Debug, Clone)]
pub struct BenchmarkOptions {
    pub root: PathBuf,
    pub repeats: usize,
    pub warmups: usize,
    pub respect_ignore: bool,
}

pub fn benchmark_repository(options: &BenchmarkOptions) -> Result<BenchmarkReport> {
    for _ in 0..options.warmups {
        let _ = scan_repository(&ScanOptions {
            root: options.root.clone(),
            respect_ignore: options.respect_ignore,
        })?;
    }

    let mut runs = Vec::new();

    for iteration in 0..options.repeats {
        let report = scan_repository(&ScanOptions {
            root: options.root.clone(),
            respect_ignore: options.respect_ignore,
        })?;

        runs.push(BenchmarkRun {
            iteration: iteration + 1,
            files_analyzed: report.files_analyzed,
            functions_found: report.functions_found,
            findings_found: report.findings.len(),
            timings: report.timings,
        });
    }

    let files_analyzed = runs.last().map(|run| run.files_analyzed).unwrap_or(0);
    let functions_found = runs.last().map(|run| run.functions_found).unwrap_or(0);
    let findings_found = runs.last().map(|run| run.findings_found).unwrap_or(0);

    Ok(BenchmarkReport {
        root: options.root.clone(),
        warmups: options.warmups,
        repeats: options.repeats,
        files_analyzed,
        functions_found,
        findings_found,
        discover: calculate_stats(runs.iter().map(|run| run.timings.discover_ms).collect()),
        parse: calculate_stats(runs.iter().map(|run| run.timings.parse_ms).collect()),
        index: calculate_stats(runs.iter().map(|run| run.timings.index_ms).collect()),
        heuristics: calculate_stats(runs.iter().map(|run| run.timings.heuristics_ms).collect()),
        total: calculate_stats(runs.iter().map(|run| run.timings.total_ms).collect()),
        runs,
    })
}

fn calculate_stats(mut samples: Vec<u128>) -> StageStats {
    if samples.is_empty() {
        return StageStats {
            min_ms: 0,
            max_ms: 0,
            mean_ms: 0.0,
            median_ms: 0.0,
        };
    }

    samples.sort_unstable();
    let min_ms = *samples.first().unwrap_or(&0);
    let max_ms = *samples.last().unwrap_or(&0);
    let sum: u128 = samples.iter().copied().sum();
    let mean_ms = sum as f64 / samples.len() as f64;
    let median_ms = if samples.len().is_multiple_of(2) {
        let upper = samples[samples.len() / 2];
        let lower = samples[(samples.len() / 2) - 1];
        (lower + upper) as f64 / 2.0
    } else {
        samples[samples.len() / 2] as f64
    };

    StageStats {
        min_ms,
        max_ms,
        mean_ms,
        median_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::calculate_stats;

    #[test]
    fn test_calc_stats() {
        let stats = calculate_stats(vec![3, 1, 2, 4]);
        assert_eq!(stats.min_ms, 1);
        assert_eq!(stats.max_ms, 4);
        assert_eq!(stats.median_ms, 2.5);
    }
}
