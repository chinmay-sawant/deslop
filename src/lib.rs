mod analysis;
pub mod benchmark;
mod config;
mod error;
mod heuristics;
mod index;
mod io;
mod model;
mod scan;

pub use analysis::Error as AnalysisError;
pub use benchmark::{BenchmarkOptions, benchmark_repository};
pub use config::Error as ConfigError;
pub(crate) use config::{RepoConfig, load_repository_config};
pub use error::{Error, Result};
pub use io::{DEFAULT_MAX_BYTES, read_to_string_limited};
pub use model::{
    BenchmarkReport, FileReport, Finding, FunctionFingerprint, IndexSummary, ParseFailure,
    ScanOptions, ScanReport, Severity, StageStats, TimingBreakdown,
};
pub use scan::scan_repository;
