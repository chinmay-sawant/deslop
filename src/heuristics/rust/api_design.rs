use crate::analysis::{FieldSummary, ParsedFile, ParsedFunction, StructSummary};
use crate::model::{Finding, Severity};

use super::{file_finding, function_finding, matches_token, struct_severity};

const OPTION_BAG_THRESHOLD: usize = 4;

pub(crate) fn api_design_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let mut findings = Vec::new();
    findings.extend(shared_state_findings(file));
    findings.extend(serde_contract_findings(file));
    findings.extend(builder_state_file_findings(file));
    findings
}

pub(crate) fn api_design_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let mut findings = Vec::new();
    findings.extend(api_surface_findings(file, function));
    findings.extend(builder_state_function_findings(file, function));
    findings
}

fn api_surface_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !is_public_api(function) {
        return Vec::new();
    }

    let mut findings = Vec::new();

    if is_library_like(file)
        && let Some(return_type) = return_type_text(&function.signature_text)
    {
        if return_type_uses_anyhow_like_result(file, &return_type) {
            findings.push(function_finding(
                file,
                function,
                "rust_public_anyhow_result",
                Severity::Warning,
                function.fingerprint.start_line,
                format!(
                    "public API {} returns an application-edge anyhow-style Result",
                    function.fingerprint.name
                ),
                vec![format!("return_type={return_type}")],
            ));
        }

        if return_type_uses_box_dyn_error(file, &return_type) {
            findings.push(function_finding(
                file,
                function,
                "rust_public_box_dyn_error",
                Severity::Warning,
                function.fingerprint.start_line,
                format!(
                    "public API {} exposes Box<dyn Error> instead of a clearer error surface",
                    function.fingerprint.name
                ),
                vec![format!("return_type={return_type}")],
            ));
        }
    }

    let parameters = parameter_entries(&function.signature_text);
    let bool_params = parameters
        .iter()
        .filter_map(|entry| parameter_name_and_type(entry))
        .filter(|(_, type_text)| normalized_type(type_text) == "bool")
        .collect::<Vec<_>>();

    if bool_params.len() == 1 && !builder_internal(function) {
        let (param_name, _) = &bool_params[0];
        findings.push(function_finding(
            file,
            function,
            "rust_public_bool_parameter_api",
            Severity::Info,
            function.fingerprint.start_line,
            format!(
                "public API {} exposes a raw boolean mode switch",
                function.fingerprint.name
            ),
            vec![format!("bool_parameter={param_name}")],
        ));
    }

    if !builder_internal(function) {
        for (param_name, type_text) in parameters
            .iter()
            .filter_map(|entry| parameter_name_and_type(entry))
        {
            if is_borrowed_string_type(&type_text) {
                findings.push(function_finding(
                    file,
                    function,
                    "rust_borrowed_string_api",
                    Severity::Info,
                    function.fingerprint.start_line,
                    format!(
                        "public API {} borrows String where &str would be more general",
                        function.fingerprint.name
                    ),
                    vec![
                        format!("parameter={param_name}"),
                        format!("type={type_text}"),
                    ],
                ));
            }

            if is_borrowed_vec_type(&type_text) {
                findings.push(function_finding(
                    file,
                    function,
                    "rust_borrowed_vec_api",
                    Severity::Info,
                    function.fingerprint.start_line,
                    format!(
                        "public API {} borrows Vec directly where a slice would be more flexible",
                        function.fingerprint.name
                    ),
                    vec![
                        format!("parameter={param_name}"),
                        format!("type={type_text}"),
                    ],
                ));
            }

            if is_borrowed_pathbuf_type(file, &type_text) {
                findings.push(function_finding(
                    file,
                    function,
                    "rust_borrowed_pathbuf_api",
                    Severity::Info,
                    function.fingerprint.start_line,
                    format!(
                        "public API {} borrows PathBuf where &Path would better match the contract",
                        function.fingerprint.name
                    ),
                    vec![
                        format!("parameter={param_name}"),
                        format!("type={type_text}"),
                    ],
                ));
            }
        }
    }

    findings
}

fn shared_state_findings(file: &ParsedFile) -> Vec<Finding> {
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

fn serde_contract_findings(file: &ParsedFile) -> Vec<Finding> {
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

fn builder_state_file_findings(file: &ParsedFile) -> Vec<Finding> {
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

fn builder_state_function_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
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

fn is_public_api(function: &ParsedFunction) -> bool {
    let signature = function.signature_text.trim_start();
    signature.starts_with("pub ") || signature.starts_with("pub(")
}

fn is_library_like(file: &ParsedFile) -> bool {
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    !path.ends_with("main.rs") && !path.contains("/bin/")
}

fn builder_internal(function: &ParsedFunction) -> bool {
    function
        .fingerprint
        .receiver_type
        .as_deref()
        .is_some_and(|receiver| receiver.ends_with("Builder"))
        || function.fingerprint.name.starts_with("with_")
        || function.fingerprint.name.starts_with("set_")
}

fn return_type_text(signature_text: &str) -> Option<String> {
    let header = signature_text.replace('\n', " ");
    let (_, return_text) = header.split_once("->")?;
    let return_text = return_text.trim();
    let return_text = return_text
        .split(" where ")
        .next()
        .unwrap_or(return_text)
        .trim();
    (!return_text.is_empty()).then(|| return_text.to_string())
}

fn parameter_entries(signature_text: &str) -> Vec<String> {
    let Some(start) = signature_text.find('(') else {
        return Vec::new();
    };
    let mut depth = 0usize;
    let mut end = None;
    for (offset, character) in signature_text[start..].char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = Some(start + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(end) = end else {
        return Vec::new();
    };
    split_top_level_commas(&signature_text[start + 1..end])
}

fn split_top_level_commas(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_single_quote = false;

    for character in text.chars() {
        match character {
            '\'' => in_single_quote = !in_single_quote,
            '<' if !in_single_quote => angle_depth += 1,
            '>' if !in_single_quote => angle_depth = angle_depth.saturating_sub(1),
            '(' if !in_single_quote => paren_depth += 1,
            ')' if !in_single_quote => paren_depth = paren_depth.saturating_sub(1),
            '[' if !in_single_quote => bracket_depth += 1,
            ']' if !in_single_quote => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if !in_single_quote
                && angle_depth == 0
                && paren_depth == 0
                && bracket_depth == 0 =>
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
        current.push(character);
    }

    let tail = current.trim();
    if !tail.is_empty() {
        parts.push(tail.to_string());
    }

    parts
}

fn parameter_name_and_type(entry: &str) -> Option<(String, String)> {
    let trimmed = entry.trim();
    if trimmed.is_empty() || trimmed.contains("self") {
        return None;
    }
    let (name, type_text) = trimmed.split_once(':')?;
    let name = name
        .trim()
        .trim_start_matches("mut ")
        .trim_start_matches("pub ")
        .to_string();
    let type_text = type_text.trim().to_string();
    (!name.is_empty() && !type_text.is_empty()).then_some((name, type_text))
}

fn normalized_type(type_text: &str) -> String {
    type_text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn import_alias_for<'a>(file: &'a ParsedFile, path_suffix: &str) -> Vec<&'a str> {
    file.imports
        .iter()
        .filter(|import| import.path.ends_with(path_suffix))
        .map(|import| import.alias.as_str())
        .collect()
}

fn return_type_uses_anyhow_like_result(file: &ParsedFile, return_type: &str) -> bool {
    let normalized = normalized_type(return_type).to_ascii_lowercase();
    if normalized.contains("anyhow::result<")
        || normalized.contains("eyre::result<")
        || normalized.contains("color_eyre::result<")
        || normalized.contains("anyhow::error")
        || normalized.contains("eyre::report")
    {
        return true;
    }

    import_alias_for(file, "::Result").into_iter().any(|alias| {
        let normalized_alias = alias.to_ascii_lowercase();
        normalized.starts_with(&format!("{normalized_alias}<"))
            && file.imports.iter().any(|import| {
                import.alias == alias
                    && (import.path.starts_with("anyhow::")
                        || import.path.starts_with("eyre::")
                        || import.path.starts_with("color_eyre::"))
            })
    })
}

fn return_type_uses_box_dyn_error(file: &ParsedFile, return_type: &str) -> bool {
    let normalized = normalized_type(return_type);
    if normalized.contains("Box<dynstd::error::Error")
        || normalized.contains("Box<dyncore::error::Error")
    {
        return true;
    }

    import_alias_for(file, "::Error").into_iter().any(|alias| {
        normalized.contains(&format!("Box<dyn{alias}"))
            && file.imports.iter().any(|import| {
                import.alias == alias
                    && (import.path.ends_with("std::error::Error")
                        || import.path.ends_with("core::error::Error"))
            })
    })
}

fn is_borrowed_string_type(type_text: &str) -> bool {
    matches!(
        normalized_type(type_text).as_str(),
        "&String" | "&std::string::String"
    )
}

fn is_borrowed_vec_type(type_text: &str) -> bool {
    let normalized = normalized_type(type_text);
    normalized.starts_with("&Vec<") || normalized.starts_with("&std::vec::Vec<")
}

fn is_borrowed_pathbuf_type(file: &ParsedFile, type_text: &str) -> bool {
    let normalized = normalized_type(type_text);
    if normalized.starts_with("&PathBuf") || normalized.starts_with("&std::path::PathBuf") {
        return true;
    }

    import_alias_for(file, "std::path::PathBuf")
        .into_iter()
        .any(|alias| normalized.starts_with(&format!("&{alias}")))
}

fn contains_interior_mutability(type_text: &str) -> bool {
    [
        "Mutex<",
        "RwLock<",
        "RefCell<",
        "Cell<",
        "OnceCell<",
        "LazyCell<",
    ]
    .iter()
    .any(|needle| type_text.contains(needle))
}

fn contains_global_lock_state(type_text: &str) -> bool {
    contains_interior_mutability(type_text)
        || ((type_text.contains("OnceLock<")
            || type_text.contains("Lazy<")
            || type_text.contains("LazyLock<"))
            && contains_interior_mutability(type_text))
}

fn is_arc_mutex_option_type(type_text: &str) -> bool {
    type_text.contains("Arc<")
        && (type_text.contains("Mutex<Option<") || type_text.contains("RwLock<Option<"))
}

fn is_mutex_collection_type(type_text: &str) -> bool {
    (type_text.contains("Mutex<") || type_text.contains("RwLock<"))
        && ["Vec<", "HashMap<", "BTreeMap<", "HashSet<", "VecDeque<"]
            .iter()
            .any(|needle| type_text.contains(needle))
}

fn is_rc_refcell_type(type_text: &str) -> bool {
    type_text.contains("Rc<RefCell<") || type_text.contains("Rc<std::cell::RefCell<")
}

fn graph_or_ui_shape(summary: &StructSummary) -> bool {
    matches_token(
        &summary.name,
        &["node", "tree", "graph", "widget", "ui", "scene", "dom"],
    )
}

fn central_state_name(name: &str) -> bool {
    matches_token(name, &["state", "store", "cache", "manager", "registry"])
}

fn attribute_has(attributes: &[String], needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    attributes
        .iter()
        .any(|attribute| attribute.to_ascii_lowercase().contains(&needle))
}

fn strict_contract_name(name: &str) -> bool {
    matches_token(
        name,
        &["config", "settings", "request", "options", "params"],
    )
}

fn required_like_field(summary: &StructSummary, field: &FieldSummary) -> bool {
    strict_contract_name(&summary.name)
        || matches_token(
            &field.name,
            &[
                "id", "kind", "type", "mode", "status", "endpoint", "host", "path", "url", "method",
            ],
        )
}

fn flatten_catchall_type(type_text: &str) -> bool {
    let normalized = normalized_type(type_text);
    normalized.contains("HashMap<")
        || normalized.contains("BTreeMap<")
        || normalized.contains("serde_json::Value")
        || normalized.contains("Map<")
}

fn enum_like_string_field(name: &str) -> bool {
    matches_token(
        name,
        &["kind", "type", "status", "state", "mode", "role", "level"],
    )
}

fn config_like_name(name: &str) -> bool {
    matches_token(
        name,
        &["config", "options", "request", "settings", "params"],
    )
}

fn has_validation_method(file: &ParsedFile, type_name: &str) -> bool {
    file.functions.iter().any(|function| {
        function.fingerprint.receiver_type.as_deref() == Some(type_name)
            && matches!(
                function.fingerprint.name.as_str(),
                "validate" | "check" | "ensure_valid" | "build"
            )
            && body_has_validation_markers(&function.body_text)
    })
}

fn body_has_validation_markers(body_text: &str) -> bool {
    [
        "validate(",
        "ensure!",
        "ok_or",
        "ok_or_else",
        "is_none()",
        "Err(",
        "bail!",
        "missing",
    ]
    .iter()
    .any(|marker| body_text.contains(marker))
}

fn state_like_name(name: &str) -> bool {
    matches_token(
        name,
        &[
            "state",
            "status",
            "session",
            "connection",
            "job",
            "task",
            "process",
        ],
    )
}

fn constructor_like_name(name: &str) -> bool {
    name == "new"
        || name.starts_with("new_")
        || name.starts_with("create")
        || name.starts_with("build")
        || name.starts_with("finish")
        || name.starts_with("from_")
}

fn body_shows_partial_init_escape(body_text: &str) -> bool {
    (body_text.contains("None") && body_text.contains('{') && body_text.contains(':'))
        || body_text.contains("..Default::default()")
}
