mod concurrency;
mod consistency;
mod context;
mod errors;
mod framework_patterns;
mod idioms;
mod library_misuse;
mod performance;
mod style;

pub(super) use concurrency::{
    coordination_findings, deeper_goroutine_lifetime_findings, mutex_findings, shutdown_findings,
};
pub(super) use consistency::{receiver_findings, tag_findings};
pub(super) use context::{
    busy_findings, cancel_findings, ctx_findings, propagate_findings, sleep_findings,
};
pub(super) use errors::error_findings;
pub(super) use framework_patterns::go_framework_patterns_file_findings;
pub(super) use idioms::{go_file_findings, go_repo_findings};
pub(super) use library_misuse::go_library_misuse_file_findings;
pub(super) use performance::{
    alloc_findings, concat_findings, db_findings, fmt_findings, json_findings, load_findings,
    n_squared_findings, reflect_findings,
};
pub(super) use style::{import_grouping_findings, package_name_consistency};
