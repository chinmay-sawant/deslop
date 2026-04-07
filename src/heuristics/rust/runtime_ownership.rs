use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::function_finding;
use super::{contains_any, first_line_with_any, is_scanner_infra_file, is_test_like};

pub(crate) const RULE_DEFINITIONS: &[crate::rules::catalog::RuleDefinition] = &[
    crate::rules::catalog::RuleDefinition {
        id: "rust_detached_spawn_without_handle",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "runtime_ownership",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Spawned background work whose JoinHandle is immediately discarded or never supervised.",
        binding_location: crate::rules::catalog::bindings::RUST_RUNTIME_OWNERSHIP,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_channel_created_per_request",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "runtime_ownership",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Per-request channel and notification state creation instead of startup-owned coordination.",
        binding_location: crate::rules::catalog::bindings::RUST_RUNTIME_OWNERSHIP,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_block_in_place_request_path",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "runtime_ownership",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Blocking runtime bridges such as block_in_place or block_on inside request-handling code.",
        binding_location: crate::rules::catalog::bindings::RUST_RUNTIME_OWNERSHIP,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_runtime_builder_in_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "runtime_ownership",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Repeated runtime or executor builder setup inside loops or retry bodies.",
        binding_location: crate::rules::catalog::bindings::RUST_RUNTIME_OWNERSHIP,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_notify_without_shutdown_contract",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "runtime_ownership",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Info,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Notify/wait coordination that lacks any visible shutdown or cancellation branch.",
        binding_location: crate::rules::catalog::bindings::RUST_RUNTIME_OWNERSHIP,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_process_global_env_toggle",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "runtime_ownership",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "Process-global environment mutation used as runtime control flow.",
        binding_location: crate::rules::catalog::bindings::RUST_RUNTIME_OWNERSHIP,
    },
];

pub(crate) fn runtime_ownership_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if is_test_like(file, Some(function)) || is_scanner_infra_file(file) {
        return Vec::new();
    }

    let mut findings = Vec::new();
    findings.extend(detached_spawn_findings(file, function));
    findings.extend(channel_creation_findings(file, function));
    findings.extend(blocking_bridge_findings(file, function));
    findings.extend(runtime_builder_loop_findings(file, function));
    findings.extend(notify_shutdown_findings(file, function));
    findings.extend(process_env_toggle_findings(file, function));
    findings
}

fn detached_spawn_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let markers = ["tokio::spawn(", "task::spawn("];
    for (offset, line) in function.body_text.lines().enumerate() {
        if !contains_any(line, &markers) {
            continue;
        }

        let trimmed = line.trim();
        let discarded = trimmed.starts_with("tokio::spawn(")
            || trimmed.starts_with("task::spawn(")
            || line.contains("let _ =")
            || line.contains("drop(tokio::spawn(")
            || line.contains("drop(task::spawn(");
        if !discarded || line.contains(".await") {
            continue;
        }

        return vec![function_finding(
            file,
            function,
            "rust_detached_spawn_without_handle",
            Severity::Warning,
            function.fingerprint.start_line + offset,
            format!(
                "function {} spawns background work without an obvious JoinHandle owner",
                function.fingerprint.name
            ),
            vec![trimmed.to_string()],
        )];
    }

    Vec::new()
}

fn channel_creation_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !request_path_like(file, function) {
        return Vec::new();
    }

    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &[
            "mpsc::channel(",
            "mpsc::channel::<",
            "broadcast::channel(",
            "broadcast::channel::<",
            "watch::channel(",
            "watch::channel::<",
            "oneshot::channel(",
            "oneshot::channel::<",
            "tokio::sync::mpsc::channel(",
            "tokio::sync::mpsc::channel::<",
        ],
    ) else {
        return Vec::new();
    };

    vec![function_finding(
        file,
        function,
        "rust_channel_created_per_request",
        Severity::Warning,
        line,
        format!(
            "function {} creates channel state on a request path",
            function.fingerprint.name
        ),
        vec![
            "channel creation on a hot path usually belongs at startup".to_string(),
            "prefer startup-owned senders or injected coordination state when possible".to_string(),
        ],
    )]
}

fn blocking_bridge_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !request_path_like(file, function) {
        return Vec::new();
    }

    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &["block_in_place(", "block_on("],
    ) else {
        return Vec::new();
    };

    vec![function_finding(
        file,
        function,
        "rust_block_in_place_request_path",
        Severity::Warning,
        line,
        format!(
            "function {} bridges into blocking runtime work on a request path",
            function.fingerprint.name
        ),
        vec![
            "synchronous runtime bridging inside handlers tends to stall executor capacity"
                .to_string(),
        ],
    )]
}

fn runtime_builder_loop_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !contains_any(&function.body_text, &["for ", "while ", "loop {"]) {
        return Vec::new();
    }

    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &[
            "Runtime::new(",
            "Builder::new_multi_thread(",
            "Builder::new_current_thread(",
        ],
    ) else {
        return Vec::new();
    };

    vec![function_finding(
        file,
        function,
        "rust_runtime_builder_in_loop",
        Severity::Info,
        line,
        format!(
            "function {} rebuilds runtime infrastructure inside looped control flow",
            function.fingerprint.name
        ),
        vec![
            "runtime builders are usually startup infrastructure rather than repeated work"
                .to_string(),
        ],
    )]
}

fn notify_shutdown_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !contains_any(&function.body_text, &["Notify::new(", "notified().await"]) {
        return Vec::new();
    }

    if contains_any(
        &function.body_text,
        &[
            "shutdown",
            "cancel",
            "cancellation",
            "abort",
            "stop",
            "close",
        ],
    ) {
        return Vec::new();
    }

    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &["Notify::new(", "notified().await"],
    ) else {
        return Vec::new();
    };

    vec![function_finding(
        file,
        function,
        "rust_notify_without_shutdown_contract",
        Severity::Info,
        line,
        format!(
            "function {} waits on notification state without a visible shutdown branch",
            function.fingerprint.name
        ),
        vec![
            "notify-based coordination should normally make cancellation or shutdown explicit"
                .to_string(),
        ],
    )]
}

fn process_env_toggle_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !request_path_like(file, function)
        && !contains_any(&function.body_text, &["for ", "while ", "loop {"])
    {
        return Vec::new();
    }

    let Some(line) = first_line_with_any(
        &function.body_text,
        function.fingerprint.start_line,
        &[
            "set_var(",
            "remove_var(",
            "std::env::set_var(",
            "std::env::remove_var(",
        ],
    ) else {
        return Vec::new();
    };

    vec![function_finding(
        file,
        function,
        "rust_process_global_env_toggle",
        Severity::Warning,
        line,
        format!(
            "function {} mutates process-global environment state at runtime",
            function.fingerprint.name
        ),
        vec![
            "runtime env toggles are global side effects that are hard to isolate safely"
                .to_string(),
        ],
    )]
}

fn request_path_like(file: &ParsedFile, function: &ParsedFunction) -> bool {
    has_import_fragment(file, "axum")
        && contains_any(
            &function.signature_text,
            &["State<", "Json<", "Path<", "Query<", "Form<"],
        )
        || has_import_fragment(file, "actix_web")
            && contains_any(
                &function.signature_text,
                &["HttpRequest", "web::Json<", "web::Data<", "Responder"],
            )
        || has_import_fragment(file, "tonic")
            && contains_any(
                &function.signature_text,
                &["Request<", "tonic::Request<", "Streaming<"],
            )
        || contains_any(
            &function.signature_text,
            &["Request<", "HttpRequest", "Responder", "Handler", "State<"],
        )
}

fn has_import_fragment(file: &ParsedFile, fragment: &str) -> bool {
    file.imports
        .iter()
        .any(|import| import.path.contains(fragment) || import.alias.contains(fragment))
}
