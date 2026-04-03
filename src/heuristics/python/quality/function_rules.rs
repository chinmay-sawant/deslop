use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::{
    body_lines, constant_async_sleep_line, contains_task_factory, explicit_lock_acquire_name,
    function_line_text, has_retry_backoff_markers, heavy_post_init_detail, indented_block,
    is_subprocess_call, is_tempfile_cleanup_line, is_unrelated_await_line, is_valid_identifier,
    key_guarded_before, looks_like_lock_context, mutable_default_kind, nearby_tar_guard,
    parameter_entries, resolve_call_path, should_skip_wide_contract_rule, split_assignment,
    task_factory_marker, task_group_create_task, task_group_receivers, task_handle_observed,
    temp_resource_cleaned_later, typed_dict_field_is_optional, wide_contract_markers,
};

pub(super) fn function_rules_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(untracked_asyncio_task_findings(file, function));
    findings.extend(background_task_exception_unobserved_findings(
        file, function,
    ));
    findings.extend(async_lock_held_across_await_findings(file, function));
    findings.extend(async_retry_sleep_without_backoff_findings(file, function));
    findings.extend(mutable_default_argument_findings(file, function));
    findings.extend(dataclass_heavy_post_init_findings(file, function));
    findings.extend(public_any_type_leak_function_findings(file, function));
    findings.extend(typeddict_unchecked_access_findings(file, function));
    findings.extend(unsafe_yaml_loader_findings(file, function));
    findings.extend(pickle_deserialization_boundary_findings(file, function));
    findings.extend(subprocess_shell_true_findings(file, function));
    findings.extend(tar_extractall_unfiltered_findings(file, function));
    findings.extend(tempfile_without_cleanup_findings(file, function));
    findings
}

fn untracked_asyncio_task_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !function.python_evidence().is_async {
        return Vec::new();
    }

    let task_groups = task_group_receivers(function);
    body_lines(function)
        .into_iter()
        .filter_map(|(line_no, line)| {
            let trimmed = line.trim();
            let marker = task_factory_marker(trimmed)?;
            if task_group_create_task(trimmed, &task_groups) {
                return None;
            }
            let ignored_binding = split_assignment(trimmed)
                .and_then(|(left, right)| {
                    (left.trim() == "_" && contains_task_factory(right)).then_some(right)
                })
                .is_some();
            if !trimmed.starts_with(marker) && !ignored_binding {
                return None;
            }

            Some(Finding {
                rule_id: "untracked_asyncio_task".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: line_no,
                end_line: line_no,
                message: format!(
                    "function {} starts an asyncio task without tracking the returned handle",
                    function.fingerprint.name
                ),
                evidence: vec![format!("task_creation={}", trimmed)],
            })
        })
        .collect()
}

fn background_task_exception_unobserved_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !function.python_evidence().is_async {
        return Vec::new();
    }

    let task_groups = task_group_receivers(function);
    body_lines(function)
        .into_iter()
        .filter_map(|(line_no, line)| {
            let trimmed = line.trim();
            let (left, right) = split_assignment(trimmed)?;
            let task_name = left.trim();
            if !is_valid_identifier(task_name)
                || !contains_task_factory(right)
                || task_group_create_task(right.trim(), &task_groups)
                || task_handle_observed(function, task_name, line_no)
            {
                return None;
            }

            Some(Finding {
                rule_id: "background_task_exception_unobserved".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: line_no,
                end_line: line_no,
                message: format!(
                    "function {} creates background task {} without an obvious await, callback, or supervisor path",
                    function.fingerprint.name, task_name
                ),
                evidence: vec![
                    format!("task_binding={task_name}"),
                    format!("creation={}", right.trim()),
                ],
            })
        })
        .collect()
}

fn async_lock_held_across_await_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !function.python_evidence().is_async {
        return Vec::new();
    }

    let entries = body_lines(function);
    let mut findings = Vec::new();

    for (index, (line_no, line)) in entries.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("async with ") && looks_like_lock_context(trimmed) {
            let block = indented_block(&entries, index);
            if let Some((await_line, await_text)) = block
                .into_iter()
                .find(|(_, block_line)| is_unrelated_await_line(block_line))
            {
                findings.push(Finding {
                    rule_id: "async_lock_held_across_await".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: await_line,
                    end_line: await_line,
                    message: format!(
                        "function {} awaits while still inside an async lock scope",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("lock_scope_line={line_no}"),
                        format!("await_line={}", await_text.trim()),
                    ],
                });
            }
            continue;
        }

        let Some(lock_name) = explicit_lock_acquire_name(trimmed) else {
            continue;
        };
        let later_lines = entries.iter().skip(index + 1);
        for (later_line_no, later_line) in later_lines {
            let later_trimmed = later_line.trim();
            if later_trimmed.contains(&format!("{lock_name}.release(")) {
                break;
            }
            if is_unrelated_await_line(later_trimmed) {
                findings.push(Finding {
                    rule_id: "async_lock_held_across_await".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: *later_line_no,
                    end_line: *later_line_no,
                    message: format!(
                        "function {} awaits after acquiring {} and before releasing it",
                        function.fingerprint.name, lock_name
                    ),
                    evidence: vec![
                        format!("lock_acquire_line={line_no}"),
                        format!("await_line={}", later_trimmed),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn async_retry_sleep_without_backoff_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !function.python_evidence().is_async {
        return Vec::new();
    }

    let entries = body_lines(function);
    let mut findings = Vec::new();
    for (index, (line_no, line)) in entries.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("for ") && !trimmed.starts_with("while ") {
            continue;
        }

        let block = indented_block(&entries, index);
        let Some((sleep_line, sleep_text)) = block
            .iter()
            .find(|(_, block_line)| constant_async_sleep_line(block_line.trim()))
        else {
            continue;
        };

        let combined = std::iter::once(trimmed)
            .chain(block.iter().map(|(_, block_line)| block_line.trim()))
            .collect::<Vec<_>>()
            .join("\n")
            .to_ascii_lowercase();
        if has_retry_backoff_markers(&combined) {
            continue;
        }

        findings.push(Finding {
            rule_id: "async_retry_sleep_without_backoff".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *sleep_line,
            end_line: *sleep_line,
            message: format!(
                "function {} retries async work with a fixed sleep and no obvious backoff policy",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("loop_line={line_no}"),
                format!("sleep_call={}", sleep_text.trim()),
            ],
        });
    }

    findings
}

fn mutable_default_argument_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    parameter_entries(&function.signature_text)
        .into_iter()
        .filter_map(|entry| {
            let (name, default) = entry.split_once('=')?;
            let name = name
                .split(':')
                .next()
                .unwrap_or(name)
                .trim()
                .trim_start_matches('*');
            let default = default.trim();
            let mutable_kind = mutable_default_kind(default)?;
            if matches!(name, "self" | "cls") || name.is_empty() {
                return None;
            }

            Some(Finding {
                rule_id: "mutable_default_argument".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line,
                end_line: function.fingerprint.start_line,
                message: format!(
                    "function {} uses a mutable default argument for parameter {}",
                    function.fingerprint.name, name
                ),
                evidence: vec![
                    format!("parameter={name}"),
                    format!("default_expression={default}"),
                    format!("mutable_kind={mutable_kind}"),
                ],
            })
        })
        .collect()
}

fn dataclass_heavy_post_init_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || function.fingerprint.name != "__post_init__" {
        return Vec::new();
    }

    let Some(model_name) = function.fingerprint.receiver_type.as_deref() else {
        return Vec::new();
    };
    let Some(model) = file
        .python_models()
        .iter()
        .find(|model| model.name == model_name && model.is_dataclass)
    else {
        return Vec::new();
    };
    if !model
        .method_names
        .iter()
        .any(|method_name| method_name == "__post_init__")
    {
        return Vec::new();
    }

    let Some((line, detail)) = heavy_post_init_detail(file, function) else {
        return Vec::new();
    };

    vec![Finding {
        rule_id: "dataclass_heavy_post_init".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!(
            "dataclass {} performs boundary or heavyweight setup inside __post_init__",
            model.name
        ),
        evidence: vec![detail],
    }]
}

fn public_any_type_leak_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || function.fingerprint.name.starts_with('_') {
        return Vec::new();
    }

    let markers = wide_contract_markers(&function.signature_text);
    if markers.is_empty() || should_skip_wide_contract_rule(file) {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "public_any_type_leak".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.start_line,
        message: format!(
            "public function {} exposes a very wide type contract",
            function.fingerprint.name
        ),
        evidence: markers
            .into_iter()
            .map(|marker| format!("wide_type_marker={marker}"))
            .collect(),
    }]
}

fn typeddict_unchecked_access_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let entries = body_lines(function);
    let mut findings = Vec::new();
    for model in file
        .python_models()
        .iter()
        .filter(|model| model.is_typed_dict)
    {
        let optional_keys = model
            .fields
            .iter()
            .filter(|field| typed_dict_field_is_optional(field.annotation_text.as_deref()))
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>();
        if optional_keys.is_empty() {
            continue;
        }

        for (line_no, line) in &entries {
            let trimmed = line.trim();
            for key in &optional_keys {
                if !trimmed.contains(&format!("['{key}']"))
                    && !trimmed.contains(&format!("[\"{key}\"]"))
                {
                    continue;
                }
                if key_guarded_before(&entries, *line_no, key) {
                    continue;
                }

                findings.push(Finding {
                    rule_id: "typeddict_unchecked_access".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: *line_no,
                    end_line: *line_no,
                    message: format!(
                        "function {} indexes optional TypedDict key {} without an obvious guard",
                        function.fingerprint.name, key
                    ),
                    evidence: vec![
                        format!("typed_dict_model={}", model.name),
                        format!("unchecked_key={key}"),
                    ],
                });
                break;
            }
        }
    }

    findings
}

fn unsafe_yaml_loader_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || file.is_test_file {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter_map(|call| {
            let resolved = resolve_call_path(file, call.receiver.as_deref(), &call.name);
            let lower_resolved = resolved.to_ascii_lowercase();
            if !matches!(lower_resolved.as_str(), "yaml.load" | "yaml.full_load") {
                return None;
            }

            let line_text = function_line_text(function, call.line)?;
            let lower_line = line_text.trim().to_ascii_lowercase();
            if lower_line.contains("safe_load(")
                || lower_line.contains("safeloader")
                || lower_line.contains("csafeloader")
            {
                return None;
            }

            Some(Finding {
                rule_id: "unsafe_yaml_loader".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} uses a YAML loader that is broader than safe_load",
                    function.fingerprint.name
                ),
                evidence: vec![format!("yaml_call={}", line_text.trim())],
            })
        })
        .collect()
}

fn pickle_deserialization_boundary_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || file.is_test_file {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter_map(|call| {
            let resolved = resolve_call_path(file, call.receiver.as_deref(), &call.name);
            let lower_resolved = resolved.to_ascii_lowercase();
            if !matches!(
                lower_resolved.as_str(),
                "pickle.load" | "pickle.loads" | "dill.load" | "dill.loads"
            ) {
                return None;
            }

            Some(Finding {
                rule_id: "pickle_deserialization_boundary".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} deserializes pickled data at a trust boundary",
                    function.fingerprint.name
                ),
                evidence: vec![format!("deserializer={resolved}")],
            })
        })
        .collect()
}

fn subprocess_shell_true_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || file.is_test_file {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter_map(|call| {
            let resolved = resolve_call_path(file, call.receiver.as_deref(), &call.name);
            if !is_subprocess_call(&resolved) {
                return None;
            }
            let line_text = function_line_text(function, call.line)?;
            let lower_line = line_text.to_ascii_lowercase();
            if !lower_line.contains("shell=true") && !lower_line.contains("shell = true") {
                return None;
            }

            Some(Finding {
                rule_id: "subprocess_shell_true".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} enables shell=True in a subprocess boundary",
                    function.fingerprint.name
                ),
                evidence: vec![format!("subprocess_call={}", line_text.trim())],
            })
        })
        .collect()
}

fn tar_extractall_unfiltered_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || file.is_test_file {
        return Vec::new();
    }

    let entries = body_lines(function);
    function
        .calls
        .iter()
        .filter_map(|call| {
            let resolved = resolve_call_path(file, call.receiver.as_deref(), &call.name);
            let lower_resolved = resolved.to_ascii_lowercase();
            if !lower_resolved.ends_with("extractall") {
                return None;
            }
            let line_text = function_line_text(function, call.line)?;
            let lower_line = line_text.to_ascii_lowercase();
            if lower_line.contains("filter=") || lower_line.contains("members=") {
                return None;
            }
            if nearby_tar_guard(&entries, call.line) {
                return None;
            }

            Some(Finding {
                rule_id: "tar_extractall_unfiltered".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} extracts a tar archive without an obvious filter or path guard",
                    function.fingerprint.name
                ),
                evidence: vec![format!("tar_call={}", line_text.trim())],
            })
        })
        .collect()
}

fn tempfile_without_cleanup_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || file.is_test_file {
        return Vec::new();
    }

    let entries = body_lines(function);
    let has_cleanup = function
        .body_text
        .to_ascii_lowercase()
        .lines()
        .any(is_tempfile_cleanup_line);

    function
        .calls
        .iter()
        .filter_map(|call| {
            let resolved = resolve_call_path(file, call.receiver.as_deref(), &call.name);
            let lower_resolved = resolved.to_ascii_lowercase();
            if !matches!(
                lower_resolved.as_str(),
                "tempfile.mkdtemp"
                    | "tempfile.mkstemp"
                    | "tempfile.temporarydirectory"
                    | "tempfile.namedtemporaryfile"
            ) {
                return None;
            }

            let line_text = function_line_text(function, call.line)?;
            let lower_line = line_text.trim().to_ascii_lowercase();
            if lower_line.starts_with("with ") {
                return None;
            }
            if lower_resolved == "tempfile.namedtemporaryfile"
                && !lower_line.contains("delete=false")
                && !lower_line.contains("delete = false")
            {
                return None;
            }
            if has_cleanup || temp_resource_cleaned_later(&entries, call.line) {
                return None;
            }

            Some(Finding {
                rule_id: "tempfile_without_cleanup".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} creates a temporary resource without an obvious cleanup path",
                    function.fingerprint.name
                ),
                evidence: vec![format!("temp_resource_call={}", line_text.trim())],
            })
        })
        .collect()
}
