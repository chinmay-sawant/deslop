mod architecture;
mod concurrency;
mod consistency;
mod context;
mod errors;
mod framework_patterns;
mod idioms;
mod library_misuse;
mod performance;
mod performance_extra;
mod style;

pub(crate) const BINDING_LOCATION: &str = file!();
pub(crate) const ARCHITECTURE_BINDING_LOCATION: &str = architecture::BINDING_LOCATION;
pub(crate) const CONCURRENCY_BINDING_LOCATION: &str = concurrency::BINDING_LOCATION;
pub(crate) const CONSISTENCY_BINDING_LOCATION: &str = consistency::BINDING_LOCATION;
pub(crate) const CONTEXT_BINDING_LOCATION: &str = context::BINDING_LOCATION;
pub(crate) const ERRORS_BINDING_LOCATION: &str = errors::BINDING_LOCATION;
pub(crate) const DATA_ACCESS_BINDING_LOCATION: &str =
    framework_patterns::DATA_ACCESS_BINDING_LOCATION;
pub(crate) const DATA_ACCESS_CLIENTS_BINDING_LOCATION: &str =
    framework_patterns::DATA_ACCESS_CLIENTS_BINDING_LOCATION;
pub(crate) const DATA_ACCESS_GORM_BINDING_LOCATION: &str =
    framework_patterns::DATA_ACCESS_GORM_BINDING_LOCATION;
pub(crate) const DATA_ACCESS_SQL_BINDING_LOCATION: &str =
    framework_patterns::DATA_ACCESS_SQL_BINDING_LOCATION;
pub(crate) const GIN_BINDING_LOCATION: &str = framework_patterns::GIN_BINDING_LOCATION;
pub(crate) const HOT_PATH_BINDING_LOCATION: &str = framework_patterns::HOT_PATH_BINDING_LOCATION;
pub(crate) const IDIOMS_BINDING_LOCATION: &str = idioms::BINDING_LOCATION;
pub(crate) const LIBRARY_MISUSE_LIBRARY_BINDING_LOCATION: &str =
    library_misuse::LIBRARY_BINDING_LOCATION;
pub(crate) const LIBRARY_MISUSE_SECURITY_BINDING_LOCATION: &str =
    library_misuse::SECURITY_BINDING_LOCATION;
pub(crate) const PERFORMANCE_BINDING_LOCATION: &str = performance::BINDING_LOCATION;
pub(crate) const PERFORMANCE_EXTRA_BINDING_LOCATION: &str = performance_extra::BINDING_LOCATION;
pub(crate) const STYLE_BINDING_LOCATION: &str = style::BINDING_LOCATION;

pub(super) use architecture::{go_architecture_file_findings, go_architecture_repo_findings};
pub(super) use concurrency::{
    coordination_findings, deeper_goroutine_lifetime_findings, mutex_findings, shutdown_findings,
};
pub(super) use consistency::{receiver_findings, tag_findings};
pub(super) use context::{
    busy_findings, cache_context_file_findings, cache_method_findings, cancel_findings,
    ctx_findings, propagate_findings, sleep_findings,
};
pub(super) use errors::error_findings;
pub(super) use framework_patterns::go_framework_patterns_file_findings;
pub(super) use idioms::{go_file_findings, go_repo_findings};
pub(super) use library_misuse::go_library_misuse_file_findings;
pub(super) use performance::{
    alloc_findings, concat_findings, db_findings, fmt_findings, json_findings, load_findings,
    n_squared_findings, reflect_findings,
};
pub(super) use performance_extra::extra_performance_findings;
pub(super) use style::{import_grouping_findings, package_name_consistency};
