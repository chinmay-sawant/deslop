use std::collections::BTreeSet;

use crate::analysis::ParsedFile;
use crate::model::{Finding, Severity};

use super::{
    OPTION_BAG_FIELD_THRESHOLD, OPTION_BAG_SIGNAL_THRESHOLD, is_client_constructor_text,
    is_config_load_call, is_config_load_text, is_file_io_call, is_network_call_path,
    is_subprocess_call, mutable_default_kind, mutates_binding, option_bag_boolean_field,
    option_bag_optional_field, resolve_top_level_call_path, should_skip_wide_contract_rule,
    wide_contract_markers,
};

pub(super) fn module_state_findings(file: &ParsedFile) -> Vec<Finding> {
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

fn dataclass_mutable_default_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let mut findings = Vec::new();
    for model in file.python_models().iter().filter(|model| model.is_dataclass) {
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

    file.python_models()
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
    for model in file
        .python_models()
        .iter()
        .filter(|model| !model.name.starts_with('_'))
    {
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
        .filter(|binding| {
            !binding
                .name
                .chars()
                .all(|character| character.is_ascii_uppercase() || character == '_')
        })
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
                evidence: vec![format!(
                    "binding={}= {}",
                    binding.name,
                    binding.value_text.trim()
                )],
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
            let mutable_kind = mutable_default_kind(&binding.value_text)?;

            let mutating_functions = file
                .functions
                .iter()
                .filter(|function| {
                    !function.is_test_function && mutates_binding(function, &binding.name)
                })
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
            evidence: vec![format!(
                "binding={}= {}",
                binding.name,
                binding.value_text.trim()
            )],
        });
    }

    findings
}
