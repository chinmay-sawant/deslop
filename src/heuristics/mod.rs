mod comments;
mod common;
mod consistency;
mod concurrency;
mod context;
mod errors;
mod hallucination;
mod naming;
mod performance;
mod security;
mod test_quality;
#[cfg(test)]
mod tests;

use crate::analysis::ParsedFile;
use crate::index::RepositoryIndex;
use crate::model::Finding;

use self::comments::comment_style_findings;
use self::consistency::{mixed_receiver_kind_findings, struct_tag_findings};
use self::concurrency::{
    goroutine_coordination_findings, goroutine_shutdown_findings, mutex_contention_findings,
};
use self::context::{
    busy_waiting_findings, missing_cancel_call_findings, missing_context_findings,
    sleep_polling_findings,
};
use self::errors::error_handling_findings;
use self::hallucination::local_hallucination_findings;
use self::naming::{generic_name_finding, overlong_name_finding, weak_typing_finding};
use self::performance::{
    allocation_churn_findings, database_query_findings, fmt_hot_path_findings,
    full_dataset_load_findings, reflection_hot_path_findings,
    repeated_json_marshaling_findings, string_concat_in_loop_findings,
};
use self::security::{
    hardcoded_secret_findings, package_hardcoded_secret_findings, sql_string_concat_findings,
    weak_crypto_findings,
};
use self::test_quality::test_quality_findings;

pub(crate) fn evaluate_findings(files: &[ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    for file in files {
        findings.extend(package_hardcoded_secret_findings(file));
        findings.extend(struct_tag_findings(file));

        for function in &file.functions {
            if let Some(finding) = generic_name_finding(file, function) {
                findings.push(finding);
            }

            if let Some(finding) = overlong_name_finding(file, function) {
                findings.push(finding);
            }

            if let Some(finding) = weak_typing_finding(file, function) {
                findings.push(finding);
            }

            findings.extend(error_handling_findings(file, function));
            findings.extend(comment_style_findings(file, function));
            findings.extend(weak_crypto_findings(file, function));
            findings.extend(hardcoded_secret_findings(file, function));
            findings.extend(sql_string_concat_findings(file, function));
            findings.extend(missing_context_findings(file, function));
            findings.extend(missing_cancel_call_findings(file, function));
            findings.extend(sleep_polling_findings(file, function));
            findings.extend(busy_waiting_findings(file, function));
            findings.extend(goroutine_shutdown_findings(file, function));
            findings.extend(mutex_contention_findings(file, function, &file.imports));
            findings.extend(allocation_churn_findings(file, function));
            findings.extend(fmt_hot_path_findings(file, function));
            findings.extend(reflection_hot_path_findings(file, function));
            findings.extend(string_concat_in_loop_findings(file, function));
            findings.extend(repeated_json_marshaling_findings(file, function));
            findings.extend(database_query_findings(file, function));
            findings.extend(full_dataset_load_findings(file, function));
            findings.extend(goroutine_coordination_findings(file, function));
            findings.extend(test_quality_findings(file, function));
            findings.extend(local_hallucination_findings(file, function, index));
        }
    }

    findings.extend(mixed_receiver_kind_findings(files));

    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    findings
}