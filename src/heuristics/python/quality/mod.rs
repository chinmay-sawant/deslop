mod function_rules;
mod module_state;

use std::collections::BTreeSet;

use crate::analysis::{ParsedFile, ParsedFunction, TopLevelCallSummary};
use crate::model::Finding;

pub(crate) const BINDING_LOCATION: &str = file!();

use self::function_rules::function_rules_findings;
use self::module_state::module_state_findings;

pub(super) const OPTION_BAG_FIELD_THRESHOLD: usize = 6;
pub(super) const OPTION_BAG_SIGNAL_THRESHOLD: usize = 4;

pub(super) fn quality_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = function_rules_findings(file, function);
    findings.extend(project_agnostic_quality_function_findings(file, function));
    findings
}

pub(super) fn quality_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = module_state_findings(file);
    findings.extend(project_agnostic_quality_file_findings(file));
    findings
}

pub(super) fn should_skip_wide_contract_function(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> bool {
    should_skip_wide_contract_rule(file)
        || (signature_looks_json_like(&function.signature_text)
            && (is_framework_route_function(function) || is_boundary_contract_module(file)))
}

pub(super) fn should_skip_weak_typing_function(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> bool {
    should_skip_wide_contract_function(file, function)
        || (function.fingerprint.name.starts_with('_')
            && signature_looks_json_like(&function.signature_text))
}

pub(super) fn body_lines(function: &ParsedFunction) -> Vec<(usize, &str)> {
    function
        .body_text
        .lines()
        .enumerate()
        .map(|(index, line)| (function.body_start_line + index, line))
        .collect()
}

pub(super) fn function_line_text(function: &ParsedFunction, line_no: usize) -> Option<&str> {
    function
        .body_text
        .lines()
        .nth(line_no.checked_sub(function.body_start_line)?)
}

pub(super) fn task_group_receivers(function: &ParsedFunction) -> BTreeSet<String> {
    body_lines(function)
        .into_iter()
        .filter_map(|(_, line)| {
            let trimmed = line.trim();
            if !trimmed.starts_with("async with ") || !trimmed.contains("TaskGroup(") {
                return None;
            }
            let (_, tail) = trimmed.split_once(" as ")?;
            let name = tail.trim().trim_end_matches(':').trim();
            is_valid_identifier(name).then(|| name.to_string())
        })
        .collect()
}

pub(super) fn split_assignment(text: &str) -> Option<(&str, &str)> {
    if text.contains("==") {
        return None;
    }
    text.split_once('=')
}

pub(super) fn task_factory_marker(text: &str) -> Option<&'static str> {
    [
        "asyncio.create_task(",
        "create_task(",
        "asyncio.ensure_future(",
        "ensure_future(",
    ]
    .into_iter()
    .find(|marker| text.contains(marker))
}

pub(super) fn contains_task_factory(text: &str) -> bool {
    task_factory_marker(text.trim()).is_some() || text.contains(".create_task(")
}

pub(super) fn task_group_create_task(text: &str, task_groups: &BTreeSet<String>) -> bool {
    let trimmed = text.trim();
    let Some((receiver, _)) = trimmed.split_once(".create_task(") else {
        return false;
    };
    task_groups.contains(receiver.trim())
}

pub(super) fn task_handle_observed(
    function: &ParsedFunction,
    task_name: &str,
    created_line: usize,
) -> bool {
    body_lines(function)
        .into_iter()
        .filter(|(line_no, _)| *line_no > created_line)
        .any(|(_, line)| {
            let trimmed = line.trim();
            trimmed.contains(&format!("await {task_name}"))
                || trimmed.contains(&format!("return {task_name}"))
                || trimmed.contains(&format!("yield {task_name}"))
                || trimmed.contains(&format!("{task_name}.result("))
                || trimmed.contains(&format!("{task_name}.exception("))
                || trimmed.contains(&format!("{task_name}.add_done_callback("))
                || trimmed.contains(&format!("{task_name}.cancel("))
                || trimmed.contains(&format!("append({task_name})"))
                || trimmed.contains(&format!("add({task_name})"))
                || trimmed.contains(&format!("register({task_name})"))
                || trimmed.contains(&format!("track({task_name})"))
                || trimmed.contains(&format!("gather({task_name}"))
                || trimmed.contains(&format!("wait({task_name}"))
        })
}

pub(super) fn looks_like_lock_context(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("lock") || lower.contains("mutex") || lower.contains("semaphore")
}

pub(super) fn indented_block<'a>(
    entries: &'a [(usize, &'a str)],
    header_index: usize,
) -> Vec<(usize, &'a str)> {
    let base_indent = indentation(entries[header_index].1);
    let mut block = Vec::new();
    for (line_no, line) in entries.iter().skip(header_index + 1) {
        if !line.trim().is_empty() && indentation(line) <= base_indent {
            break;
        }
        block.push((*line_no, *line));
    }
    block
}

fn indentation(line: &str) -> usize {
    line.chars()
        .take_while(|character| character.is_ascii_whitespace())
        .count()
}

pub(super) fn explicit_lock_acquire_name(line: &str) -> Option<String> {
    let acquire_marker = ".acquire(";
    if !line.trim_start().starts_with("await ") || !line.contains(acquire_marker) {
        return None;
    }
    let after_await = line.trim_start().trim_start_matches("await ").trim();
    let receiver = after_await.split_once(acquire_marker)?.0.trim();
    let lower = receiver.to_ascii_lowercase();
    (lower.contains("lock") || lower.contains("mutex") || lower.contains("semaphore"))
        .then(|| receiver.to_string())
}

pub(super) fn is_unrelated_await_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("await ")
        && !trimmed.contains(".acquire(")
        && !trimmed.contains(".release(")
}

pub(super) fn constant_async_sleep_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with("await ")
        || (!trimmed.contains("asyncio.sleep(") && !trimmed.contains("sleep("))
    {
        return false;
    }
    let Some((_, rest)) = trimmed.split_once("sleep(") else {
        return false;
    };
    let argument = rest.split(')').next().unwrap_or_default().trim();
    !argument.is_empty()
        && argument
            .chars()
            .all(|character| character.is_ascii_digit() || matches!(character, '.' | '_' | ' '))
}

pub(super) fn has_retry_backoff_markers(text: &str) -> bool {
    [
        "backoff",
        "jitter",
        "max_retries",
        "retry_count",
        "attempt",
        "*= 2",
        "*=2",
        "**",
        "pow(",
        "exponential",
        "min(",
        "max_delay",
    ]
    .iter()
    .any(|marker| text.contains(marker))
}

pub(super) fn parameter_entries(signature_text: &str) -> Vec<String> {
    let Some(start) = signature_text.find('(') else {
        return Vec::new();
    };
    let Some(end) = signature_text.rfind(')') else {
        return Vec::new();
    };
    split_top_level_commas(&signature_text[start + 1..end])
}

fn split_top_level_commas(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut previous_was_escape = false;

    for character in text.chars() {
        match character {
            '\\' if in_single || in_double => {
                previous_was_escape = !previous_was_escape;
                current.push(character);
                continue;
            }
            '\'' if !in_double && !previous_was_escape => in_single = !in_single,
            '"' if !in_single && !previous_was_escape => in_double = !in_double,
            '(' if !in_single && !in_double => paren_depth += 1,
            ')' if !in_single && !in_double => paren_depth = paren_depth.saturating_sub(1),
            '[' if !in_single && !in_double => bracket_depth += 1,
            ']' if !in_single && !in_double => bracket_depth = bracket_depth.saturating_sub(1),
            '{' if !in_single && !in_double => brace_depth += 1,
            '}' if !in_single && !in_double => brace_depth = brace_depth.saturating_sub(1),
            ',' if !in_single
                && !in_double
                && paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0 =>
            {
                let piece = current.trim();
                if !piece.is_empty() {
                    parts.push(piece.to_string());
                }
                current.clear();
                continue;
            }
            _ => {}
        }

        if character != '\\' {
            previous_was_escape = false;
        }
        current.push(character);
    }

    let tail = current.trim();
    if !tail.is_empty() {
        parts.push(tail.to_string());
    }
    parts
}

pub(super) fn mutable_default_kind(default_text: &str) -> Option<&'static str> {
    let normalized = default_text.trim();
    if normalized.starts_with('[') && normalized.ends_with(']') {
        return Some("list");
    }
    if normalized.starts_with('{') && normalized.ends_with('}') {
        return Some(if normalized.contains(':') {
            "dict"
        } else {
            "set"
        });
    }
    matches!(normalized, "list()" | "dict()" | "set()" | "defaultdict()").then_some(
        if normalized.starts_with("set") {
            "set"
        } else if normalized.starts_with("dict") || normalized.starts_with("defaultdict") {
            "dict"
        } else {
            "list"
        },
    )
}

pub(super) fn heavy_post_init_detail(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Option<(usize, String)> {
    for call in &function.calls {
        let resolved = resolve_call_path(file, call.receiver.as_deref(), &call.name);
        let lower_resolved = resolved.to_ascii_lowercase();
        if is_heavy_post_init_call(&lower_resolved) {
            return Some((call.line, format!("heavy_call={resolved}")));
        }
    }

    body_lines(function)
        .into_iter()
        .find_map(|(line_no, line)| {
            let lower = line.trim().to_ascii_lowercase();
            (lower.contains("subprocess.")
                || lower.contains(" open(")
                || lower.starts_with("open("))
            .then(|| (line_no, format!("heavy_line={}", line.trim())))
        })
}

fn is_heavy_post_init_call(resolved_call: &str) -> bool {
    resolved_call.starts_with("requests.")
        || resolved_call.starts_with("httpx.")
        || resolved_call.starts_with("urllib.")
        || resolved_call.starts_with("subprocess.")
        || resolved_call == "open"
        || resolved_call.ends_with(".read_text")
        || resolved_call.ends_with(".read_bytes")
        || resolved_call.ends_with(".client")
        || resolved_call.ends_with(".resource")
        || resolved_call.contains("create_engine")
        || resolved_call.contains("sessionmaker")
        || resolved_call.contains("redis")
        || resolved_call.contains("mongoclient")
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn project_agnostic_quality_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let body = &function.body_text;
    let lower_body = body.to_ascii_lowercase();
    let sig = function.signature_text.replace('\n', " ");
    let line = function.fingerprint.start_line;

    let push = |rule_id: &str, severity: crate::model::Severity, message: String| Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message,
        evidence: vec![format!("function={}", function.fingerprint.name)],
    };

    if sig.contains("->") && body.contains("return None") && body.contains("return ") {
        findings.push(push(
            "public_api_returns_none_or_value_without_explicit_optional_contract",
            crate::model::Severity::Info,
            format!(
                "function {} returns None on some paths without an explicit optional contract",
                function.fingerprint.name
            ),
        ));
    }

    if except_block_returns_plausible_default(body) {
        findings.push(push(
            "fallback_branch_swallows_invariant_violation_and_returns_plausible_default",
            crate::model::Severity::Warning,
            format!(
                "function {} returns a plausible default from a failure path and may hide invariant violations",
                function.fingerprint.name
            ),
        ));
    }

    if body.contains("except Exception")
        && contains_any(&lower_body, &["typeerror", "keyerror", "attributeerror"])
    {
        findings.push(push(
            "broad_except_used_to_mask_type_or_shape_bug",
            crate::model::Severity::Warning,
            format!(
                "function {} uses a broad except that can conceal type or shape bugs",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(body, &["list(set(", "sorted(set("]) {
        findings.push(push(
            "order_dependent_set_to_list_conversion_exposed_in_public_result",
            crate::model::Severity::Info,
            format!(
                "function {} converts set-like data to list in a public result path",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(
        body,
        &[
            "requests.get(",
            "requests.post(",
            "httpx.get(",
            "httpx.post(",
        ],
    ) && !contains_any(body, &["timeout=", "Timeout("])
    {
        findings.push(push(
            "default_timeout_missing_on_external_boundary_wrapper",
            crate::model::Severity::Warning,
            format!(
                "function {} wraps an external call without a visible timeout policy",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(body, &["== 0.1", "== 0.2", "== result", "!= result"])
        && contains_any(&lower_body, &["float", "round", "ratio", "score"])
    {
        findings.push(push(
            "float_equality_controls_branching_on_computed_values",
            crate::model::Severity::Info,
            format!(
                "function {} uses exact float equality in control flow",
                function.fingerprint.name
            ),
        ));
    }

    if !function.python_evidence().recursive_call_lines.is_empty()
        && !contains_any(&lower_body, &["max_depth", "depth >", "level >"])
    {
        findings.push(push(
            "recursive_walk_over_untrusted_input_lacks_depth_limit",
            crate::model::Severity::Warning,
            format!(
                "function {} recurses without an obvious depth limit",
                function.fingerprint.name
            ),
        ));
    }

    if sig.contains("Iterator[")
        && contains_any(body, &["yield {", "yield (", "yield value", "yield error"])
    {
        findings.push(push(
            "public_iterator_yields_heterogeneous_item_shapes",
            crate::model::Severity::Info,
            format!(
                "function {} appears to yield heterogeneous item shapes from one iterator contract",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["payload[", "data["])
        && contains_any(&lower_body, &["validate", "schema", "raise"])
    {
        findings.push(push(
            "partial_update_mutates_input_before_validation_succeeds",
            crate::model::Severity::Warning,
            format!(
                "function {} mutates update payloads before all validation has succeeded",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["cache_key =", "key = str("]) {
        findings.push(push(
            "cache_key_derived_from_stringified_mutable_object",
            crate::model::Severity::Info,
            format!(
                "function {} derives a cache key from stringified mutable state",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["for key in mapping", "for key in data"])
        && !contains_any(&lower_body, &["sorted(", "order_by", "key="])
    {
        findings.push(push(
            "sort_order_depends_on_non_explicit_mapping_iteration_semantics",
            crate::model::Severity::Info,
            format!(
                "function {} appears to rely on mapping iteration order without making it explicit",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["set(", "seen ="])
        && contains_any(&lower_body, &["return", "yield"])
    {
        findings.push(push(
            "duplicate_items_silently_dropped_without_contract_signal",
            crate::model::Severity::Info,
            format!(
                "function {} silently deduplicates items without making the contract explicit",
                function.fingerprint.name
            ),
        ));
    }

    if sig.contains("datetime") && !contains_any(&sig, &["timezone", "tzinfo", "AwareDatetime"]) {
        findings.push(push(
            "timezone_naive_datetime_accepted_in_public_contract",
            crate::model::Severity::Warning,
            format!(
                "function {} accepts datetimes without an explicit timezone contract",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["open(", "write_text(", "write_bytes("])
        && contains_any(&lower_body, &["replace", "atomic"])
    {
        findings.push(push(
            "atomic_replace_semantics_implemented_with_non_atomic_file_write",
            crate::model::Severity::Warning,
            format!(
                "function {} appears to promise atomic replace semantics with non-atomic file writes",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&sig, &["mode: str", "kind: str", "strategy: str"]) {
        findings.push(push(
            "string_mode_parameter_replaces_enum_or_literal_contract",
            crate::model::Severity::Info,
            format!(
                "function {} uses string mode parameters that may want enums or literal types",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["errors.append(", "failed.append("])
        && contains_any(
            &lower_body,
            &[
                "return success",
                "return {\"ok\": true",
                "return {'ok': true",
            ],
        )
    {
        findings.push(push(
            "helper_returns_success_shape_even_when_substeps_partially_fail",
            crate::model::Severity::Warning,
            format!(
                "function {} returns success-like shapes while collecting partial failures",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["merge(", "combine(", "update("])
        && !contains_any(&lower_body, &["assert", "raise", "duplicate"])
    {
        findings.push(push(
            "comparison_or_merge_logic_assumes_unique_keys_without_assertion",
            crate::model::Severity::Info,
            format!(
                "function {} merges or compares records without asserting uniqueness assumptions",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["requests.", "httpx.", "open(", "write("])
        && contains_any(&lower_body, &["validate", "schema"])
    {
        findings.push(push(
            "validation_only_happens_after_expensive_side_effect_has_started",
            crate::model::Severity::Warning,
            format!(
                "function {} begins expensive work before completing validation",
                function.fingerprint.name
            ),
        ));
    }

    findings
}

fn except_block_returns_plausible_default(body: &str) -> bool {
    let lines: Vec<&str> = body.lines().collect();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with("except:") || trimmed.starts_with("except ")) {
            continue;
        }

        let except_indent = indentation(line);
        let mut has_default_return = false;
        let mut has_raise = false;

        for block_line in lines.iter().skip(index + 1) {
            let block_trimmed = block_line.trim();
            if block_trimmed.is_empty() {
                continue;
            }

            if indentation(block_line) <= except_indent {
                break;
            }

            let normalized = block_trimmed.to_ascii_lowercase();
            if normalized == "raise" || normalized.starts_with("raise ") {
                has_raise = true;
            }

            if normalized == "return default"
                || normalized == "return {}"
                || normalized == "return []"
                || normalized.starts_with("return {")
                || normalized.starts_with("return [")
            {
                has_default_return = true;
            }
        }

        if has_default_return && !has_raise {
            return true;
        }
    }

    false
}

fn project_agnostic_quality_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let _ = file;
    Vec::new()
}

pub(super) fn wide_contract_markers(text: &str) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let mut markers = Vec::new();
    if lower.contains("any") {
        markers.push("Any".to_string());
    }
    if lower.contains(": object") || lower.contains("-> object") {
        markers.push("object".to_string());
    }
    if lower.contains("dict[str, any]") || lower.contains("dict[str, typing.any]") {
        markers.push("dict[str, Any]".to_string());
    }
    markers.sort();
    markers.dedup();
    markers
}

pub(super) fn should_skip_wide_contract_rule(file: &ParsedFile) -> bool {
    file.path.components().any(|component| {
        let part = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        matches!(
            part.as_str(),
            "migrations"
                | "migration"
                | "serializers"
                | "serializer"
                | "storage"
                | "repositories"
                | "repository"
                | "exporters"
                | "exporter"
        )
    }) || matches!(
        file.path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.to_ascii_lowercase())
            .as_deref(),
        Some("database" | "repository" | "serializer" | "export")
    )
}

pub(super) fn typed_dict_field_is_optional(annotation_text: Option<&str>) -> bool {
    annotation_text.is_some_and(|annotation| {
        let lower = annotation.to_ascii_lowercase();
        lower.contains("notrequired[") || lower.contains("optional[") || lower.contains("| none")
    })
}

pub(super) fn key_guarded_before(entries: &[(usize, &str)], line_no: usize, key: &str) -> bool {
    entries
        .iter()
        .filter(|(entry_line, _)| *entry_line < line_no)
        .any(|(_, line)| {
            let trimmed = line.trim();
            trimmed.contains(&format!(".get('{key}'"))
                || trimmed.contains(&format!(".get(\"{key}\""))
                || trimmed.contains(&format!("'{key}' in "))
                || trimmed.contains(&format!("\"{key}\" in "))
        })
}

pub(super) fn resolve_call_path(file: &ParsedFile, receiver: Option<&str>, name: &str) -> String {
    if let Some(receiver) = receiver {
        if let Some(import_path) = file
            .imports
            .iter()
            .find(|import| import.alias == receiver)
            .map(|import| import.path.as_str())
        {
            return if import_path.ends_with(name) {
                import_path.to_string()
            } else {
                format!("{import_path}.{name}")
            };
        }
        return format!("{receiver}.{name}");
    }

    file.imports
        .iter()
        .find(|import| import.alias == name)
        .map(|import| import.path.clone())
        .unwrap_or_else(|| name.to_string())
}

pub(super) fn resolve_top_level_call_path(file: &ParsedFile, call: &TopLevelCallSummary) -> String {
    resolve_call_path(file, call.receiver.as_deref(), &call.name)
}

pub(super) fn nearby_tar_guard(entries: &[(usize, &str)], line_no: usize) -> bool {
    entries
        .iter()
        .filter(|(entry_line, _)| {
            *entry_line <= line_no && line_no.saturating_sub(*entry_line) <= 3
        })
        .any(|(_, line)| {
            let lower = line.trim().to_ascii_lowercase();
            lower.contains("validate")
                || lower.contains("safe_extract")
                || lower.contains("is_within_directory")
        })
}

pub(super) fn is_tempfile_cleanup_line(line: &str) -> bool {
    line.contains(".cleanup(")
        || line.contains("unlink(")
        || line.contains("remove(")
        || line.contains("rmtree(")
}

pub(super) fn temp_resource_cleaned_later(entries: &[(usize, &str)], line_no: usize) -> bool {
    entries
        .iter()
        .filter(|(entry_line, _)| *entry_line > line_no)
        .any(|(_, line)| is_tempfile_cleanup_line(&line.to_ascii_lowercase()))
}

pub(super) fn option_bag_optional_field(
    annotation_text: Option<&str>,
    default_text: Option<&str>,
) -> bool {
    annotation_text.is_some_and(|annotation| {
        let lower = annotation.to_ascii_lowercase();
        lower.contains("optional[") || lower.contains("| none") || lower.contains("notrequired[")
    }) || default_text.is_some_and(|default| default.trim() == "None")
}

pub(super) fn option_bag_boolean_field(
    annotation_text: Option<&str>,
    default_text: Option<&str>,
) -> bool {
    annotation_text.is_some_and(|annotation| annotation.to_ascii_lowercase().contains("bool"))
        || default_text.is_some_and(|default| matches!(default.trim(), "True" | "False"))
}

pub(super) fn is_network_call_path(resolved_path: &str, call_name: &str) -> bool {
    let lower = resolved_path.to_ascii_lowercase();
    let lower_name = call_name.to_ascii_lowercase();
    (lower.starts_with("requests.")
        || lower.starts_with("httpx.")
        || lower.starts_with("urllib.")
        || lower.starts_with("socket."))
        && matches!(
            lower_name.as_str(),
            "get"
                | "post"
                | "put"
                | "patch"
                | "delete"
                | "request"
                | "urlopen"
                | "connect"
                | "create_connection"
        )
}

pub(super) fn is_file_io_call(call: &TopLevelCallSummary, resolved_path: &str) -> bool {
    let lower = resolved_path.to_ascii_lowercase();
    matches!(
        call.name.as_str(),
        "open"
            | "read_text"
            | "read_bytes"
            | "write_text"
            | "write_bytes"
            | "listdir"
            | "scandir"
            | "walk"
            | "glob"
            | "iglob"
            | "rglob"
    ) || lower == "open"
        || lower.ends_with(".read_text")
        || lower.ends_with(".read_bytes")
        || lower.ends_with(".write_text")
        || lower.ends_with(".write_bytes")
        || lower.ends_with(".listdir")
        || lower.ends_with(".scandir")
        || lower.ends_with(".walk")
        || lower.contains("path(")
            && matches!(
                call.name.as_str(),
                "read_text" | "read_bytes" | "write_text" | "write_bytes"
            )
}

pub(super) fn is_subprocess_call(resolved_path: &str) -> bool {
    matches!(
        resolved_path.to_ascii_lowercase().as_str(),
        "subprocess.run"
            | "subprocess.popen"
            | "subprocess.check_output"
            | "subprocess.check_call"
            | "subprocess.call"
            | "os.system"
    )
}

pub(super) fn is_client_constructor_text(lower_value: &str) -> bool {
    [
        "httpx.client(",
        "requests.session(",
        "boto3.client(",
        "boto3.resource(",
        "create_engine(",
        "sessionmaker(",
        "redis(",
        "redis.redis(",
        "mongoclient(",
        "dockerclient.from_env(",
    ]
    .iter()
    .any(|marker| lower_value.contains(marker))
}

pub(super) fn mutates_binding(function: &ParsedFunction, binding_name: &str) -> bool {
    let indexed_assignment = format!("{binding_name}[");
    let global_marker = format!("global {binding_name}");
    function.body_text.lines().map(str::trim).any(|line| {
        line.contains(&format!("{binding_name}.append("))
            || line.contains(&format!("{binding_name}.extend("))
            || line.contains(&format!("{binding_name}.update("))
            || line.contains(&format!("{binding_name}.add("))
            || line.contains(&format!("{binding_name}.remove("))
            || line.contains(&format!("{binding_name}.discard("))
            || line.contains(&format!("{binding_name}.pop("))
            || line.contains(&format!("{binding_name}.clear("))
            || line.starts_with(&indexed_assignment)
            || line == global_marker
            || (line.starts_with(&format!("{binding_name} ="))
                && function.body_text.contains(&global_marker))
    })
}

pub(super) fn is_config_load_call(resolved_path: &str, lower_text: &str) -> bool {
    matches!(
        resolved_path,
        "os.getenv" | "os.environ.get" | "dotenv.load_dotenv" | "dotenv.dotenv_values"
    ) || is_config_load_text(lower_text)
}

pub(super) fn is_config_load_text(lower_text: &str) -> bool {
    (lower_text.contains("os.getenv(")
        || lower_text.contains("os.environ.get(")
        || lower_text.contains("load_dotenv(")
        || lower_text.contains("dotenv_values(")
        || lower_text.contains("settings")
        || lower_text.contains("config")
        || lower_text.contains("secret")
        || lower_text.contains(".env"))
        && (lower_text.contains("read_text(")
            || lower_text.contains("read_bytes(")
            || lower_text.contains("yaml.")
            || lower_text.contains("json.load")
            || lower_text.contains("getenv")
            || lower_text.contains("dotenv"))
}

pub(super) fn should_skip_import_time_config_finding(
    file: &ParsedFile,
    resolved_path: &str,
    lower_text: &str,
) -> bool {
    is_framework_entrypoint_module(file) && is_env_bootstrap_lookup(resolved_path, lower_text)
}

pub(super) fn is_valid_identifier(candidate: &str) -> bool {
    let mut characters = candidate.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }

    characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn signature_looks_json_like(signature_text: &str) -> bool {
    let lower = signature_text
        .replace(char::is_whitespace, "")
        .to_ascii_lowercase();
    [
        "dict[str,any]",
        "dict[str,typing.any]",
        "list[dict[str,any]]",
        "list[dict[str,typing.any]]",
        "mapping[str,any]",
        "mapping[str,typing.any]",
        "sequence[dict[str,any]]",
        "sequence[dict[str,typing.any]]",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn is_framework_route_function(function: &ParsedFunction) -> bool {
    function.signature_text.lines().any(|line| {
        let lower = line.trim().to_ascii_lowercase();
        lower.starts_with('@')
            && [
                ".get(",
                ".post(",
                ".put(",
                ".patch(",
                ".delete(",
                ".route(",
                ".api_route(",
                ".websocket(",
                ".head(",
                ".options(",
            ]
            .iter()
            .any(|marker| lower.contains(marker))
    })
}

fn is_boundary_contract_module(file: &ParsedFile) -> bool {
    file.path.components().any(|component| {
        let part = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        matches!(part.as_str(), "api" | "views" | "handlers" | "endpoints")
    })
}

fn is_framework_entrypoint_module(file: &ParsedFile) -> bool {
    let file_name = file
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase());
    let is_entrypoint_name = matches!(file_name.as_deref(), Some("main.py" | "app.py"));
    if !is_entrypoint_name {
        return false;
    }

    file.imports.iter().any(|import| {
        let path = import.path.to_ascii_lowercase();
        path.starts_with("fastapi") || path.starts_with("flask") || path.starts_with("django")
    })
}

fn is_env_bootstrap_lookup(resolved_path: &str, lower_text: &str) -> bool {
    matches!(
        resolved_path,
        "os.getenv" | "os.environ.get" | "dotenv.load_dotenv" | "dotenv.dotenv_values"
    ) || lower_text.contains("os.getenv(")
        || lower_text.contains("os.environ.get(")
        || lower_text.contains("load_dotenv(")
        || lower_text.contains("dotenv_values(")
}
