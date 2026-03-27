use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(super) fn exception_swallowed_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
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
                "function {} swallows a broad exception handler",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("handler clause: {}", handler.clause),
                format!(
                    "handler action: {}",
                    handler.action.as_deref().unwrap_or("<unknown>")
                ),
            ],
        })
        .collect()
}

pub(super) fn eval_exec_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
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
                "function {} uses {} in non-test Python code",
                function.fingerprint.name, call.name
            ),
            evidence: vec![format!("builtin call: {}()", call.name)],
        })
        .collect()
}

pub(super) fn print_debugging_findings(
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
                "function {} leaves print-style debugging in Python code",
                function.fingerprint.name
            ),
            evidence: vec!["builtin call: print()".to_string()],
        })
        .collect()
}

pub(super) fn none_comparison_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
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
            evidence: vec!["prefer is None or is not None for None checks".to_string()],
        })
        .collect()
}

pub(super) fn side_effect_comprehension_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
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
            evidence: vec!["prefer an explicit loop when the result is discarded".to_string()],
        })
        .collect()
}

pub(super) fn redundant_return_none_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
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
                "explicit return None can add noise in simple Python control flow".to_string(),
            ],
        })
        .collect()
}

pub(super) fn hardcoded_path_findings(
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
            evidence: vec![format!("binding {} = {}", literal.name, literal.value)],
        })
        .collect()
}

pub(super) fn hardcoded_business_rule_findings(
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
            .map(|literal| format!("policy_literal={literal}"))
            .collect(),
    }]
}

pub(super) fn magic_value_branching_findings(
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
            .map(|(literal, count)| format!("literal={literal} occurrences={count}"))
            .collect(),
    }]
}

pub(super) fn reinvented_utility_findings(
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
        evidence: vec!["available_import=itertools".to_string()],
    }]
}

pub(super) fn variadic_public_api_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function
        || function.fingerprint.receiver_type.is_some()
        || function.fingerprint.name.starts_with('_')
        || (!function.has_varargs && !function.has_kwargs)
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
            "has_varargs={} has_kwargs={}",
            function.has_varargs, function.has_kwargs
        )],
    }]
}

pub(super) fn builtin_reduction_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
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
                "consider any(), all(), or sum() when the loop is only aggregating a result"
                    .to_string(),
            ],
        })
        .collect()
}

pub(super) fn network_boundary_without_timeout_findings(
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
        evidence: vec![format!("http_calls={}", http_calls.join(","))],
    }]
}

pub(super) fn environment_boundary_without_fallback_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !looks_like_startup_context(file, function) {
        return Vec::new();
    }

    let env_lines = function
        .body_text
        .lines()
        .map(str::trim)
        .filter(|line| is_env_lookup_line(line))
        .collect::<Vec<_>>();
    if env_lines.is_empty() || env_lines.iter().all(|line| env_lookup_has_default(line)) {
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

pub(super) fn external_input_without_validation_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function
        || function.fingerprint.complexity_score > 1
        || !looks_like_input_boundary_context(file, function)
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
            "function {} consumes external input without an obvious validation or guard path",
            function.fingerprint.name
        ),
        evidence: matched_markers
            .into_iter()
            .take(3)
            .map(|marker| format!("input_marker={marker}"))
            .collect(),
    }]
}

pub(super) fn broad_exception_handler_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
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
            evidence: vec![format!("handler clause: {}", handler.clause)],
        })
        .collect()
}

pub(super) fn missing_context_manager_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
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
                "prefer with statements for file handles, locks, and similar resources".to_string(),
            ],
        })
        .collect()
}

pub(super) fn public_api_missing_type_hints_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function
        || function.fingerprint.name.starts_with('_')
        || function.has_complete_type_hints
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
        evidence: vec!["the signature is missing parameter or return annotations".to_string()],
    }]
}

pub(super) fn commented_out_code_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let suspicious_comments = file
        .comments
        .iter()
        .filter(|comment| looks_like_commented_out_code(&comment.text))
        .collect::<Vec<_>>();
    if suspicious_comments.is_empty() {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "commented_out_code".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: suspicious_comments[0].line,
        end_line: suspicious_comments[0].line,
        message: "file contains comments that look like disabled code".to_string(),
        evidence: suspicious_comments
            .iter()
            .take(3)
            .map(|comment| format!("line {}: {}", comment.line, comment.text))
            .collect(),
    }]
}

pub(super) fn mixed_sync_async_module_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file
        || !file
            .imports
            .iter()
            .any(|import| import.path.starts_with("asyncio"))
    {
        return Vec::new();
    }

    let async_public = file
        .functions
        .iter()
        .filter(|function| {
            function.fingerprint.kind.starts_with("async")
                && !function.is_test_function
                && !function.fingerprint.name.starts_with('_')
        })
        .count();
    let sync_public = file
        .functions
        .iter()
        .filter(|function| {
            !function.fingerprint.kind.starts_with("async")
                && !function.is_test_function
                && !function.fingerprint.name.starts_with('_')
        })
        .count();
    if async_public == 0 || sync_public == 0 {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "mixed_sync_async_module".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: file.functions[0].fingerprint.start_line,
        end_line: file.functions[0].fingerprint.start_line,
        message: "module mixes public sync and async entry points".to_string(),
        evidence: vec![
            format!("async_public_functions={async_public}"),
            format!("sync_public_functions={sync_public}"),
        ],
    }]
}

fn should_skip_print_rule(file: &ParsedFile, function: &ParsedFunction) -> bool {
    function.fingerprint.name == "main"
        || file
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "__main__.py")
}

fn looks_like_hardcoded_path(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value
            .chars()
            .nth(1)
            .is_some_and(|character| character == ':')
            && value.contains('\\')
        || (value.contains('/') && has_path_like_suffix(value))
}

fn has_path_like_suffix(value: &str) -> bool {
    [
        ".json", ".yaml", ".yml", ".txt", ".csv", ".db", ".sqlite", ".ini", ".cfg", ".conf",
        ".pem", ".log",
    ]
    .iter()
    .any(|suffix| value.ends_with(suffix))
}

fn looks_like_commented_out_code(text: &str) -> bool {
    let normalized = text.trim();
    normalized.starts_with("if ")
        || normalized.starts_with("for ")
        || normalized.starts_with("while ")
        || normalized.starts_with("return ")
        || normalized.starts_with("def ")
        || normalized.starts_with("class ")
        || normalized.starts_with("try:")
        || normalized.starts_with("except ")
        || (normalized.contains('=')
            && normalized.contains('(')
            && normalized
                .chars()
                .any(|character| character.is_ascii_alphabetic()))
}

fn looks_like_business_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let markers = [
        "eligib",
        "discount",
        "pricing",
        "price",
        "risk",
        "approve",
        "approval",
        "tier",
        "quota",
        "commission",
        "policy",
        "status",
        "fraud",
        "score",
    ];
    function_or_path_matches(file, function, &markers)
}

fn looks_like_boundary_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let markers = [
        "handler",
        "endpoint",
        "route",
        "view",
        "controller",
        "cli",
        "command",
        "main",
        "sync",
        "fetch",
        "publish",
        "process",
        "ingest",
        "import",
        "export",
        "job",
        "startup",
        "bootstrap",
        "config",
    ];
    function_or_path_matches(file, function, &markers)
}

fn looks_like_startup_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let markers = ["config", "settings", "startup", "bootstrap", "main", "env"];
    function_or_path_matches(file, function, &markers)
}

fn looks_like_input_boundary_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let markers = [
        "cli", "command", "handler", "request", "ingest", "import", "parse",
    ];
    function_or_path_matches(file, function, &markers)
}

fn function_or_path_matches(
    file: &ParsedFile,
    function: &ParsedFunction,
    markers: &[&str],
) -> bool {
    let function_name = function.fingerprint.name.to_ascii_lowercase();
    if markers.iter().any(|marker| function_name.contains(marker)) {
        return true;
    }

    if file.package_name.as_deref().is_some_and(|name| {
        markers
            .iter()
            .any(|marker| name.to_ascii_lowercase().contains(marker))
    }) {
        return true;
    }

    file.path.components().any(|component| {
        let part = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        markers.iter().any(|marker| part.contains(marker))
    })
}

fn collect_branch_literals(body_text: &str) -> Vec<String> {
    body_text
        .lines()
        .map(str::trim)
        .filter(|line| is_branch_line(line))
        .flat_map(|line| {
            let mut literals = extract_string_literals(line);
            literals.extend(extract_numeric_literals(line));
            literals
        })
        .collect()
}

fn is_branch_line(line: &str) -> bool {
    line.starts_with("if ")
        || line.starts_with("elif ")
        || line.starts_with("case ")
        || line.starts_with("match ")
}

fn extract_string_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let characters = line.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < characters.len() {
        let quote = characters[index];
        if quote != '\'' && quote != '"' {
            index += 1;
            continue;
        }

        index += 1;
        let start = index;
        while index < characters.len() && characters[index] != quote {
            if characters[index] == '\\' {
                index += 1;
            }
            index += 1;
        }

        if index > start {
            let literal = characters[start..index].iter().collect::<String>();
            if !literal.trim().is_empty() {
                literals.push(literal);
            }
        }
        index += 1;
    }

    literals
}

fn extract_numeric_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut current = String::new();

    for character in line.chars() {
        if character.is_ascii_digit() || character == '.' {
            current.push(character);
        } else {
            flush_numeric_literal(&mut current, &mut literals);
        }
    }
    flush_numeric_literal(&mut current, &mut literals);

    literals
}

fn flush_numeric_literal(current: &mut String, literals: &mut Vec<String>) {
    let token = current.trim_matches('.');
    if !token.is_empty()
        && token
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
        && token.chars().any(|character| character.is_ascii_digit())
    {
        literals.push(token.to_string());
    }
    current.clear();
}

fn is_policy_literal(literal: &str) -> bool {
    let lower = literal.to_ascii_lowercase();
    lower
        .parse::<f64>()
        .is_ok_and(|value| value >= 0.0 && (value.fract() != 0.0 || value >= 20.0))
        || matches!(
            lower.as_str(),
            "approved"
                | "rejected"
                | "manual_review"
                | "priority"
                | "standard"
                | "premium"
                | "enterprise"
                | "eligible"
                | "blocked"
                | "pending"
        )
}

fn is_magic_literal(literal: &str) -> bool {
    let lower = literal.to_ascii_lowercase();
    if let Ok(value) = lower.parse::<f64>() {
        return value.fract() != 0.0 || value >= 20.0;
    }

    lower.len() >= 5 && !matches!(lower.as_str(), "false" | "true" | "none")
}

fn count_prefixed_lines(body_text: &str, prefix: &str) -> usize {
    body_text
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with(prefix))
        .count()
}

fn http_boundary_calls(file: &ParsedFile, function: &ParsedFunction) -> Vec<String> {
    let alias_lookup = file
        .imports
        .iter()
        .map(|import| (import.alias.as_str(), import.path.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();

    function
        .calls
        .iter()
        .filter_map(|call| {
            let receiver = call.receiver.as_deref().unwrap_or(call.name.as_str());
            let import_path = alias_lookup.get(receiver).copied().unwrap_or(receiver);
            (import_path.starts_with("requests")
                || import_path.starts_with("httpx")
                || import_path.starts_with("urllib")
                || matches!(
                    call.name.as_str(),
                    "get" | "post" | "put" | "patch" | "delete" | "request"
                ))
            .then(|| call.name.clone())
        })
        .collect()
}

fn is_env_lookup_line(line: &str) -> bool {
    line.contains("os.getenv(") || line.contains("os.environ.get(") || line.contains("os.environ[")
}

fn env_lookup_has_default(line: &str) -> bool {
    (line.contains("os.getenv(") || line.contains("os.environ.get("))
        && line
            .split_once('(')
            .and_then(|(_, tail)| tail.split_once(')'))
            .is_some_and(|(args, _)| args.contains(','))
        || line.contains(" or ")
}

fn has_validation_markers(function: &ParsedFunction, lower_body: &str) -> bool {
    !function.exception_handlers.is_empty()
        || lower_body.contains("if not ")
        || lower_body.contains("if len(")
        || lower_body.contains(" is none")
        || lower_body.contains("validate")
        || lower_body.contains("assert ")
        || lower_body.contains("raise ")
        || lower_body.contains("schema")
        || lower_body.contains("pydantic")
}
