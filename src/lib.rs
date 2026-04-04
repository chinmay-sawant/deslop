//! Crate architecture overview.
//!
//! The crate is intentionally layered from parsing to reporting:
//!
//! - `analysis` owns language-specific parsing and evidence extraction.
//! - `index` resolves files, packages, and import relationships across a repository.
//! - `heuristics` turns parsed evidence into findings for each language and rule family.
//! - `scan` orchestrates repository traversal, parsing, indexing, evaluation, and reporting.
//! - `cli` formats the public command-line experience around the library entry points.
//!
//! Each subsystem keeps its facade small and pushes implementation detail into
//! focused modules. When expanding the crate, prefer adding a new leaf module
//! over growing a facade into a catch-all file.

mod analysis;
pub mod benchmark;
mod config;
mod error;
mod heuristics;
mod index;
mod io;
mod model;
mod rules;
mod scan;

pub use analysis::Error as AnalysisError;
pub use analysis::syntax_error_for_source;
pub use analysis::validate_source;
pub use benchmark::{
    BenchmarkOptions, benchmark_repository, benchmark_repository_with_go_semantic,
};
pub use config::Error as ConfigError;
pub(crate) use config::{RepoConfig, load_repository_config};
pub use error::{Error, Result};
pub use io::{DEFAULT_MAX_BYTES, read_to_string_limited};
pub use model::{
    BenchmarkReport, FileReport, Finding, FunctionFingerprint, IndexSummary, ParseFailure,
    ScanOptions, ScanReport, Severity, StageStats, TimingBreakdown,
};
pub use rules::{
    RuleConfigurability, RuleDefaultSeverity, RuleLanguage, RuleMetadata, RuleStatus,
    is_detail_only_rule, rule_binding_location, rule_metadata, rule_metadata_variants,
    rule_registry,
};
pub use scan::{scan_repository, scan_repository_with_go_semantic};
