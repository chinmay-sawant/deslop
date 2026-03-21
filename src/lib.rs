mod analysis;
pub mod benchmark;
mod heuristics;
mod index;
mod model;
mod scan;

pub use benchmark::{BenchmarkOptions, benchmark_repository};
pub use model::{
    BenchmarkReport, FileReport, Finding, FunctionFingerprint, IndexSummary, ParseFailure,
    ScanOptions, ScanReport, Severity, StageStats, TimingBreakdown,
};
pub use scan::scan_repository;
