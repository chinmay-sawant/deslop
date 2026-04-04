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
    function_rules_findings(file, function)
}

pub(super) fn quality_file_findings(file: &ParsedFile) -> Vec<Finding> {
    module_state_findings(file)
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
            "migrations" | "migration" | "serializers" | "serializer"
        )
    })
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
