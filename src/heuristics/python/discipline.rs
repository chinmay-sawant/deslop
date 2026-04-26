/// discipline.rs — Section 3 (error handling, 20 rules) +
///                 Section 4 (type system, 15 rules) +
///                 Section 5 (testing anti-patterns, 20 rules)
use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_line(body: &str, needle: &str, base: usize) -> Option<usize> {
    body.lines()
        .enumerate()
        .find_map(|(i, l)| l.contains(needle).then_some(base + i))
}

fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;

    for (index, ch) in text.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                parts.push(text[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    let tail = text[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }

    parts
}

fn extract_parameter_section(signature: &str) -> Option<&str> {
    let def_start = signature.find("def ").or(signature.find("async def "))?;
    let after_def = &signature[def_start..];
    let open_offset = after_def.find('(')?;
    let open_index = def_start + open_offset;
    let mut depth = 0usize;

    for (offset, ch) in signature[open_index + 1..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(&signature[open_index + 1..open_index + 1 + offset]),
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    None
}

fn optional_parameter_names(signature: &str) -> Vec<String> {
    let Some(parameters) = extract_parameter_section(signature) else {
        return Vec::new();
    };

    split_top_level_commas(parameters)
        .into_iter()
        .filter_map(|entry| {
            let trimmed = entry.trim().trim_start_matches('*').trim();
            if trimmed.is_empty() || matches!(trimmed, "/" | "*" | "self" | "cls") {
                return None;
            }

            let subject = trimmed.split('=').next().unwrap_or(trimmed).trim();
            let annotation = subject.split(':').nth(1)?.trim().to_ascii_lowercase();
            let is_optional = annotation.contains("optional[")
                || annotation.contains(" | none")
                || annotation.contains("none |");
            if !is_optional {
                return None;
            }

            Some(
                subject
                    .split(':')
                    .next()
                    .unwrap_or(subject)
                    .trim()
                    .to_string(),
            )
        })
        .collect()
}

fn make_finding(
    rule_id: &str,
    severity: Severity,
    file: &ParsedFile,
    function: &ParsedFunction,
    line: usize,
    msg: &str,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!("function {} {msg}", function.fingerprint.name),
        evidence: vec![format!("rule={rule_id}")],
    }
}

// ── Section 3 · Error Handling Discipline ────────────────────────────────────

pub(super) fn exception_raised_without_chaining_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Look for `raise XxxError(` inside an `except` block without `from`
    let mut in_except = false;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("except ") || trimmed == "except:" {
            in_except = true;
        }
        if in_except && trimmed.starts_with("raise ") && !trimmed.ends_with("raise") {
            let chained = body
                .lines()
                .skip(i)
                .take(8)
                .any(|candidate| candidate.contains(" from "));
            if chained {
                continue;
            }
            return vec![make_finding(
                "exception_raised_without_chaining_original_cause",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "raises a new exception inside except without `from e`; original cause is lost",
            )];
        }
    }
    Vec::new()
}

pub(super) fn exception_branches_on_message_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &["str(e)", "e.args[0]", "e.message", "str(exc)"];
    for p in PATTERNS {
        if body.contains(p) && (body.contains("if ") || body.contains("elif ")) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "exception_handler_branches_on_error_message_string",
                Severity::Warning,
                file,
                function,
                line,
                "branches on exception message string; match on exception type or typed attributes instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn bare_except_catches_system_exit_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "except:" {
            return vec![make_finding(
                "bare_except_clause_catches_system_exit",
                Severity::Warning,
                file,
                function,
                function.fingerprint.start_line + i,
                "uses bare `except:` which catches SystemExit and KeyboardInterrupt",
            )];
        }
    }
    Vec::new()
}

pub(super) fn exception_logged_and_reraised_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut in_except = false;
    let mut has_log = false;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("except") {
            in_except = true;
            has_log = false;
        }
        if in_except {
            if trimmed.contains("logger.") || trimmed.contains("logging.") {
                has_log = true;
            }
            if has_log && trimmed.starts_with("raise") {
                return vec![make_finding(
                    "exception_logged_and_then_re_raised_redundantly",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "logs the exception and then re-raises; produces duplicate log entries",
                )];
            }
        }
    }
    Vec::new()
}

pub(super) fn validation_error_mapped_to_500_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("except ValueError") || body.contains("except ValidationError"))
        && (body.contains("500")
            || body.contains("status_code=500")
            || body.contains("return Response(content="))
    {
        let line = find_line(body, "except ValueError", function.fingerprint.start_line)
            .or_else(|| {
                find_line(
                    body,
                    "except ValidationError",
                    function.fingerprint.start_line,
                )
            })
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "validation_or_parse_error_mapped_to_500_status",
            Severity::Warning,
            file,
            function,
            line,
            "maps input validation error to HTTP 500; use 400-class status for client errors",
        )];
    }
    Vec::new()
}

pub(super) fn exception_silenced_in_finally_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut in_finally = false;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "finally:" || trimmed.ends_with("finally:") {
            in_finally = true;
        }
        if in_finally && trimmed.starts_with("except") {
            // Look ahead for pass
            if body
                .lines()
                .nth(i + 1)
                .map(|l| l.trim() == "pass")
                .unwrap_or(false)
            {
                return vec![make_finding(
                    "exception_silenced_in_cleanup_or_finally_block",
                    Severity::Warning,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "silences exception inside finally/cleanup block; failures are hidden from caller",
                )];
            }
        }
    }
    Vec::new()
}

pub(super) fn exception_not_inheriting_base_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        // class FooError(Exception): not inheriting project base
        if trimmed.starts_with("class ")
            && trimmed.contains("Error(Exception)")
            && !trimmed.contains("Base")
            && !trimmed.contains("Project")
            && !trimmed.contains("App")
        {
            return vec![make_finding(
                "project_exception_class_not_inheriting_shared_base",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "defines exception inheriting directly from Exception; inherit from a project base instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn exception_for_control_flow_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // raise + except in close proximity within same function body with same exception type
    let has_raise = body.contains("raise StopIteration") || body.contains("raise LoopExit");
    let has_catch = body.contains("except StopIteration") || body.contains("except LoopExit");
    if has_raise && has_catch {
        let line = find_line(body, "raise StopIteration", function.fingerprint.start_line)
            .or_else(|| find_line(body, "raise LoopExit", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "exception_raised_and_caught_for_control_flow_within_same_function",
            Severity::Info,
            file,
            function,
            line,
            "raises and catches exception for control flow within the same function; use early returns instead",
        )];
    }
    Vec::new()
}

pub(super) fn error_message_embeds_sensitive_data_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "raise ValueError(f\"",
        "raise RuntimeError(f\"",
        "raise Exception(f\"",
    ];
    const SENSITIVE: &[&str] = &["password", "secret", "token", "key", "credential", "sql"];
    for p in PATTERNS {
        if let Some(pos) = body.find(p) {
            let snippet = &body[pos..pos.min(pos + 120)];
            if SENSITIVE.iter().any(|s| snippet.to_lowercase().contains(s)) {
                let line = find_line(body, p, function.fingerprint.start_line)
                    .unwrap_or(function.fingerprint.start_line);
                return vec![make_finding(
                    "error_message_embeds_sensitive_data",
                    Severity::Warning,
                    file,
                    function,
                    line,
                    "interpolates sensitive data into an error message",
                )];
            }
        }
    }
    Vec::new()
}

pub(super) fn retry_catches_broad_exception_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_retry = body.contains("for attempt")
        || body.contains("while retry")
        || body.contains("retry_count");
    let has_broad = body.contains("except Exception:") || body.contains("except BaseException:");
    if has_retry && has_broad {
        let line = find_line(body, "except Exception:", function.fingerprint.start_line)
            .or_else(|| {
                find_line(
                    body,
                    "except BaseException:",
                    function.fingerprint.start_line,
                )
            })
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "retry_loop_catches_broad_base_exception",
            Severity::Warning,
            file,
            function,
            line,
            "retry loop catches broad Exception; declare a specific set of retryable exception types",
        )];
    }
    Vec::new()
}

pub(super) fn transaction_missing_rollback_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_begin = body.contains("session.begin()")
        || body.contains("db.begin()")
        || body.contains("conn.begin()");
    let has_rollback = body.contains(".rollback()");
    let has_except = body.contains("except ");
    if has_begin && has_except && !has_rollback {
        let line = find_line(body, "begin()", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "transaction_block_missing_rollback_on_exception",
            Severity::Warning,
            file,
            function,
            line,
            "starts a DB transaction with except block but no .rollback() on error path",
        )];
    }
    Vec::new()
}

pub(super) fn assert_for_validation_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "assert isinstance(",
        "assert len(",
        "assert user",
        "assert request",
        "assert value is not None",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "assert_used_for_runtime_input_validation_in_production",
                Severity::Warning,
                file,
                function,
                line,
                "uses assert for input validation; assert is stripped by -O flag",
            )];
        }
    }
    Vec::new()
}

pub(super) fn warning_instead_of_exception_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &[
        "warnings.warn(",
        "warnings.warn(\"Invalid",
        "warnings.warn(\"Missing",
        "warnings.warn(\"Unexpected",
    ];
    for p in PATTERNS {
        if body.contains(p) {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "warning_issued_instead_of_exception_for_invalid_state",
                Severity::Info,
                file,
                function,
                line,
                "issues a warning for an invalid state; raise an exception instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn exception_handler_no_logging_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    if function.fingerprint.name.starts_with("_safe_") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut in_except = false;
    let mut except_start = 0;
    let mut except_indent = 0;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("except ") || trimmed == "except:" {
            in_except = true;
            except_start = i;
            except_indent = indent_level(line);
        }
        if in_except && i > except_start {
            if trimmed.starts_with("return ") {
                let return_value = trimmed.trim_start_matches("return").trim();
                if matches!(return_value, "False" | "True")
                    || function.signature_text.contains("-> bool")
                {
                    continue;
                }
                // Check no log in the block
                let block: Vec<&str> = body
                    .lines()
                    .skip(except_start)
                    .take(i - except_start + 1)
                    .collect();
                let has_log = block
                    .iter()
                    .any(|l| l.contains("log") || l.contains("logger") || l.contains("print"));
                if !has_log {
                    return vec![make_finding(
                        "exception_handler_returns_default_without_any_logging",
                        Severity::Info,
                        file,
                        function,
                        function.fingerprint.start_line + i,
                        "returns default from except block with no logging; suppressed failures are invisible",
                    )];
                }
            }
            if !trimmed.starts_with('#')
                && !trimmed.is_empty()
                && indent_level(line) <= except_indent
            {
                in_except = false;
            }
        }
    }
    Vec::new()
}

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

pub(super) fn deeply_nested_try_except_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let try_count = body.matches("try:").count();
    if try_count >= 3 {
        let line = find_line(body, "try:", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "deeply_nested_try_except_beyond_two_levels",
            Severity::Info,
            file,
            function,
            line,
            "has 3+ nested try/except blocks; decompose into separate functions",
        )];
    }
    Vec::new()
}

pub(super) fn suppress_with_base_exception_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("contextlib.suppress(Exception)")
        || body.contains("contextlib.suppress(BaseException)")
    {
        let line = find_line(
            body,
            "contextlib.suppress(",
            function.fingerprint.start_line,
        )
        .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "contextlib_suppress_applied_with_exception_base_class",
            Severity::Warning,
            file,
            function,
            line,
            "suppresses all exceptions with contextlib.suppress(Exception); use a specific subset",
        )];
    }
    Vec::new()
}

pub(super) fn oserror_without_errno_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if (body.contains("except OSError") || body.contains("except IOError"))
        && !body.contains(".errno")
        && !body.contains("errno.")
    {
        let line = find_line(body, "except OSError", function.fingerprint.start_line)
            .or_else(|| find_line(body, "except IOError", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "oserror_caught_without_errno_inspection",
            Severity::Info,
            file,
            function,
            line,
            "catches OSError without inspecting .errno; specific OS failure kind determines recovery",
        )];
    }
    Vec::new()
}

pub(super) fn custom_exception_string_code_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const PATTERNS: &[&str] = &["self.code = ", "self.error_code = ", "self.type = "];
    for p in PATTERNS {
        if body.contains(p) && body.contains("def __init__") {
            let line = find_line(body, p, function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "custom_exception_encodes_identity_as_string_code_attribute",
                Severity::Info,
                file,
                function,
                line,
                "exception class uses a string code attribute for identity; use the class hierarchy instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn generator_close_without_finally_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Detect generator functions with teardown but no finally/GeneratorExit handling
    let has_yield = body.contains("yield ");
    let has_teardown =
        body.contains(".close()") || body.contains(".cleanup()") || body.contains(".disconnect()");
    let has_finally = body.contains("finally:");
    let has_generator_exit = body.contains("GeneratorExit");
    if has_yield && has_teardown && !has_finally && !has_generator_exit {
        let line = find_line(body, "yield ", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "generator_close_exception_not_handled_when_cleanup_required",
            Severity::Info,
            file,
            function,
            line,
            "generator with cleanup code lacks finally/GeneratorExit guard; teardown may be skipped on close",
        )];
    }
    Vec::new()
}

// ── Section 4 · Type System and API Contracts ─────────────────────────────────

pub(super) fn overloaded_without_decorator_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Multiple isinstance checks on same arg but no @overload
    let isinstance_count = body.matches("isinstance(").count();
    if isinstance_count >= 3
        && !function.signature_text.contains("@overload")
        && !body.contains("@typing.overload")
    {
        let line = find_line(body, "isinstance(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "overloaded_dispatch_without_typing_overload_decorator",
            Severity::Info,
            file,
            function,
            line,
            "dispatches on isinstance for multiple types without @typing.overload signatures",
        )];
    }
    Vec::new()
}

pub(super) fn protocol_isinstance_without_runtime_checkable_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Look for isinstance(obj, SomeProtocol) where Protocol is imported but @runtime_checkable is absent
    if body.contains("isinstance(") {
        let imports: Vec<&str> = file.imports.iter().map(|i| i.path.as_str()).collect();
        let has_protocol =
            imports.iter().any(|i| i.contains("Protocol")) || body.contains("Protocol");
        let has_runtime_checkable = body.contains("runtime_checkable")
            || file
                .imports
                .iter()
                .any(|i| i.path.contains("runtime_checkable"))
            || file
                .functions
                .iter()
                .any(|f| f.body_text.contains("@runtime_checkable"));
        if has_protocol && !has_runtime_checkable {
            let line = find_line(body, "isinstance(", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "protocol_used_in_isinstance_without_runtime_checkable",
                Severity::Warning,
                file,
                function,
                line,
                "uses Protocol in isinstance() without @runtime_checkable; raises TypeError at runtime",
            )];
        }
    }
    Vec::new()
}

pub(super) fn optional_without_none_guard_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    let body = &function.body_text;
    let optional_parameters = optional_parameter_names(sig);
    if optional_parameters.is_empty() {
        return Vec::new();
    }

    let has_none_guard = body.contains(" is None")
        || body.contains(" is not None")
        || body.contains("if not ")
        || body.contains(" or ");
    if has_none_guard
        || optional_parameters.iter().any(|name| {
            body.contains(&format!("if {name}"))
                || body.contains(&format!("{name} and"))
                || body.contains(&format!("{name} else"))
        })
    {
        return Vec::new();
    }

    vec![make_finding(
        "optional_parameter_used_without_none_guard",
        Severity::Info,
        file,
        function,
        function.fingerprint.start_line,
        "has Optional parameter but dereferences it without a None guard",
    )]
}

pub(super) fn callable_without_param_types_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    // Detect bare Callable annotations
    if (sig.contains(": Callable)")
        || sig.contains(": Callable,")
        || sig.contains("Callable[..., Any]"))
        && !function.fingerprint.name.starts_with('_')
    {
        return vec![make_finding(
            "callable_annotation_without_parameter_types",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "uses bare Callable annotation without parameter types on a public API boundary",
        )];
    }
    Vec::new()
}

pub(super) fn cast_without_narrowing_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("typing.cast(") || body.contains("cast(") {
        // Check for guard before cast
        let has_guard =
            body.contains("isinstance(") || body.contains("assert ") || body.contains("if ");
        if !has_guard {
            let line = find_line(body, "cast(", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "cast_applied_without_preceding_type_narrowing_guard",
                Severity::Info,
                file,
                function,
                line,
                "applies typing.cast without a preceding isinstance/assert guard",
            )];
        }
    }
    Vec::new()
}

pub(super) fn type_alias_shadows_builtin_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    const BUILTINS: &[&str] = &[
        "list", "dict", "set", "type", "id", "filter", "map", "input", "bytes",
    ];
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        let Some((lhs, _rhs)) = trimmed.split_once(" = ") else {
            continue;
        };
        let candidate = lhs.trim().split(':').next().unwrap_or(lhs.trim()).trim();
        if BUILTINS.contains(&candidate) {
            return vec![make_finding(
                "type_alias_shadows_builtin_name",
                Severity::Warning,
                file,
                function,
                function.fingerprint.start_line + i,
                &format!("assigns to name `{candidate}` which shadows a Python builtin"),
            )];
        }
    }
    Vec::new()
}

pub(super) fn namedtuple_where_dataclass_fits_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("namedtuple(") || body.contains("collections.namedtuple(") {
        // Signal if there's mutable intent (lists/dicts as defaults, or post-init-like comments)
        if body.contains("default=") || body.contains("# mutable") || body.contains("= []") {
            let line = find_line(body, "namedtuple(", function.fingerprint.start_line)
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "namedtuple_used_where_dataclass_better_fits",
                Severity::Info,
                file,
                function,
                line,
                "uses namedtuple for mutable data; consider @dataclass instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn public_return_union_many_types_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || function.fingerprint.name.starts_with('_') {
        return Vec::new();
    }
    let sig = &function.signature_text;
    // Union with 4+ pipe-separated or explicit Union types
    let union_count = sig.matches(" | ").count();
    if union_count >= 3 {
        return vec![make_finding(
            "public_function_return_type_annotated_as_union_of_many_unrelated_types",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "returns a Union of many unrelated types; use a more specific sum type or protocol",
        )];
    }
    Vec::new()
}

pub(super) fn typevar_without_bound_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("TypeVar(") && !body.contains("bound=") && !body.contains("constraints=") {
        let line = find_line(body, "TypeVar(", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "typevar_defined_without_bound_or_constraints_for_narrow_use",
            Severity::Info,
            file,
            function,
            line,
            "defines TypeVar without bound or constraints for a narrowly-typed use site",
        )];
    }
    Vec::new()
}

pub(super) fn generic_class_without_type_param_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    // Detect bare Generic usage in annotations
    if sig.contains(": list)") || sig.contains(": dict)") || sig.contains(": set)") {
        return vec![make_finding(
            "generic_class_used_without_type_parameter_application",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "uses bare generic container annotation without type parameters",
        )];
    }
    Vec::new()
}

pub(super) fn protocol_method_lacks_annotations_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Class body with Protocol but methods have no return annotation
    if body.contains("Protocol") && body.contains("def ") && !body.contains("->") {
        let line = find_line(body, "Protocol", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "protocol_method_lacks_type_annotations",
            Severity::Info,
            file,
            function,
            line,
            "Protocol method lacks return type annotation; structural contract is incomplete",
        )];
    }
    Vec::new()
}

pub(super) fn typed_dict_access_without_guard_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Detect TypedDict key access without get or guard
    if (body.contains("TypedDict") || file.imports.iter().any(|i| i.path.contains("TypedDict")))
        && body.contains("[\"")
        && !body.contains(".get(")
        && !body.contains(" in ")
        && !body.contains("if \"")
    {
        let line = find_line(body, "[\"", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "typed_dict_key_access_without_get_or_guard",
            Severity::Info,
            file,
            function,
            line,
            "accesses optional TypedDict key by index without .get() or membership guard",
        )];
    }
    Vec::new()
}

pub(super) fn typed_dict_total_false_no_doc_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("TypedDict")
        && body.contains("total=False")
        && !body.contains("\"\"\"")
        && !body.contains("# optional")
    {
        let line = find_line(body, "total=False", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "typed_dict_total_false_without_docstring_noting_optional_keys",
            Severity::Info,
            file,
            function,
            line,
            "TypedDict with total=False lacks a docstring identifying which keys are optional",
        )];
    }
    Vec::new()
}

pub(super) fn forward_ref_not_under_type_checking_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    // Forward references as strings not under TYPE_CHECKING
    if sig.contains(": \"")
        && !file
            .imports
            .iter()
            .any(|i| i.path.contains("TYPE_CHECKING"))
    {
        return vec![make_finding(
            "string_forward_reference_in_annotation_not_under_type_checking_guard",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "uses string forward reference without TYPE_CHECKING guard",
        )];
    }
    Vec::new()
}

// ── Section 5 · Testing Anti-patterns ────────────────────────────────────────

pub(super) fn test_too_many_patches_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let patch_count = function.signature_text.matches("mock.patch").count()
        + function.signature_text.matches("mocker.patch").count()
        + function.signature_text.matches("@patch(").count();
    if patch_count >= 5 {
        return vec![make_finding(
            "test_function_stacks_too_many_mock_patch_decorators",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "stacks 5+ mock.patch decorators; the production code likely needs better dependency injection",
        )];
    }
    Vec::new()
}

pub(super) fn test_sleeps_for_coordination_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("time.sleep(") || body.contains("asyncio.sleep(") {
        // Skip sleep(0)
        if !body.contains("time.sleep(0)") && !body.contains("asyncio.sleep(0)") {
            let line = find_line(body, "time.sleep(", function.fingerprint.start_line)
                .or_else(|| find_line(body, "asyncio.sleep(", function.fingerprint.start_line))
                .unwrap_or(function.fingerprint.start_line);
            return vec![make_finding(
                "test_calls_time_sleep_for_coordination",
                Severity::Warning,
                file,
                function,
                line,
                "uses time.sleep for test coordination; use events, mocks, or condition variables instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn test_mutates_global_without_restore_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Look for direct module-global assignment not through monkeypatch
    let has_global_mutation =
        body.contains("module.") && body.contains(" = ") && !body.contains("monkeypatch");
    let has_teardown =
        body.contains("yield") || body.contains("addCleanup") || body.contains("finally:");
    if has_global_mutation && !has_teardown {
        let line = find_line(body, " = ", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "test_mutates_module_global_without_restore",
            Severity::Warning,
            file,
            function,
            line,
            "mutates a module global without teardown; risks test-ordering pollution",
        )];
    }
    Vec::new()
}

pub(super) fn test_asserts_private_attribute_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // assert obj._private == something
    if body.contains("assert ") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("assert ") && trimmed.contains("._") && trimmed.contains(" == ")
            {
                return vec![make_finding(
                    "test_asserts_private_attribute_value_instead_of_behavior",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "asserts on a private attribute; test observable public behavior instead",
                )];
            }
        }
    }
    Vec::new()
}

pub(super) fn test_datetime_without_freeze_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_time_call = body.contains("datetime.now()")
        || body.contains("datetime.utcnow()")
        || body.contains("date.today()")
        || body.contains("time.time()");
    let has_freeze = body.contains("freeze_time")
        || body.contains("time_machine")
        || body.contains("monkeypatch");
    if has_time_call && !has_freeze {
        let line = find_line(body, "datetime.now()", function.fingerprint.start_line)
            .or_else(|| find_line(body, "date.today()", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "test_fixture_calls_datetime_now_without_freezing",
            Severity::Warning,
            file,
            function,
            line,
            "calls datetime.now() without freezing time; test results are non-deterministic",
        )];
    }
    Vec::new()
}

pub(super) fn test_wraps_sut_in_try_except_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("try:") && body.contains("except ") && !body.contains("pytest.raises") {
        let line = find_line(body, "try:", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "test_wraps_sut_in_try_except_hiding_exception_detail",
            Severity::Warning,
            file,
            function,
            line,
            "wraps SUT in try/except without asserting exception type; hides unexpected exceptions",
        )];
    }
    Vec::new()
}

pub(super) fn test_parametrize_single_case_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    if sig.contains("@pytest.mark.parametrize") {
        // Single tuple: no comma-separated closing like [(a, b), (c, d)]
        let param_count = sig.matches("(\"").count() + sig.matches(", (").count();
        if param_count <= 1 {
            return vec![make_finding(
                "pytest_parametrize_with_single_test_case",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line,
                "uses @pytest.mark.parametrize with a single case; use a plain test instead",
            )];
        }
    }
    Vec::new()
}

pub(super) fn test_no_reason_skip_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let sig = &function.signature_text;
    if (sig.contains("@pytest.mark.skip") || sig.contains("@pytest.mark.xfail"))
        && !sig.contains("reason=")
    {
        return vec![make_finding(
            "test_skipped_with_no_reason_string",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "skips or xfails test without a reason= string",
        )];
    }
    Vec::new()
}

pub(super) fn test_float_equality_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("assert ") && body.contains(" == ") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("assert ")
                && trimmed.contains(" == ")
                && (trimmed.contains(".0")
                    || trimmed.contains("float")
                    || trimmed.contains("result"))
                && !trimmed.contains("approx")
                && !trimmed.contains("isclose")
            {
                return vec![make_finding(
                    "test_compares_float_with_equality_operator",
                    Severity::Warning,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "compares float with == operator; use pytest.approx() or math.isclose()",
                )];
            }
        }
    }
    Vec::new()
}

pub(super) fn test_loads_real_config_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("load_dotenv()") || body.contains(".env") && body.contains("open(") {
        let line = find_line(body, "load_dotenv", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "test_loads_real_application_config_or_secrets",
            Severity::Warning,
            file,
            function,
            line,
            "loads real application config or .env file in tests; use isolated test configuration",
        )];
    }
    Vec::new()
}

pub(super) fn test_makes_real_http_call_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let has_real_call = body.contains("requests.get(")
        || body.contains("requests.post(")
        || body.contains("httpx.get(")
        || body.contains("httpx.post(");
    let has_mock = body.contains("responses.")
        || body.contains("respx.")
        || body.contains("vcr")
        || body.contains("mock");
    if has_real_call && !has_mock {
        let line = find_line(body, "requests.get(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "httpx.get(", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "test_makes_real_outbound_http_call_without_mock_or_vcr",
            Severity::Warning,
            file,
            function,
            line,
            "makes a real outbound HTTP call in a test without a mock or VCR cassette",
        )];
    }
    Vec::new()
}

pub(super) fn test_coverage_multiple_scenarios_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    let assert_count = body.matches("assert ").count();
    let has_many_setups = body.matches("# Arrange").count() + body.matches("# Setup").count() >= 2;
    if assert_count >= 6 && has_many_setups {
        return vec![make_finding(
            "test_function_covers_multiple_unrelated_scenarios",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "has 6+ assertions across multiple setups; split into focused test cases",
        )];
    }
    Vec::new()
}

pub(super) fn integration_test_without_cleanup_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // integration test signature: write to DB or filesystem
    let has_write = body.contains(".save()")
        || body.contains(".create(")
        || body.contains(".insert(")
        || body.contains("open(") && body.contains("\"w\"");
    let has_cleanup =
        body.contains("finally:") || body.contains("addCleanup") || body.contains("yield");
    if has_write && !has_cleanup {
        let line = find_line(body, ".save()", function.fingerprint.start_line)
            .or_else(|| find_line(body, ".create(", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "integration_test_writes_state_without_cleanup",
            Severity::Warning,
            file,
            function,
            line,
            "integration test writes state without visible cleanup path",
        )];
    }
    Vec::new()
}

pub(super) fn pytest_raises_no_match_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    if body.contains("pytest.raises(Exception)") && !body.contains("match=") {
        let line = find_line(
            body,
            "pytest.raises(Exception)",
            function.fingerprint.start_line,
        )
        .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "pytest_raises_without_match_parameter_on_broad_exception",
            Severity::Info,
            file,
            function,
            line,
            "uses pytest.raises(Exception) without match=; any exception message satisfies the assertion",
        )];
    }
    Vec::new()
}

pub(super) fn test_reimplements_production_logic_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Heuristic: test re-implements complex validation logic inline
    let has_direct_validation = (body.contains("if len(") || body.contains("if not re.match"))
        && body.contains("ValidationError")
        && body.contains("assert ");
    if has_direct_validation {
        let line = find_line(body, "if len(", function.fingerprint.start_line)
            .or_else(|| find_line(body, "if not re.match", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "test_re_implements_production_validation_logic",
            Severity::Info,
            file,
            function,
            line,
            "re-implements production validation logic in a test; use the production validator directly",
        )];
    }
    Vec::new()
}

pub(super) fn test_imports_private_module_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function && !file.is_test_file {
        return Vec::new();
    }
    let imports: Vec<&str> = file.imports.iter().map(|i| i.path.as_str()).collect();
    for imp in imports {
        if imp.contains("._") || imp.contains("._impl") || imp.contains("._internal") {
            return vec![Finding {
                rule_id: "test_imports_private_production_module".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line,
                end_line: function.fingerprint.start_line,
                message: format!(
                    "test file imports private production module {}; test through public API",
                    imp
                ),
                evidence: vec![format!("private_import={imp}")],
            }];
        }
    }
    Vec::new()
}

pub(super) fn mock_return_incompatible_type_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Detect mock.return_value = None when return type is not optional
    if body.contains(".return_value = None") || body.contains("return_value=None") {
        let line = find_line(body, "return_value", function.fingerprint.start_line)
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "mock_return_value_is_incompatible_type_with_real_signature",
            Severity::Info,
            file,
            function,
            line,
            "mock return_value may be incompatible with the real function's return type annotation",
        )];
    }
    Vec::new()
}

pub(super) fn test_unittest_duplicated_setup_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Detect duplicate setUp body without shared base class
    if function.fingerprint.name == "setUp"
        && body.len() > 200
        && !body.contains("super().setUp")
        && !body.contains("TestBase")
        && !body.contains("BaseTestCase")
    {
        return vec![make_finding(
            "unittest_test_class_duplicates_setup_without_base_class",
            Severity::Info,
            file,
            function,
            function.fingerprint.start_line,
            "test class provides large setUp without inheriting from a shared base class",
        )];
    }
    Vec::new()
}

pub(super) fn test_depends_on_sibling_side_effects_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !function.is_test_function {
        return Vec::new();
    }
    let body = &function.body_text;
    // Detect reading from module-level state that looks like it was set by another test
    if body.contains("_state[") || body.contains("SHARED_STATE") || body.contains("global_result") {
        let line = find_line(body, "_state[", function.fingerprint.start_line)
            .or_else(|| find_line(body, "SHARED_STATE", function.fingerprint.start_line))
            .unwrap_or(function.fingerprint.start_line);
        return vec![make_finding(
            "test_depends_on_sibling_test_side_effects",
            Severity::Warning,
            file,
            function,
            line,
            "test reads shared mutable state that may be set by a sibling test; ensure isolation",
        )];
    }
    Vec::new()
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

pub(super) fn project_agnostic_discipline_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let sig = function.signature_text.replace('\n', " ");
    let body = &function.body_text;
    let lower_body = body.to_ascii_lowercase();
    let line = function.fingerprint.start_line;

    if contains_any(&sig, &["=False", "=True", ": bool"]) && lower_body.matches("if ").count() >= 2
    {
        findings.push(make_finding(
            "boolean_flag_parameter_controls_unrelated_behaviors",
            Severity::Info,
            file,
            function,
            line,
            "uses a boolean parameter to select materially different behaviors",
        ));
    }

    if contains_any(&lower_body, &["validate", "schema", "check_"])
        && contains_any(&lower_body, &["execute", "process", "run", "save"])
        && contains_any(&lower_body, &["format", "serialize", "json", "render"])
    {
        findings.push(make_finding(
            "function_body_contains_setup_validation_execution_and_formatting_all_at_once",
            Severity::Info,
            file,
            function,
            line,
            "packs setup, validation, execution, and formatting concerns into one function body",
        ));
    }

    if lower_body.matches("if ").count() + lower_body.matches("elif ").count() >= 4
        && lower_body.matches("    if ").count() >= 2
    {
        findings.push(make_finding(
            "condition_tree_nests_past_two_business_decision_levels",
            Severity::Info,
            file,
            function,
            line,
            "contains a deeply nested decision tree that would benefit from decomposition",
        ));
    }

    if contains_any(
        body,
        &["requests.", "httpx.", "open(", "subprocess.", ".read_text("],
    ) && contains_any(&lower_body, &["if not ", "raise valueerror", "assert "])
    {
        findings.push(make_finding(
            "expensive_work_starts_before_input_validation",
            Severity::Warning,
            file,
            function,
            line,
            "starts expensive work before validating cheap preconditions",
        ));
    }

    if body.matches(".close()").count() >= 2 || body.matches("finally:").count() >= 2 {
        findings.push(make_finding(
            "duplicated_cleanup_paths_instead_of_context_manager",
            Severity::Info,
            file,
            function,
            line,
            "duplicates cleanup logic that could likely be owned by a helper or context manager",
        ));
    }

    let read_like_name = contains_any(
        &function.fingerprint.name.to_ascii_lowercase(),
        &["get", "list", "format", "render", "load"],
    );
    if read_like_name && contains_any(&lower_body, &[".write(", ".save(", "requests.", "open("]) {
        findings.push(make_finding(
            "helper_name_hides_mutation_or_io_side_effect",
            Severity::Info,
            file,
            function,
            line,
            "helper name sounds pure but the body performs mutation or I/O",
        ));
    }

    if body.contains("self.")
        && (lower_body.contains("self._") || lower_body.matches("self.").count() >= 4)
        && body.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("self.") && trimmed.contains(" = ") && !trimmed.contains(" == ")
        })
        && contains_any(&lower_body, &["return {", "return json", "return str("])
    {
        findings.push(make_finding(
            "method_mutates_state_and_emits_user_facing_representation",
            Severity::Info,
            file,
            function,
            line,
            "both mutates object state and formats output in one method",
        ));
    }

    if let Some(loop_line) = loop_with_recovery_logging_line(body, function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "loop_interleaves_core_work_logging_and_recovery_logic",
            Severity::Info,
            file,
            function,
            loop_line,
            "interleaves core work, logging, and recovery logic in the same loop body",
        ));
    }

    if contains_any(&lower_body, &["isinstance(", "type("])
        && contains_any(&lower_body, &["mode", "kind", "strategy"])
    {
        findings.push(make_finding(
            "type_branch_and_mode_branch_compounded_in_same_function",
            Severity::Info,
            file,
            function,
            line,
            "branches on both runtime type and mode-like values in the same function",
        ));
    }

    if body.matches("try:").count() >= 2 && body.matches("finally:").count() >= 1 {
        findings.push(make_finding(
            "repeated_try_finally_release_pattern_not_extracted",
            Severity::Info,
            file,
            function,
            line,
            "repeats try/finally release patterns that could be extracted behind one boundary",
        ));
    }

    let primitive_param_count = sig.matches(": str").count()
        + sig.matches(": int").count()
        + sig.matches(": bool").count()
        + sig.matches(": float").count();
    if primitive_param_count >= 5 {
        findings.push(make_finding(
            "long_parameter_list_of_primitives_without_options_object",
            Severity::Info,
            file,
            function,
            line,
            "takes many primitive parameters and may need a focused options object",
        ));
    }

    if contains_any(
        &lower_body,
        &["not enabled", "not should", "enabled = not", "valid = not"],
    ) {
        findings.push(make_finding(
            "negated_boolean_reassigned_and_inverted_again",
            Severity::Info,
            file,
            function,
            line,
            "reassigns and re-inverts boolean state instead of naming the intended condition directly",
        ));
    }

    if body.matches("self.").count() >= 2
        && contains_any(&lower_body, &["other.", "peer.", "target."])
    {
        findings.push(make_finding(
            "method_mutates_self_and_peer_object_in_same_block",
            Severity::Warning,
            file,
            function,
            line,
            "mutates both local state and peer state inside the same unit of work",
        ));
    }

    if contains_any(
        &lower_body,
        &["items[0]", "first_item", "return process_one("],
    ) && contains_any(
        &function.fingerprint.name.to_ascii_lowercase(),
        &["batch", "bulk"],
    ) {
        findings.push(make_finding(
            "batch_api_silently_falls_back_to_single_item_semantics",
            Severity::Info,
            file,
            function,
            line,
            "batch-oriented function appears to fall back to single-item semantics",
        ));
    }

    if lower_body.matches("if not ").count() >= 3
        || lower_body.matches("if value is None").count() >= 2
    {
        findings.push(make_finding(
            "same_precondition_checked_in_multiple_sibling_branches",
            Severity::Info,
            file,
            function,
            line,
            "repeats the same precondition in several branches instead of normalizing once",
        ));
    }

    if body.contains("return (")
        || (body.contains("return {") && lower_body.matches("return ").count() >= 2)
    {
        findings.push(make_finding(
            "function_returns_multiple_unlabeled_shape_variants",
            Severity::Info,
            file,
            function,
            line,
            "returns multiple unlabeled shapes from the same function",
        ));
    }

    if file.top_level_bindings.len() >= 8
        && file.functions.len() >= 8
        && contains_any(&lower_body, &["class ", "def ", "return "])
    {
        findings.push(make_finding(
            "module_mixes_constants_types_helpers_and_execution_flow",
            Severity::Info,
            file,
            function,
            line,
            "lives in a module that mixes constants, helper logic, and execution flow densely",
        ));
    }

    if contains_any(
        &lower_body,
        &[
            "call init first",
            "must initialize",
            "must load before",
            "after setup call",
        ],
    ) {
        findings.push(make_finding(
            "correctness_depends_on_specific_call_order_not_encoded_in_api",
            Severity::Info,
            file,
            function,
            line,
            "documents call ordering in text instead of encoding the safe sequence in the API",
        ));
    }

    findings
}

fn loop_with_recovery_logging_line(body: &str, start_line: usize) -> Option<usize> {
    let lines: Vec<&str> = body.lines().collect();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !(trimmed.starts_with("for ") || trimmed.starts_with("while ")) {
            continue;
        }

        let loop_indent = indent_level(line);
        let mut block_lines = Vec::new();
        for block_line in lines.iter().skip(index + 1) {
            let block_trimmed = block_line.trim();
            if block_trimmed.is_empty() {
                continue;
            }
            if indent_level(block_line) <= loop_indent {
                break;
            }
            block_lines.push(*block_line);
        }

        let block_text = block_lines.join("\n").to_ascii_lowercase();
        if contains_any(
            &block_text,
            &[
                "logger.exception(",
                "logger.error(",
                "logging.exception(",
                "logging.error(",
            ],
        ) && contains_any(&block_text, &["except ", "retry", "continue"])
        {
            return Some(start_line + index);
        }
    }

    None
}
