pub mod benchmark;
mod analysis;
mod heuristics;
mod index;
mod model;
mod scan;

pub use benchmark::{benchmark_repository, BenchmarkOptions};
pub use model::{
	BenchmarkReport, FileReport, Finding, FunctionFingerprint, IndexSummary, ParseFailure,
	ScanOptions, ScanReport, StageStats, TimingBreakdown,
};
pub use scan::scan_repository;