use std::collections::BTreeSet;

use crate::analysis::{ParsedFile, ParsedFunction, TopLevelCallSummary};
use crate::model::{Finding, Severity};

const OPTION_BAG_FIELD_THRESHOLD: usize = 6;
const OPTION_BAG_SIGNAL_THRESHOLD: usize = 4;

pub(super) fn advanceplan2_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(untracked_asyncio_task_findings(file, function));
    findings.extend(background_task_exception_unobserved_findings(file, function));
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

pub(super) fn advanceplan2_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(dataclass_mutable_default_findings(file));
    findings.extend(option_bag_model_findings(file));
    findings.extend(public_any_type_leak_model_findings(file));
    findings.extend(import_time_network_call_findings(file));
    findings.extend(import_time_file_io_findings(file));
    findings.extend(import_time_subprocess_findings(file));
    findings.extend(module_singleton_client_side_effect_findings(file));
    findings.extend(mutable_module_global_state_findings(file));
    findings.extend(import_time_config_load_findings(file));
    findings
}

fn untracked_asyncio_task_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !function.is_async {
        return Vec::new();
    }

    let task_groups = task_group_receivers(function);
    body_lines(function)
        .into_iter()
        .filter_map(|(line_no, line)| {
            let trimmed = line.trim();
            let Some(marker) = task_factory_marker(trimmed) else {
                return None;
            };
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
    if function.is_test_function || !function.is_async {
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
    if function.is_test_function || !function.is_async {
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
    if function.is_test_function || !function.is_async {
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

fn mutable_default_argument_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
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
        .python_models
        .iter()
        .find(|model| model.name == model_name && model.is_dataclass)
    else {
        return Vec::new();
    };
    if !model.method_names.iter().any(|method_name| method_name == "__post_init__") {
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
    for model in file.python_models.iter().filter(|model| model.is_typed_dict) {
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

fn tar_extractall_unfiltered_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
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

fn dataclass_mutable_default_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let mut findings = Vec::new();
    for model in file.python_models.iter().filter(|model| model.is_dataclass) {
        for field in &model.fields {
            let Some(default_text) = field.default_text.as_deref() else {
                continue;
            };
            let Some(mutable_kind) = mutable_default_kind(default_text) else {
                continue;
            };
            if default_text.contains("default_factory") {
                continue;
            }

            findings.push(Finding {
                rule_id: "dataclass_mutable_default".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: None,
                start_line: field.line,
                end_line: field.line,
                message: format!(
                    "dataclass {} uses a mutable default for field {} instead of default_factory",
                    model.name, field.name
                ),
                evidence: vec![
                    format!("field={}", field.name),
                    format!("default_expression={default_text}"),
                    format!("mutable_kind={mutable_kind}"),
                ],
            });
        }
    }

    findings
}

fn option_bag_model_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    file.python_models
        .iter()
        .filter(|model| (model.is_dataclass || model.is_typed_dict) && !model.name.starts_with('_'))
        .filter_map(|model| {
            let optional_fields = model
                .fields
                .iter()
                .filter(|field| option_bag_optional_field(field.annotation_text.as_deref(), field.default_text.as_deref()))
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>();
            let boolean_fields = model
                .fields
                .iter()
                .filter(|field| option_bag_boolean_field(field.annotation_text.as_deref(), field.default_text.as_deref()))
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>();

            if model.fields.len() < OPTION_BAG_FIELD_THRESHOLD
                || optional_fields.len() + boolean_fields.len() < OPTION_BAG_SIGNAL_THRESHOLD
                || (optional_fields.len() < 3 && boolean_fields.len() < 2)
            {
                return None;
            }

            let mut evidence = vec![format!("field_count={}", model.fields.len())];
            evidence.push(format!("optional_fields={}", optional_fields.join(",")));
            evidence.push(format!("boolean_fields={}", boolean_fields.join(",")));
            evidence.push(format!("method_count={}", model.method_names.len()));
            if !model.base_classes.is_empty() {
                evidence.push(format!("base_classes={}", model.base_classes.join(",")));
            }
            if !model.decorators.is_empty() {
                evidence.push(format!("decorators={}", model.decorators.join(",")));
            }

            Some(Finding {
                rule_id: "option_bag_model".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: model.line,
                end_line: model.line,
                message: format!(
                    "model {} encodes many optional or boolean switches and looks like an option bag",
                    model.name
                ),
                evidence,
            })
        })
        .collect()
}

fn public_any_type_leak_model_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file || should_skip_wide_contract_rule(file) {
        return Vec::new();
    }

    let mut findings = Vec::new();
    for model in file.python_models.iter().filter(|model| !model.name.starts_with('_')) {
        for field in &model.fields {
            let Some(annotation_text) = field.annotation_text.as_deref() else {
                continue;
            };
            let markers = wide_contract_markers(annotation_text);
            if markers.is_empty() {
                continue;
            }

            findings.push(Finding {
                rule_id: "public_any_type_leak".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: field.line,
                end_line: field.line,
                message: format!(
                    "model {} exposes field {} with a very wide type contract",
                    model.name, field.name
                ),
                evidence: markers
                    .into_iter()
                    .map(|marker| format!("wide_type_marker={marker}"))
                    .collect(),
            });
        }
    }

    findings
}

fn import_time_network_call_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    file.module_scope_calls
        .iter()
        .filter_map(|call| {
            let resolved = resolve_top_level_call_path(file, call);
            if !is_network_call_path(&resolved, &call.name) {
                return None;
            }

            Some(Finding {
                rule_id: "import_time_network_call".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: None,
                start_line: call.line,
                end_line: call.line,
                message: "module performs a network call while being imported".to_string(),
                evidence: vec![format!("module_scope_call={}", call.text.trim())],
            })
        })
        .collect()
}

fn import_time_file_io_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    file.module_scope_calls
        .iter()
        .filter_map(|call| {
            let resolved = resolve_top_level_call_path(file, call);
            if !is_file_io_call(call, &resolved) {
                return None;
            }

            Some(Finding {
                rule_id: "import_time_file_io".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: call.line,
                end_line: call.line,
                message: "module performs filesystem work while being imported".to_string(),
                evidence: vec![format!("module_scope_call={}", call.text.trim())],
            })
        })
        .collect()
}

fn import_time_subprocess_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    file.module_scope_calls
        .iter()
        .filter_map(|call| {
            let resolved = resolve_top_level_call_path(file, call);
            if !is_subprocess_call(&resolved) {
                return None;
            }

            Some(Finding {
                rule_id: "import_time_subprocess".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: None,
                start_line: call.line,
                end_line: call.line,
                message: "module launches a subprocess while being imported".to_string(),
                evidence: vec![format!("module_scope_call={}", call.text.trim())],
            })
        })
        .collect()
}

fn module_singleton_client_side_effect_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    file.top_level_bindings
        .iter()
        .filter(|binding| !binding.name.chars().all(|character| character.is_ascii_uppercase() || character == '_'))
        .filter_map(|binding| {
            let lower_value = binding.value_text.to_ascii_lowercase();
            if !is_client_constructor_text(&lower_value) {
                return None;
            }

            Some(Finding {
                rule_id: "module_singleton_client_side_effect".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: binding.line,
                end_line: binding.line,
                message: format!(
                    "module eagerly constructs client-like global {} at import time",
                    binding.name
                ),
                evidence: vec![format!("binding={}= {}", binding.name, binding.value_text.trim())],
            })
        })
        .collect()
}

fn mutable_module_global_state_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    file.top_level_bindings
        .iter()
        .filter_map(|binding| {
            let Some(mutable_kind) = mutable_default_kind(&binding.value_text) else {
                return None;
            };

            let mutating_functions = file
                .functions
                .iter()
                .filter(|function| !function.is_test_function && mutates_binding(function, &binding.name))
                .map(|function| function.fingerprint.name.as_str())
                .collect::<BTreeSet<_>>();
            if mutating_functions.len() < 2 {
                return None;
            }

            Some(Finding {
                rule_id: "mutable_module_global_state".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: binding.line,
                end_line: binding.line,
                message: format!(
                    "module-level mutable global {} is updated from multiple functions",
                    binding.name
                ),
                evidence: vec![
                    format!("mutable_kind={mutable_kind}"),
                    format!(
                        "mutating_functions={}",
                        mutating_functions.into_iter().collect::<Vec<_>>().join(",")
                    ),
                ],
            })
        })
        .collect()
}

fn import_time_config_load_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let mut findings = Vec::new();
    for call in &file.module_scope_calls {
        let resolved = resolve_top_level_call_path(file, call).to_ascii_lowercase();
        let lower_text = call.text.to_ascii_lowercase();
        if !is_config_load_call(&resolved, &lower_text) {
            continue;
        }

        findings.push(Finding {
            rule_id: "import_time_config_load".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: call.line,
            end_line: call.line,
            message: "module loads configuration or secrets while being imported".to_string(),
            evidence: vec![format!("module_scope_call={}", call.text.trim())],
        });
    }

    for binding in &file.top_level_bindings {
        let lower_value = binding.value_text.to_ascii_lowercase();
        if !is_config_load_text(&lower_value) {
            continue;
        }

        findings.push(Finding {
            rule_id: "import_time_config_load".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: binding.line,
            end_line: binding.line,
            message: format!(
                "module initializes {} from configuration or secrets at import time",
                binding.name
            ),
            evidence: vec![format!("binding={}= {}", binding.name, binding.value_text.trim())],
        });
    }

    findings
}

fn body_lines(function: &ParsedFunction) -> Vec<(usize, &str)> {
    function
        .body_text
        .lines()
        .enumerate()
        .map(|(index, line)| (function.body_start_line + index, line))
        .collect()
}

fn function_line_text(function: &ParsedFunction, line_no: usize) -> Option<&str> {
    function
        .body_text
        .lines()
        .nth(line_no.checked_sub(function.body_start_line)?)
}

fn task_group_receivers(function: &ParsedFunction) -> BTreeSet<String> {
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

fn split_assignment(text: &str) -> Option<(&str, &str)> {
    if text.contains("==") {
        return None;
    }
    text.split_once('=')
}

fn task_factory_marker(text: &str) -> Option<&'static str> {
    [
        "asyncio.create_task(",
        "create_task(",
        "asyncio.ensure_future(",
        "ensure_future(",
    ]
    .into_iter()
    .find(|marker| text.contains(marker))
}

fn contains_task_factory(text: &str) -> bool {
    task_factory_marker(text.trim()).is_some() || text.contains(".create_task(")
}

fn task_group_create_task(text: &str, task_groups: &BTreeSet<String>) -> bool {
    let trimmed = text.trim();
    let Some((receiver, _)) = trimmed.split_once(".create_task(") else {
        return false;
    };
    task_groups.contains(receiver.trim())
}

fn task_handle_observed(function: &ParsedFunction, task_name: &str, created_line: usize) -> bool {
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

fn looks_like_lock_context(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("lock") || lower.contains("mutex") || lower.contains("semaphore")
}

fn indented_block<'a>(entries: &'a [(usize, &'a str)], header_index: usize) -> Vec<(usize, &'a str)> {
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
    line.chars().take_while(|character| character.is_ascii_whitespace()).count()
}

fn explicit_lock_acquire_name(line: &str) -> Option<String> {
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

fn is_unrelated_await_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("await ")
        && !trimmed.contains(".acquire(")
        && !trimmed.contains(".release(")
}

fn constant_async_sleep_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with("await ") || (!trimmed.contains("asyncio.sleep(") && !trimmed.contains("sleep(")) {
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

fn has_retry_backoff_markers(text: &str) -> bool {
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

fn parameter_entries(signature_text: &str) -> Vec<String> {
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
                && brace_depth == 0 => {
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

fn mutable_default_kind(default_text: &str) -> Option<&'static str> {
    let normalized = default_text.trim();
    if normalized.starts_with('[') && normalized.ends_with(']') {
        return Some("list");
    }
    if normalized.starts_with('{') && normalized.ends_with('}') {
        return Some(if normalized.contains(':') { "dict" } else { "set" });
    }
    matches!(normalized, "list()" | "dict()" | "set()" | "defaultdict()")
        .then_some(if normalized.starts_with("set") {
            "set"
        } else if normalized.starts_with("dict") || normalized.starts_with("defaultdict") {
            "dict"
        } else {
            "list"
        })
}

fn heavy_post_init_detail(file: &ParsedFile, function: &ParsedFunction) -> Option<(usize, String)> {
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
            (lower.contains("subprocess.") || lower.contains(" open(") || lower.starts_with("open("))
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

fn wide_contract_markers(text: &str) -> Vec<String> {
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

fn should_skip_wide_contract_rule(file: &ParsedFile) -> bool {
    file.path.components().any(|component| {
        let part = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        matches!(part.as_str(), "migrations" | "migration" | "serializers" | "serializer")
    })
}

fn typed_dict_field_is_optional(annotation_text: Option<&str>) -> bool {
    annotation_text.is_some_and(|annotation| {
        let lower = annotation.to_ascii_lowercase();
        lower.contains("notrequired[") || lower.contains("optional[") || lower.contains("| none")
    })
}

fn key_guarded_before(entries: &[(usize, &str)], line_no: usize, key: &str) -> bool {
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

fn resolve_call_path(file: &ParsedFile, receiver: Option<&str>, name: &str) -> String {
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

fn resolve_top_level_call_path(file: &ParsedFile, call: &TopLevelCallSummary) -> String {
    resolve_call_path(file, call.receiver.as_deref(), &call.name)
}

fn nearby_tar_guard(entries: &[(usize, &str)], line_no: usize) -> bool {
    entries
        .iter()
        .filter(|(entry_line, _)| *entry_line <= line_no && line_no.saturating_sub(*entry_line) <= 3)
        .any(|(_, line)| {
            let lower = line.trim().to_ascii_lowercase();
            lower.contains("validate")
                || lower.contains("safe_extract")
                || lower.contains("is_within_directory")
        })
}

fn is_tempfile_cleanup_line(line: &str) -> bool {
    line.contains(".cleanup(")
        || line.contains("unlink(")
        || line.contains("remove(")
        || line.contains("rmtree(")
}

fn temp_resource_cleaned_later(entries: &[(usize, &str)], line_no: usize) -> bool {
    entries
        .iter()
        .filter(|(entry_line, _)| *entry_line > line_no)
        .any(|(_, line)| is_tempfile_cleanup_line(&line.to_ascii_lowercase()))
}

fn option_bag_optional_field(annotation_text: Option<&str>, default_text: Option<&str>) -> bool {
    annotation_text.is_some_and(|annotation| {
        let lower = annotation.to_ascii_lowercase();
        lower.contains("optional[") || lower.contains("| none") || lower.contains("notrequired[")
    }) || default_text.is_some_and(|default| default.trim() == "None")
}

fn option_bag_boolean_field(annotation_text: Option<&str>, default_text: Option<&str>) -> bool {
    annotation_text.is_some_and(|annotation| annotation.to_ascii_lowercase().contains("bool"))
        || default_text.is_some_and(|default| matches!(default.trim(), "True" | "False"))
}

fn is_network_call_path(resolved_path: &str, call_name: &str) -> bool {
    let lower = resolved_path.to_ascii_lowercase();
    let lower_name = call_name.to_ascii_lowercase();
    (lower.starts_with("requests.") || lower.starts_with("httpx.") || lower.starts_with("urllib.") || lower.starts_with("socket."))
        && matches!(
            lower_name.as_str(),
            "get" | "post" | "put" | "patch" | "delete" | "request" | "urlopen" | "connect" | "create_connection"
        )
}

fn is_file_io_call(call: &TopLevelCallSummary, resolved_path: &str) -> bool {
    let lower = resolved_path.to_ascii_lowercase();
    matches!(
        call.name.as_str(),
        "open" | "read_text" | "read_bytes" | "write_text" | "write_bytes" | "listdir" | "scandir" | "walk" | "glob" | "iglob" | "rglob"
    ) || lower == "open"
        || lower.ends_with(".read_text")
        || lower.ends_with(".read_bytes")
        || lower.ends_with(".write_text")
        || lower.ends_with(".write_bytes")
        || lower.ends_with(".listdir")
        || lower.ends_with(".scandir")
        || lower.ends_with(".walk")
        || lower.contains("path(")
            && matches!(call.name.as_str(), "read_text" | "read_bytes" | "write_text" | "write_bytes")
}

fn is_subprocess_call(resolved_path: &str) -> bool {
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

fn is_client_constructor_text(lower_value: &str) -> bool {
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

fn mutates_binding(function: &ParsedFunction, binding_name: &str) -> bool {
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
            || (line.starts_with(&format!("{binding_name} =")) && function.body_text.contains(&global_marker))
    })
}

fn is_config_load_call(resolved_path: &str, lower_text: &str) -> bool {
    matches!(resolved_path, "os.getenv" | "os.environ.get" | "dotenv.load_dotenv" | "dotenv.dotenv_values")
        || is_config_load_text(lower_text)
}

fn is_config_load_text(lower_text: &str) -> bool {
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

fn is_valid_identifier(candidate: &str) -> bool {
    let mut characters = candidate.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }

    characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}