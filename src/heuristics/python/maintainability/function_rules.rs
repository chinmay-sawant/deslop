use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::helpers::{
    collect_branch_literals, count_prefixed_lines, has_validation_markers, http_boundary_calls,
    is_env_lookup_line, is_input_boundary, is_magic_literal, is_policy_literal,
    looks_like_boundary_context, looks_like_business_context, looks_like_hardcoded_path,
    looks_like_startup_context, should_skip_print_rule,
};

pub(crate) fn exception_swallowed_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .exception_handlers
        .iter()
        .filter(|handler| handler.is_broad && handler.suppresses)
        .map(|handler| Finding {
            rule_id: "exception_swallowed".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: handler.line,
            end_line: handler.line,
            message: format!(
                "function {} suppresses a broad exception instead of surfacing or narrowing it",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("handler_clause={}", handler.clause),
                format!(
                    "handler_action={}",
                    handler.action.as_deref().unwrap_or("<unknown>")
                ),
            ],
        })
        .collect()
}

pub(crate) fn eval_exec_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter(|call| call.receiver.is_none() && matches!(call.name.as_str(), "eval" | "exec"))
        .map(|call| Finding {
            rule_id: "eval_exec_usage".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} evaluates dynamic Python code with {}",
                function.fingerprint.name, call.name
            ),
            evidence: vec![format!("dynamic_builtin={}()", call.name)],
        })
        .collect()
}

pub(crate) fn print_debugging_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || should_skip_print_rule(file, function) {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter(|call| call.receiver.is_none() && call.name == "print")
        .map(|call| Finding {
            rule_id: "print_debugging_leftover".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} leaves print()-based debugging in production code",
                function.fingerprint.name
            ),
            evidence: vec!["output_call=print()".to_string()],
        })
        .collect()
}

pub(crate) fn none_comparison_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .python_evidence()
        .none_comparison_lines
        .iter()
        .map(|line| Finding {
            rule_id: "none_comparison".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} compares against None with == or != instead of identity checks",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=none_equality_comparison".to_string(),
                "suggestion=prefer is None or is not None".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn side_effect_comprehension_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .python_evidence()
        .side_effect_comprehension_lines
        .iter()
        .map(|line| Finding {
            rule_id: "side_effect_comprehension".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses a comprehension only for side effects",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=discarded_comprehension_result".to_string(),
                "suggestion=prefer an explicit loop when the result is discarded".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn redundant_return_none_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .python_evidence()
        .redundant_return_none_lines
        .iter()
        .map(|line| Finding {
            rule_id: "redundant_return_none".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} returns None explicitly where falling through would be clearer",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=explicit_return_none".to_string(),
                "impact=adds noise in simple control flow".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn hardcoded_path_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .local_strings
        .iter()
        .filter(|literal| looks_like_hardcoded_path(&literal.value))
        .map(|literal| Finding {
            rule_id: "hardcoded_path_string".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: literal.line,
            end_line: literal.line,
            message: format!(
                "function {} hardcodes a filesystem path string",
                function.fingerprint.name
            ),
            evidence: vec![format!("path_binding={}={}", literal.name, literal.value)],
        })
        .collect()
}

pub(crate) fn hardcoded_business_rule_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function
        || function.fingerprint.line_count < 6
        || !looks_like_business_context(file, function)
    {
        return Vec::new();
    }

    let policy_literals = collect_branch_literals(&function.body_text)
        .into_iter()
        .filter(|literal| is_policy_literal(literal))
        .collect::<std::collections::BTreeSet<_>>();
    if policy_literals.len() < 2 {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "hardcoded_business_rule".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} hardcodes policy thresholds or status outcomes in business logic",
            function.fingerprint.name
        ),
        evidence: policy_literals
            .into_iter()
            .take(3)
            .map(|literal| format!("business_policy_literal={literal}"))
            .collect(),
    }]
}

pub(crate) fn magic_value_branching_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function
        || function.fingerprint.line_count < 6
        || looks_like_business_context(file, function)
    {
        return Vec::new();
    }

    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for literal in collect_branch_literals(&function.body_text) {
        if is_magic_literal(&literal) {
            *counts.entry(literal).or_default() += 1;
        }
    }

    let repeated = counts
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .collect::<Vec<_>>();
    if repeated.is_empty() {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "magic_value_branching".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} repeats branch-shaping literals instead of naming them explicitly",
            function.fingerprint.name
        ),
        evidence: repeated
            .into_iter()
            .take(3)
            .map(|(literal, count)| format!("branch_literal={literal} occurrences={count}"))
            .collect(),
    }]
}

pub(crate) fn reinvented_utility_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || function.fingerprint.line_count < 5 {
        return Vec::new();
    }

    let name = function.fingerprint.name.to_ascii_lowercase();
    if !name.contains("flatten") {
        return Vec::new();
    }

    let has_itertools = file.imports.iter().any(|import| {
        import.path.starts_with("itertools") || import.path.starts_with("more_itertools")
    });
    let lower_body = function.body_text.to_ascii_lowercase();
    if !has_itertools
        || lower_body.contains("itertools.")
        || lower_body.contains("more_itertools.")
        || count_prefixed_lines(&function.body_text, "for ") < 2
        || !lower_body.contains("append(")
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "reinvented_utility".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} manually flattens iterables even though itertools-style helpers are already imported",
            function.fingerprint.name
        ),
        evidence: vec![
            "pattern=manual_flatten_loop".to_string(),
            "available_import=itertools".to_string(),
        ],
    }]
}

pub(crate) fn variadic_public_api_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let python = function.python_evidence();

    if function.is_test_function
        || function.fingerprint.receiver_type.is_some()
        || function.fingerprint.name.starts_with('_')
        || (!python.has_varargs && !python.has_kwargs)
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "variadic_public_api".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.start_line,
        message: format!(
            "public function {} relies on *args or **kwargs instead of a clearer interface",
            function.fingerprint.name
        ),
        evidence: vec![format!(
            "signature_flags=has_varargs:{} has_kwargs:{}",
            python.has_varargs, python.has_kwargs
        )],
    }]
}

pub(crate) fn builtin_reduction_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .python_evidence()
        .builtin_candidate_lines
        .iter()
        .map(|line| Finding {
            rule_id: "builtin_reduction_candidate".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses a loop shape that may read better with a Python built-in",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=aggregation_loop_candidate".to_string(),
                "suggestion=consider any(), all(), or sum()".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn network_timeout_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !looks_like_boundary_context(file, function) {
        return Vec::new();
    }

    let http_calls = http_boundary_calls(file, function);
    if http_calls.is_empty() {
        return Vec::new();
    }

    let lower_body = function.body_text.to_ascii_lowercase();
    if lower_body.contains("timeout=")
        || lower_body.contains("retry")
        || lower_body.contains("backoff")
        || lower_body.contains("tenacity")
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "network_boundary_without_timeout".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} calls an external HTTP boundary without an obvious timeout or retry policy",
            function.fingerprint.name
        ),
        evidence: vec![
            format!("http_boundary_calls={}", http_calls.join(",")),
            "timeout_or_retry_markers=absent".to_string(),
        ],
    }]
}

pub(crate) fn env_fallback_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !looks_like_startup_context(file, function) {
        return Vec::new();
    }

    let env_lines = function
        .body_text
        .lines()
        .map(str::trim)
        .filter(|line| is_env_lookup_line(line))
        .collect::<Vec<_>>();
    if env_lines.is_empty()
        || env_lines
            .iter()
            .all(|line| super::helpers::env_lookup_has_default(line))
    {
        return Vec::new();
    }

    let lower_body = function.body_text.to_ascii_lowercase();
    if has_validation_markers(function, &lower_body) {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "environment_boundary_without_fallback".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} reads required environment configuration without an obvious fallback or validation path",
            function.fingerprint.name
        ),
        evidence: env_lines
            .into_iter()
            .take(2)
            .map(|line| format!("env_lookup={line}"))
            .collect(),
    }]
}

pub(crate) fn input_validation_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function
        || function.fingerprint.complexity_score > 1
        || !is_input_boundary(file, function)
    {
        return Vec::new();
    }

    let lower_body = function.body_text.to_ascii_lowercase();
    let input_markers = [
        "sys.argv[",
        "json.loads(",
        "request.args",
        "request.form",
        "request.json",
        "get_json(",
        "input(",
        "read_text(",
        "read_bytes(",
    ];
    let matched_markers = input_markers
        .into_iter()
        .filter(|marker| lower_body.contains(marker))
        .collect::<Vec<_>>();
    if matched_markers.is_empty() || has_validation_markers(function, &lower_body) {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "external_input_without_validation".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} consumes external input without an obvious validation or guard step",
            function.fingerprint.name
        ),
        evidence: matched_markers
            .into_iter()
            .take(3)
            .map(|marker| format!("input_marker={marker}"))
            .collect(),
    }]
}

pub(crate) fn broad_exception_handler_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .python_evidence()
        .exception_handlers
        .iter()
        .filter(|handler| handler.is_broad && !handler.suppresses)
        .map(|handler| Finding {
            rule_id: "broad_exception_handler".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: handler.line,
            end_line: handler.line,
            message: format!(
                "function {} catches a broad exception without narrowing the failure type",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("handler_clause={}", handler.clause),
                format!(
                    "handler_action={}",
                    handler.action.as_deref().unwrap_or("<unknown>")
                ),
            ],
        })
        .collect()
}

pub(crate) fn missing_context_manager_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .python_evidence()
        .missing_context_manager_lines
        .iter()
        .map(|line| Finding {
            rule_id: "missing_context_manager".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} opens or acquires a resource without an obvious context manager",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=resource_without_context_manager".to_string(),
                "suggestion=prefer with statements for file handles and similar resources"
                    .to_string(),
            ],
        })
        .collect()
}

pub(crate) fn api_type_hint_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let python = function.python_evidence();

    if function.is_test_function
        || function.fingerprint.name.starts_with('_')
        || python.has_complete_type_hints
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "public_api_missing_type_hints".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.start_line,
        message: format!(
            "public function {} omits complete type hints",
            function.fingerprint.name
        ),
        evidence: vec!["signature_annotations=incomplete".to_string()],
    }]
}
