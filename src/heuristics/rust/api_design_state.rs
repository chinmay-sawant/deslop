use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::*;

pub(super) fn shared_state_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for summary in file.structs() {
        let public_exposure =
            summary.visibility_pub || summary.fields.iter().any(|field| field.is_pub);

        for field in &summary.fields {
            let normalized = normalized_type(&field.type_text);

            if field.is_pub && contains_interior_mutability(&normalized) {
                findings.push(file_finding(
                    file,
                    "rust_pub_interior_mutability_field",
                    Severity::Warning,
                    field.line,
                    format!(
                        "struct {} exposes an interior-mutable field directly",
                        summary.name
                    ),
                    vec![
                        format!("field={}", field.name),
                        format!("type={}", field.type_text),
                    ],
                ));
            }

            if is_arc_mutex_option_type(&normalized) {
                findings.push(file_finding(
                    file,
                    "rust_arc_mutex_option_state",
                    if public_exposure {
                        Severity::Warning
                    } else {
                        Severity::Info
                    },
                    field.line,
                    format!(
                        "struct {} hides lifecycle state behind Arc<...Mutex<Option<_>>> layers",
                        summary.name
                    ),
                    vec![format!("field={} type={}", field.name, field.type_text)],
                ));
            }

            if is_mutex_collection_type(&normalized)
                && (public_exposure || central_state_name(&summary.name))
            {
                findings.push(file_finding(
                    file,
                    "rust_mutex_wrapped_collection",
                    if public_exposure {
                        Severity::Warning
                    } else {
                        Severity::Info
                    },
                    field.line,
                    format!(
                        "struct {} embeds a collection directly inside a lock wrapper",
                        summary.name
                    ),
                    vec![format!("field={} type={}", field.name, field.type_text)],
                ));
            }

            if is_rc_refcell_type(&normalized) && !graph_or_ui_shape(summary) {
                findings.push(file_finding(
                    file,
                    "rust_rc_refcell_domain_model",
                    struct_severity(summary),
                    field.line,
                    format!(
                        "struct {} uses Rc<RefCell<_>> in a domain-style type",
                        summary.name
                    ),
                    vec![format!("field={} type={}", field.name, field.type_text)],
                ));
            }
        }
    }

    for static_summary in file.rust_statics() {
        let normalized = normalized_type(&static_summary.type_text);
        if contains_global_lock_state(&normalized) {
            findings.push(file_finding(
                file,
                "rust_global_lock_state",
                Severity::Warning,
                static_summary.line,
                format!(
                    "static {} wraps mutable shared state in a global lock shape",
                    static_summary.name
                ),
                vec![
                    format!("type={}", static_summary.type_text),
                    format!("visibility_pub={}", static_summary.visibility_pub),
                    static_summary
                        .value_text
                        .as_ref()
                        .map(|value| format!("value={value}"))
                        .unwrap_or_else(|| "value=<unknown>".to_string()),
                ],
            ));
        }

        if is_arc_mutex_option_type(&normalized) {
            findings.push(file_finding(
                file,
                "rust_arc_mutex_option_state",
                Severity::Warning,
                static_summary.line,
                format!(
                    "static {} hides lifecycle state behind Arc<...Mutex<Option<_>>> layers",
                    static_summary.name
                ),
                vec![format!("type={}", static_summary.type_text)],
            ));
        }
    }

    findings
}

pub(super) fn serde_contract_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for summary in file.rust_enums() {
        if summary.variant_count >= 2
            && attribute_has(&summary.attributes, "serde(")
            && attribute_has(&summary.attributes, "untagged")
            && (summary.visibility_pub
                || summary.has_deserialize_derive
                || summary.has_serialize_derive)
        {
            findings.push(file_finding(
                file,
                "rust_serde_untagged_enum_boundary",
                Severity::Warning,
                summary.line,
                format!(
                    "enum {} derives #[serde(untagged)] on a boundary-facing type",
                    summary.name
                ),
                vec![
                    format!("variant_count={}", summary.variant_count),
                    format!("derives={}", summary.derives.join(",")),
                ],
            ));
        }
    }

    for summary in file.structs() {
        if summary.has_deserialize_derive {
            if strict_contract_name(&summary.name)
                && !attribute_has(&summary.attributes, "deny_unknown_fields")
            {
                findings.push(file_finding(
                    file,
                    "rust_serde_unknown_fields_allowed",
                    struct_severity(summary),
                    summary.line,
                    format!(
                        "struct {} deserializes a strict-looking contract without deny_unknown_fields",
                        summary.name
                    ),
                    vec![format!("derives={}", summary.derives.join(","))],
                ));
            }

            for field in &summary.fields {
                if attribute_has(&field.attributes, "serde(")
                    && attribute_has(&field.attributes, "default")
                    && !field.is_option
                    && required_like_field(summary, field)
                {
                    findings.push(file_finding(
                        file,
                        "rust_serde_default_on_required_field",
                        struct_severity(summary),
                        field.line,
                        format!(
                            "struct {} defaults field {} even though the field looks required",
                            summary.name, field.name
                        ),
                        vec![format!("field_type={}", field.type_text)],
                    ));
                }

                if attribute_has(&field.attributes, "serde(")
                    && attribute_has(&field.attributes, "flatten")
                    && flatten_catchall_type(&field.type_text)
                {
                    findings.push(file_finding(
                        file,
                        "rust_serde_flatten_catchall",
                        struct_severity(summary),
                        field.line,
                        format!(
                            "struct {} flattens unknown fields into a catch-all map-like field",
                            summary.name
                        ),
                        vec![format!("field={} type={}", field.name, field.type_text)],
                    ));
                }
            }
        }

        if (summary.has_deserialize_derive || summary.has_serialize_derive)
            && (strict_contract_name(&summary.name) || summary.visibility_pub)
        {
            for field in &summary.fields {
                if normalized_type(&field.type_text) == "String"
                    && enum_like_string_field(&field.name)
                {
                    findings.push(file_finding(
                        file,
                        "rust_stringly_typed_enum_boundary",
                        struct_severity(summary),
                        field.line,
                        format!(
                            "struct {} models enum-like boundary field {} as String",
                            summary.name, field.name
                        ),
                        vec![format!("field_type={}", field.type_text)],
                    ));
                }
            }
        }
    }

    findings
}

pub(super) fn builder_state_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for summary in file.structs() {
        let option_fields = summary
            .fields
            .iter()
            .filter(|field| field.is_option)
            .count();
        let bool_fields = summary.fields.iter().filter(|field| field.is_bool).count();

        if config_like_name(&summary.name)
            && summary.fields.len() >= 6
            && option_fields >= OPTION_BAG_THRESHOLD
            && !has_validation_method(file, &summary.name)
        {
            findings.push(file_finding(
                file,
                "rust_option_bag_config",
                struct_severity(summary),
                summary.line,
                format!(
                    "struct {} collects many Option fields without an obvious validation path",
                    summary.name
                ),
                vec![
                    format!("field_count={}", summary.fields.len()),
                    format!("option_fields={option_fields}"),
                ],
            ));
        }

        if summary.name.ends_with("Builder")
            && let Some(build_method) = file.functions.iter().find(|function| {
                function.fingerprint.receiver_type.as_deref() == Some(summary.name.as_str())
                    && function.fingerprint.name == "build"
                    && is_public_api(function)
            })
            && !has_validation_method(file, &summary.name)
            && !body_has_validation_markers(&build_method.body_text)
        {
            findings.push(file_finding(
                file,
                "rust_builder_without_validate",
                struct_severity(summary),
                build_method.fingerprint.start_line,
                format!(
                    "builder {} exposes build() without an obvious validation step",
                    summary.name
                ),
                vec![format!("build_method={}", build_method.fingerprint.name)],
            ));
        }

        if bool_fields >= 2
            && (state_like_name(&summary.name)
                || summary.fields.iter().any(|field| field.is_option))
        {
            findings.push(file_finding(
                file,
                "rust_boolean_state_machine",
                struct_severity(summary),
                summary.line,
                format!(
                    "struct {} encodes state through multiple booleans instead of a dedicated enum",
                    summary.name
                ),
                vec![
                    format!("bool_fields={bool_fields}"),
                    format!(
                        "bool_names={}",
                        summary
                            .fields
                            .iter()
                            .filter(|field| field.is_bool)
                            .map(|field| field.name.as_str())
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                ],
            ));
        }
    }

    findings
}

pub(super) fn builder_state_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if !is_public_api(function) {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let bool_params = parameter_entries(&function.signature_text)
        .into_iter()
        .filter_map(|entry| parameter_name_and_type(&entry))
        .filter(|(_, type_text)| normalized_type(type_text) == "bool")
        .map(|(name, _)| name)
        .collect::<Vec<_>>();

    if constructor_like_name(&function.fingerprint.name) && bool_params.len() >= 2 {
        findings.push(function_finding(
            file,
            function,
            "rust_constructor_many_flags",
            Severity::Warning,
            function.fingerprint.start_line,
            format!(
                "constructor-like API {} uses multiple boolean flags",
                function.fingerprint.name
            ),
            vec![format!("bool_parameters={}", bool_params.join(","))],
        ));
    }

    if constructor_like_name(&function.fingerprint.name)
        && body_shows_partial_init_escape(&function.body_text)
    {
        findings.push(function_finding(
            file,
            function,
            "rust_partial_init_escape",
            Severity::Info,
            function.fingerprint.start_line,
            format!(
                "function {} returns or stores a partially initialized struct shape",
                function.fingerprint.name
            ),
            vec!["body_contains=None_or_Default::default() inside a struct literal".to_string()],
        ));
    }

    findings
}
