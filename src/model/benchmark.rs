use std::path::PathBuf;

use serde::Serialize;

use super::scan::TimingBreakdown;

#[derive(Debug, Clone, Serialize)]
pub struct StageStats {
    pub min_ms: u128,
    pub max_ms: u128,
    pub mean_ms: f64,
    pub median_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkRun {
    pub iteration: usize,
    pub timings: TimingBreakdown,
    pub files_analyzed: usize,
    pub functions_found: usize,
    pub findings_found: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReport {
    pub root: PathBuf,
    pub warmups: usize,
    pub repeats: usize,
    pub files_analyzed: usize,
    pub functions_found: usize,
    pub findings_found: usize,
    pub discover: StageStats,
    pub parse: StageStats,
    pub index: StageStats,
    pub heuristics: StageStats,
    pub total: StageStats,
    pub runs: Vec<BenchmarkRun>,
}
