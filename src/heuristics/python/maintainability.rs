#[path = "maintainability/file_rules.rs"]
mod file_rules;
#[path = "maintainability/function_rules.rs"]
mod function_rules;
#[path = "maintainability/helpers.rs"]
mod helpers;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

pub(super) use file_rules::{commented_out_code_findings, sync_async_module_findings};
pub(super) use function_rules::{
    api_type_hint_findings, broad_exception_handler_findings, builtin_reduction_findings,
    env_fallback_findings, eval_exec_findings, exception_swallowed_findings,
    hardcoded_business_rule_findings, hardcoded_path_findings, input_validation_findings,
    magic_value_branching_findings, missing_context_manager_findings, network_timeout_findings,
    none_comparison_findings, print_debugging_findings, redundant_return_none_findings,
    reinvented_utility_findings, side_effect_comprehension_findings, variadic_public_api_findings,
};

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn maintainability_finding(
    file: &ParsedFile,
    function: &ParsedFunction,
    rule_id: &str,
    severity: Severity,
    message: &str,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.start_line,
        message: format!("function {} {message}", function.fingerprint.name),
        evidence: vec![format!("rule={rule_id}")],
    }
}

pub(super) fn project_agnostic_maintainability_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let body = &function.body_text;
    let lower_body = body.to_ascii_lowercase();
    let lower_name = function.fingerprint.name.to_ascii_lowercase();
    let sig = function.signature_text.replace('\n', " ");

    if lower_body.matches("0.95").count()
        + lower_body.matches("1000").count()
        + lower_body.matches("86400").count()
        >= 2
    {
        findings.push(maintainability_finding(
            file,
            function,
            "magic_thresholds_duplicated_across_modules",
            Severity::Info,
            "duplicates threshold-like numeric values that deserve named ownership",
        ));
    }

    if sig.contains("-> tuple") || body.contains("return (") && body.matches(',').count() >= 2 {
        findings.push(maintainability_finding(
            file,
            function,
            "tuple_return_with_three_or_more_positional_fields_in_public_api",
            Severity::Info,
            "returns a wide positional tuple from a public-looking API",
        ));
    }

    if contains_any(
        &lower_body,
        &["ids.append(", "names.append(", "values.append("],
    ) {
        findings.push(maintainability_finding(
            file,
            function,
            "parallel_lists_used_instead_of_record_object",
            Severity::Info,
            "maintains related data in parallel lists instead of one record shape",
        ));
    }

    if lower_body.matches(".strip(").count()
        + lower_body.matches(".lower(").count()
        + lower_body.matches(".replace(").count()
        >= 3
    {
        findings.push(maintainability_finding(
            file,
            function,
            "normalization_logic_duplicated_across_call_sites",
            Severity::Info,
            "contains duplicated normalization logic that likely wants a shared helper",
        ));
    }

    if body.contains("ClassVar") || (body.contains("self.") && body.contains("=[]")) {
        findings.push(maintainability_finding(
            file,
            function,
            "mutable_class_attribute_used_as_instance_storage",
            Severity::Warning,
            "relies on mutable class-level state for per-instance storage",
        ));
    }

    if contains_any(&lower_name, &["util", "helper"])
        && contains_any(
            &lower_body,
            &["path", "json", "cache", "subprocess", "http"],
        )
    {
        findings.push(maintainability_finding(
            file,
            function,
            "helper_module_accumulates_unrelated_cross_domain_utilities",
            Severity::Info,
            "looks like part of a utility sink with unrelated responsibilities",
        ));
    }

    if contains_any(
        body,
        &["\"active\"", "\"disabled\"", "\"pending\"", "\"success\""],
    ) {
        findings.push(maintainability_finding(
            file,
            function,
            "string_sentinel_values_duplicated_instead_of_constant_or_enum",
            Severity::Info,
            "duplicates string sentinel values instead of centralizing them",
        ));
    }

    if body.matches("with ").count() >= 2
        && contains_any(&lower_body, &["open(", "lock", "session"])
    {
        findings.push(maintainability_finding(
            file,
            function,
            "same_contextmanager_pattern_copied_across_modules",
            Severity::Info,
            "repeats context-manager patterns that may want one shared helper",
        ));
    }

    if lower_name.starts_with("wrap")
        || lower_name.starts_with("forward")
        || lower_name.starts_with("delegate")
    {
        findings.push(maintainability_finding(
            file,
            function,
            "wrapper_function_only_renames_arguments_and_passthroughs_behavior",
            Severity::Info,
            "appears to be a thin wrapper that mostly renames arguments and passes through behavior",
        ));
    }

    if body.matches("return {").count() >= 2 && body.matches(':').count() >= 6 {
        findings.push(maintainability_finding(
            file,
            function,
            "anonymous_dict_shape_repeated_without_shared_type_or_builder",
            Severity::Info,
            "returns ad hoc dict payloads that may want a shared shape or builder",
        ));
    }

    if contains_any(&lower_body, &[".endswith(", ".suffix", "mode =="]) {
        findings.push(maintainability_finding(
            file,
            function,
            "branching_on_file_suffix_or_mode_string_scattered_across_codebase",
            Severity::Info,
            "branches on suffix or mode strings inline instead of routing through one policy owner",
        ));
    }

    if contains_any(&lower_body, &["register(", "bootstrap", "initialize"])
        && file.imports.len() > 12
        && contains_any(&lower_body, &["__import__", "importlib", "sys.modules"])
    {
        findings.push(maintainability_finding(
            file,
            function,
            "hidden_dependency_arrives_via_import_time_side_effect",
            Severity::Info,
            "appears to depend on import-time side effects from another module",
        ));
    }

    if contains_any(&lower_body, &["cache", "memo"])
        && !contains_any(&lower_body, &["ttl", "maxsize", "evict", "lru"])
    {
        findings.push(maintainability_finding(
            file,
            function,
            "cache_object_exists_without_size_or_eviction_policy_documentation",
            Severity::Info,
            "uses cache-like state without a visible size or eviction policy signal",
        ));
    }

    if lower_body.matches("if ").count() + lower_body.matches("elif ").count() >= 4
        && function.doc_comment.is_none()
    {
        findings.push(maintainability_finding(
            file,
            function,
            "comment_required_to_explain_opaque_branching_that_code_could_express",
            Severity::Info,
            "has opaque branching that suggests the code structure itself could be clearer",
        ));
    }

    if body.contains("return (") {
        findings.push(maintainability_finding(
            file,
            function,
            "helper_returns_index_based_tuple_instead_of_named_structure",
            Severity::Info,
            "returns an index-based tuple that callers must remember positionally",
        ));
    }

    if contains_any(&lower_name, &["manager", "store"])
        && contains_any(&lower_body, &["save(", "update(", "list(", "get("])
    {
        findings.push(maintainability_finding(
            file,
            function,
            "mixed_mutation_and_query_methods_share_same_manager_class",
            Severity::Info,
            "sits on a manager-like surface that mixes reads and writes without a boundary",
        ));
    }

    if contains_any(
        &file.path.to_string_lossy().to_ascii_lowercase(),
        &["utils", "helpers", "common"],
    ) && file.imports.len() >= 8
    {
        findings.push(maintainability_finding(
            file,
            function,
            "monolithic_utils_module_becomes_default_dependency_sink",
            Severity::Info,
            "lives in a broad utility module that is accumulating cross-cutting dependencies",
        ));
    }

    if file.imports.len() >= 20 && file.functions.len() >= 12 && !lower_body.contains("self.") {
        let policy_keywords = [
            "policy",
            "config",
            "settings",
            "permission",
            "auth",
            "retry",
            "validate",
        ];
        let matching_keywords = policy_keywords
            .iter()
            .filter(|kw| lower_body.contains(**kw))
            .count();
        if matching_keywords >= 3 {
            findings.push(maintainability_finding(
                file,
                function,
                "single_feature_requires_edits_in_many_unrelated_modules_due_to_scattered_policy",
                Severity::Info,
                "appears in a file layout where policy is scattered across many unrelated modules",
            ));
        }
    }

    findings
}
