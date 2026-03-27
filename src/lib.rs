mod analysis;
pub mod benchmark;
mod error;
mod heuristics;
mod index;
mod io;
mod model;
mod scan;

pub use benchmark::{BenchmarkOptions, benchmark_repository};
pub use error::{Error, Result};
pub use io::{DEFAULT_MAX_BYTES, read_to_string_limited};
pub use model::{
    BenchmarkReport, FileReport, Finding, FunctionFingerprint, IndexSummary, ParseFailure,
    ScanOptions, ScanReport, Severity, StageStats, TimingBreakdown,
};
pub use scan::scan_repository;
