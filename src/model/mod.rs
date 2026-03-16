use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub root: PathBuf,
    pub respect_ignore: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimingBreakdown {
    pub discover_ms: u128,
    pub parse_ms: u128,
    pub index_ms: u128,
    pub heuristics_ms: u128,
    pub total_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Interface,
    Type,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseFailure {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionFingerprint {
    pub name: String,
    pub kind: String,
    pub receiver_type: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub line_count: usize,
    pub comment_lines: usize,
    pub code_lines: usize,
    pub comment_to_code_ratio: f64,
    pub complexity_score: usize,
    pub symmetry_score: f64,
    pub boilerplate_err_guards: usize,
    pub contains_any_type: bool,
    pub contains_empty_interface: bool,
    pub type_assertion_count: usize,
    pub call_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub path: PathBuf,
    pub function_name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub message: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexSummary {
    pub package_count: usize,
    pub symbol_count: usize,
    pub import_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileReport {
    pub path: PathBuf,
    pub package_name: Option<String>,
    pub syntax_error: bool,
    pub byte_size: usize,
    pub functions: Vec<FunctionFingerprint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub root: PathBuf,
    pub files_discovered: usize,
    pub files_analyzed: usize,
    pub functions_found: usize,
    pub files: Vec<FileReport>,
    pub findings: Vec<Finding>,
    pub index_summary: IndexSummary,
    pub parse_failures: Vec<ParseFailure>,
    pub timings: TimingBreakdown,
}

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