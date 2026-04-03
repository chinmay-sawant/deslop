use crate::analysis::{ImportSpec, ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use super::comments::comment_findings;
use super::go::{
    alloc_findings, busy_findings, cancel_findings, concat_findings, coordination_findings,
    ctx_findings, db_findings, deeper_goroutine_lifetime_findings, error_findings, fmt_findings,
    go_framework_patterns_file_findings, go_library_misuse_file_findings, go_file_findings,
    go_repo_findings, import_grouping_findings, json_findings, load_findings, mutex_findings,
    n_squared_findings, package_name_consistency, propagate_findings, receiver_findings,
    reflect_findings, shutdown_findings, sleep_findings, tag_findings,
};
use super::hallucination::hallucination_findings;
use super::naming::{generic_finding, overlong_finding, weak_finding};
use super::python::{python_file_findings, python_findings, python_repo_findings};
use super::security::{crypto_findings, pkg_secret_findings, secret_findings, sql_findings};
use super::test_quality::test_findings;

pub(super) type FileRule = fn(&ParsedFile) -> Vec<Finding>;
pub(super) type FunctionRule = fn(&ParsedFile, &ParsedFunction) -> Vec<Finding>;
pub(super) type OptionalFunctionRule = fn(&ParsedFile, &ParsedFunction) -> Option<Finding>;
pub(super) type IndexedFunctionRule =
    fn(&ParsedFile, &ParsedFunction, &RepositoryIndex) -> Vec<Finding>;
pub(super) type FileFunctionRule = fn(&ParsedFile, &ParsedFunction, &[ImportSpec]) -> Vec<Finding>;
pub(super) type ConfigurableFunctionRule = fn(&ParsedFile, &ParsedFunction, bool) -> Vec<Finding>;
pub(super) type RepoRule = fn(&[&ParsedFile]) -> Vec<Finding>;
pub(super) type IndexedRepoRule = fn(&[&ParsedFile], &RepositoryIndex) -> Vec<Finding>;

pub(super) const SHARED_FILE_RULES: &[FileRule] = &[pkg_secret_findings];

pub(super) const SHARED_OPTIONAL_FUNCTION_RULES: &[OptionalFunctionRule] =
    &[generic_finding, overlong_finding, weak_finding];

pub(super) const SHARED_FUNCTION_RULES: &[FunctionRule] =
    &[comment_findings, secret_findings, test_findings];

pub(super) const GO_FILE_RULES: &[FileRule] = &[
    go_file_findings,
    go_framework_patterns_file_findings,
    go_library_misuse_file_findings,
    tag_findings,
    import_grouping_findings,
];

pub(super) const GO_FUNCTION_RULES: &[FunctionRule] = &[
    error_findings,
    crypto_findings,
    sql_findings,
    ctx_findings,
    cancel_findings,
    sleep_findings,
    busy_findings,
    shutdown_findings,
    deeper_goroutine_lifetime_findings,
    alloc_findings,
    fmt_findings,
    reflect_findings,
    concat_findings,
    json_findings,
    load_findings,
    coordination_findings,
];

pub(super) const GO_FILE_FUNCTION_RULES: &[FileFunctionRule] = &[mutex_findings];

pub(super) const GO_INDEXED_FUNCTION_RULES: &[IndexedFunctionRule] =
    &[propagate_findings, hallucination_findings];

pub(super) const GO_CONFIGURABLE_FUNCTION_RULES: &[ConfigurableFunctionRule] =
    &[n_squared_findings, db_findings];

pub(super) const GO_REPO_RULES: &[RepoRule] = &[
    receiver_findings,
    package_name_consistency,
    go_repo_findings,
];

pub(super) const PYTHON_FILE_RULES: &[FileRule] = &[python_file_findings];

pub(super) const PYTHON_FUNCTION_RULES: &[FunctionRule] = &[python_findings];

pub(super) const PYTHON_INDEXED_FUNCTION_RULES: &[IndexedFunctionRule] = &[hallucination_findings];

pub(super) const PYTHON_REPO_RULES: &[IndexedRepoRule] = &[python_repo_findings];
