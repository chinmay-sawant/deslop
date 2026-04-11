use crate::analysis::Language;
use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::io::{DEFAULT_MAX_BYTES, read_to_string_limited};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

use super::super::common::import_alias_lookup;
use super::framework_patterns::{is_gin_handler, is_http_handler};

const CONTEXT_FACTORY_ESCAPES: &[&str] = &["Background", "TODO"];
const HTTP_CONTEXTLESS_CALLS: &[&str] = &["Get", "Head", "Post", "PostForm", "NewRequest"];
const EXEC_CONTEXTLESS_CALLS: &[&str] = &["Command"];
const NET_CONTEXTLESS_CALLS: &[&str] = &["Dial", "DialTimeout"];
const DB_CONTEXTLESS_CALLS: &[&str] = &["Query", "QueryRow", "Exec", "Get", "Select"];
const CACHE_METHOD_HINTS: &[&str] = &[
    "Get",
    "Set",
    "Delete",
    "Del",
    "Load",
    "Store",
    "Fetch",
    "Remember",
    "Put",
    "Invalidate",
    "Evict",
];
const CACHE_IMPORT_HINTS: &[&str] = &[
    "github.com/redis/go-redis",
    "github.com/go-redis/redis",
    "github.com/bradfitz/gomemcache",
    "github.com/allegro/bigcache",
    "github.com/dgraph-io/ristretto",
];
const CONTEXT_DECLARATION_BUILTINS: &[&str] = &["string", "int", "int64", "uint", "bool"];
const CONTEXT_DEPENDENCY_VALUE_HINTS: &[&str] = &[
    "service",
    "repo",
    "repository",
    "client",
    "config",
    "logger",
    "request",
    "payload",
    "dto",
    "body",
    "map[",
    "[]",
    "bytes.",
    "make(",
    "new(",
];
const CONTEXT_METADATA_HINTS: &[&str] = &[
    "requestid",
    "request_id",
    "traceid",
    "trace_id",
    "spanid",
    "span_id",
    "userid",
    "user_id",
    "tenantid",
    "tenant_id",
    "locale",
    "auth",
    "principal",
];

pub(crate) fn cache_context_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file
        || !file_looks_cache_related(file)
        || file.imports.iter().any(|import| import.path == "context")
    {
        return Vec::new();
    }

    let mut findings = Vec::new();
    for interface in file.interfaces() {
        if !interface_looks_cache_related(file, &interface.name) {
            continue;
        }

        for method in &interface.methods {
            if !cache_method_signature_without_context(method) {
                continue;
            }

            findings.push(Finding {
                rule_id: "cache_interface_method_missing_context".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: interface.line,
                end_line: interface.line,
                message: format!(
                    "cache-oriented interface {} exposes IO-style methods without context.Context",
                    interface.name
                ),
                evidence: vec![
                    format!("interface {} declared at line {}", interface.name, interface.line),
                    format!("cache-style method: {method}"),
                    "the file does not import context, so the cache interface cannot expose context-aware methods"
                        .to_string(),
                    "cache calls are often network-bound and benefit from request cancellation and deadlines"
                        .to_string(),
                ],
            });
        }
    }

    findings
}

pub(crate) fn generic_context_file_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let mut findings = context_struct_field_findings(file);
    findings.extend(context_key_file_findings(file));
    findings
}

pub(crate) fn ctx_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.go_evidence().has_context_parameter {
        return Vec::new();
    }

    let import_aliases = import_alias_lookup(&file.imports);

    function
        .calls
        .iter()
        .filter_map(|call| {
            let receiver = call.receiver.as_ref()?;
            let import_path = import_aliases.get(receiver)?;

            let is_context_aware_api = matches!(import_path.as_str(), "net/http")
                && HTTP_CONTEXTLESS_CALLS.contains(&call.name.as_str())
                || matches!(import_path.as_str(), "os/exec")
                    && EXEC_CONTEXTLESS_CALLS.contains(&call.name.as_str())
                || matches!(import_path.as_str(), "net")
                    && NET_CONTEXTLESS_CALLS.contains(&call.name.as_str());

            if !is_context_aware_api {
                return None;
            }

            Some(Finding {
                rule_id: "missing_context".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} performs context-aware work without accepting context.Context",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "context-free API call: {receiver}.{} from {import_path}",
                        call.name
                    ),
                    "function signature does not accept context.Context".to_string(),
                ],
            })
        })
        .collect()
}

pub(crate) fn context_parameter_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if file.is_test_file
        || function.is_test_function
        || !function.go_evidence().has_context_parameter
    {
        return Vec::new();
    }

    let context_aliases = context_aliases(file);
    if context_aliases.is_empty() {
        return Vec::new();
    }

    let params = split_signature_params(&function.signature_text);
    if params.len() <= 1 {
        return Vec::new();
    }

    let has_contextful_work = !function.go_evidence().db_query_calls.is_empty()
        || !function.go_evidence().gorm_query_chains.is_empty()
        || function_looks_cache_related(file, function)
        || function.calls.iter().any(|call| {
            let Some(receiver) = call.receiver.as_ref() else {
                return false;
            };
            let import_path = import_alias_lookup(&file.imports).get(receiver).cloned();
            matches!(
                import_path.as_deref(),
                Some("net/http" | "os/exec" | "net" | "database/sql")
            )
        });
    if !has_contextful_work {
        return Vec::new();
    }

    let first_param = params.first().map(String::as_str).unwrap_or_default();
    if parameter_mentions_context(first_param, &context_aliases) {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "context_not_first_parameter".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.start_line,
        message: format!(
            "function {} accepts context.Context but not as its first non-receiver parameter",
            function.fingerprint.name
        ),
        evidence: vec![
            format!("signature: {}", compact_signature(&function.signature_text)),
            "Go convention keeps context.Context first so cancellation and deadlines stay easy to spot".to_string(),
        ],
    }]
}

pub(crate) fn cancel_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.context_factory_calls
        .iter()
        .filter(|factory_call| {
            !function
                .calls
                .iter()
                .any(|call| call.receiver.is_none() && call.name == factory_call.cancel_name)
        })
        .map(|factory_call| Finding {
            rule_id: "missing_cancel_call".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: factory_call.line,
            end_line: factory_call.line,
            message: format!(
                "function {} creates a derived context without an observed cancel call",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "context.{} assigns cancel function {}",
                    factory_call.factory_name, factory_call.cancel_name
                ),
                "no local cancel() or defer cancel() call was observed".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn request_context_background_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if file.is_test_file
        || function.is_test_function
        || !function_has_available_context(file, function)
    {
        return Vec::new();
    }

    let go = function.go_evidence();
    if go.goroutines.is_empty() || has_documented_context_decoupling(function) {
        return Vec::new();
    }

    let lower_body = function.body_text.to_ascii_lowercase();
    if lower_body.contains("withoutcancel(") || lower_body.contains("withoutcancel (") {
        return Vec::new();
    }

    if has_waitgroup_or_errgroup(function) {
        return Vec::new();
    }

    let uses_request_context = lower_body.contains(".request.context()")
        || lower_body.contains("r.context()")
        || lower_body.contains("ctx")
        || lower_body.contains("context()");
    if !uses_request_context {
        return Vec::new();
    }

    let Some(line) = go.goroutines.first().copied() else {
        return Vec::new();
    };

    vec![Finding {
        rule_id: "request_context_passed_to_background_task_without_detach".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!(
            "function {} launches a background goroutine while request-scoped context still appears to flow into it",
            function.fingerprint.name
        ),
        evidence: vec![
            "request or caller-owned context is available in the parent function".to_string(),
            format!("goroutine launch observed at line {line}"),
            "no context.WithoutCancel(...) or explicit detached background context was observed".to_string(),
            "background work should detach request cancellation before crossing the request lifetime".to_string(),
        ],
    }]
}

pub(crate) fn context_withvalue_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if file.is_test_file || function.is_test_function {
        return Vec::new();
    }

    let context_aliases = context_aliases(file);
    if context_aliases.is_empty() {
        return Vec::new();
    }

    let mut findings = Vec::new();

    for (line_no, text) in body_lines(function) {
        if !context_aliases
            .iter()
            .any(|alias| text.contains(&format!("{alias}.WithValue(")))
        {
            continue;
        }

        let Some(args) = call_arguments(&text, "WithValue") else {
            continue;
        };
        if args.len() < 3 {
            continue;
        }

        let key_arg = args[1].trim();
        let value_arg = args[2].trim();
        let key_lower = normalize_context_token(key_arg);
        let value_lower = normalize_context_token(value_arg);

        if context_key_looks_builtin(key_arg) || context_key_looks_plain_identifier(key_arg) {
            findings.push(Finding {
                rule_id: "context_key_uses_exported_or_builtin_type".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: line_no,
                end_line: line_no,
                message: format!(
                    "function {} uses a built-in or string-like context key in context.WithValue",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("context.WithValue(...) observed at line {}", line_no),
                    format!("key argument: {key_arg}"),
                    "prefer an unexported package-local key type to avoid collisions".to_string(),
                ],
            });
        }

        let looks_metadata = CONTEXT_METADATA_HINTS
            .iter()
            .any(|hint| key_lower.contains(hint) || value_lower.contains(hint));
        let suspicious_value = CONTEXT_DEPENDENCY_VALUE_HINTS
            .iter()
            .any(|hint| value_lower.contains(hint));

        if suspicious_value && !looks_metadata {
            findings.push(Finding {
                rule_id: "context_withvalue_used_for_dependencies_or_large_payloads".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: line_no,
                end_line: line_no,
                message: format!(
                    "function {} uses context.WithValue for dependency-like or payload-like data",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("context.WithValue(...) observed at line {}", line_no),
                    format!("key argument: {key_arg}"),
                    format!("value argument: {value_arg}"),
                    "context values are better reserved for lightweight request metadata"
                        .to_string(),
                ],
            });
        }
    }

    findings
}

pub(crate) fn cache_method_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let import_aliases = import_alias_lookup(&file.imports);

    if file.is_test_file
        || function.is_test_function
        || !function_looks_cache_related(file, function)
        || !function_uses_cache_client(file, function)
    {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter(|call| {
            let Some(receiver) = call.receiver.as_deref() else {
                return false;
            };
            import_aliases.get(receiver).is_some_and(|import_path| {
                import_path == "context"
                    && CONTEXT_FACTORY_ESCAPES.contains(&call.name.as_str())
            })
        })
        .map(|call| Finding {
            rule_id: "cache_method_uses_context_background".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "cache-oriented method {} creates a fresh background context instead of accepting or propagating one",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("context.{} observed at line {}", call.name, call.line),
                "cache clients often issue network IO and should honor caller cancellation"
                    .to_string(),
            ],
        })
        .collect()
}

pub(crate) fn sleep_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    function
        .go_evidence()
        .sleep_loops
        .iter()
        .map(|line| Finding {
            rule_id: "sleep_polling".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses time.Sleep inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "time.Sleep appears inside a loop, which often indicates polling".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn busy_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let go = function.go_evidence();

    go.busy_wait_lines
        .iter()
        .map(|line| Finding {
            rule_id: "busy_waiting".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} spins on a select default branch inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "select { default: ... } appears inside a loop".to_string(),
                "default branches in looped selects often indicate busy-waiting".to_string(),
            ],
        })
        .collect()
}

pub(crate) fn propagate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    if !function_has_available_context(file, function)
        || file.is_test_file
        || has_documented_context_decoupling(function)
    {
        return Vec::new();
    }

    let import_aliases = import_alias_lookup(&file.imports);
    let go = function.go_evidence();
    let mut findings = Vec::new();

    for call in &function.calls {
        let Some(receiver) = call.receiver.as_ref() else {
            continue;
        };
        let Some(import_path) = import_aliases.get(receiver) else {
            continue;
        };

        if import_path == "context" && CONTEXT_FACTORY_ESCAPES.contains(&call.name.as_str()) {
            findings.push(Finding {
                rule_id: "context_background_used".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} has a caller-owned context available but still creates context.{}() locally",
                    function.fingerprint.name, call.name
                ),
                evidence: vec![
                    if function.go_evidence().has_context_parameter {
                        "function signature already accepts context.Context".to_string()
                    } else {
                        "request handler already has a request-scoped context available".to_string()
                    },
                    format!("observed call: {receiver}.{}()", call.name),
                    "prefer propagating the incoming context instead of starting from Background or TODO"
                        .to_string(),
                ],
            });
            continue;
        }

        if !is_contextless_wrapper_call(import_path, &call.name) {
            continue;
        }

        findings.push(Finding {
            rule_id: "missing_context_propagation".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} accepts context.Context but still calls {}.{} without propagating ctx",
                function.fingerprint.name, receiver, call.name
            ),
            evidence: vec![
                "function signature already accepts context.Context".to_string(),
                format!(
                    "context-free API call: {receiver}.{} from {import_path}",
                    call.name
                ),
                "prefer a context-aware variant or request construction that forwards ctx"
                    .to_string(),
            ],
        });
    }

    for call in &function.calls {
        let Some(receiver) = call.receiver.as_deref() else {
            continue;
        };

        if receiver.contains('.') && is_contextless_method_name(&call.name) {
            findings.push(Finding {
                rule_id: "missing_context_propagation".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} accepts context.Context but calls {}.{} without forwarding ctx",
                    function.fingerprint.name, receiver, call.name
                ),
                evidence: vec![
                    "function signature already accepts context.Context".to_string(),
                    format!("receiver-field or wrapper call observed: {receiver}.{}", call.name),
                    "field-backed clients should prefer context-aware request, network, or exec variants"
                        .to_string(),
                ],
            });
        }
    }

    for query_call in go.db_query_calls {
        if !DB_CONTEXTLESS_CALLS.contains(&query_call.method_name.as_str()) {
            continue;
        }

        let receiver = query_call.receiver.as_deref().unwrap_or("<unknown>");
        findings.push(Finding {
            rule_id: "missing_context_propagation".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: query_call.line,
            end_line: query_call.line,
            message: format!(
                "function {} accepts context.Context but still calls {}.{} without a context-aware DB variant",
                function.fingerprint.name, receiver, query_call.method_name
            ),
            evidence: vec![
                "function signature already accepts context.Context".to_string(),
                format!("observed database-style call: {receiver}.{}", query_call.method_name),
                "prefer QueryContext, QueryRowContext, ExecContext, or another ctx-aware wrapper"
                    .to_string(),
            ],
        });
    }

    let Some(package_name) = file.package_name.as_deref() else {
        return findings;
    };
    let Some(current_package) = index.package_for_file(Language::Go, &file.path, package_name)
    else {
        return findings;
    };

    for call in &function.calls {
        if call.receiver.is_some() || !current_package.has_contextless_wrapper_function(&call.name)
        {
            continue;
        }

        findings.push(Finding {
            rule_id: "missing_context_propagation".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} accepts context.Context but calls local wrapper {} without propagating ctx through the wrapper chain",
                function.fingerprint.name, call.name
            ),
            evidence: vec![
                "function signature already accepts context.Context".to_string(),
                format!("local package call observed: {}(...)", call.name),
                "the local callee also performs context-aware work without accepting context.Context"
                    .to_string(),
            ],
        });
    }

    findings
}

fn function_has_available_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    function.go_evidence().has_context_parameter
        || is_http_handler(file, function)
        || is_gin_handler(file, function)
}

fn context_struct_field_findings(file: &ParsedFile) -> Vec<Finding> {
    let aliases = context_aliases(file);
    if aliases.is_empty() {
        return Vec::new();
    }

    let mut findings = Vec::new();
    for go_struct in file.go_structs() {
        for field in &go_struct.fields {
            if aliases
                .iter()
                .any(|alias| field.type_text.contains(&format!("{alias}.Context")))
            {
                findings.push(Finding {
                    rule_id: "context_stored_in_struct_field".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: None,
                    start_line: field.line,
                    end_line: field.line,
                    message: format!(
                        "struct {} stores context.Context in field {}",
                        go_struct.name, field.name
                    ),
                    evidence: vec![
                        format!("field {} uses type {}", field.name, field.type_text),
                        "contexts are request-scoped lifetimes and usually should flow through method parameters instead".to_string(),
                    ],
                });
            }
        }
    }

    findings
}

fn context_key_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for package_var in file.package_vars() {
        let lower_name = package_var.name.to_ascii_lowercase();
        if !lower_name.ends_with("key") {
            continue;
        }

        let type_signal = package_var.type_text.as_deref().is_some_and(|ty| {
            CONTEXT_DECLARATION_BUILTINS
                .iter()
                .any(|builtin| ty.contains(builtin))
        });
        let value_signal = package_var
            .value_text
            .as_deref()
            .is_some_and(context_key_looks_builtin);

        if !(type_signal || value_signal) {
            continue;
        }

        findings.push(Finding {
            rule_id: "context_key_uses_exported_or_builtin_type".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: package_var.line,
            end_line: package_var.line,
            message: format!(
                "package key {} uses a built-in type or literal value shape that risks context collisions",
                package_var.name
            ),
            evidence: vec![
                format!("package variable declared at line {}", package_var.line),
                package_var
                    .type_text
                    .as_ref()
                    .map(|ty| format!("declared type: {ty}"))
                    .unwrap_or_else(|| "no explicit type text observed".to_string()),
                package_var
                    .value_text
                    .as_ref()
                    .map(|value| format!("value text: {value}"))
                    .unwrap_or_else(|| "no explicit value text observed".to_string()),
            ],
        });
    }

    let Ok(source) = read_to_string_limited(&file.path, DEFAULT_MAX_BYTES) else {
        return findings;
    };
    for (offset, raw_line) in source.lines().enumerate() {
        let line_no = offset + 1;
        let line = raw_line.trim();
        if !line.starts_with("type ") || !line.contains("Key ") {
            continue;
        }

        let Some(rest) = line.strip_prefix("type ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let Some(type_name) = parts.next() else {
            continue;
        };
        let Some(underlying) = parts.next() else {
            continue;
        };

        if !type_name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            continue;
        }
        if !CONTEXT_DECLARATION_BUILTINS.contains(&underlying) {
            continue;
        }

        findings.push(Finding {
            rule_id: "context_key_uses_exported_or_builtin_type".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: line_no,
            end_line: line_no,
            message: format!(
                "exported context key type {} uses built-in underlying type {}",
                type_name, underlying
            ),
            evidence: vec![
                format!("type declaration at line {line_no}: {line}"),
                "prefer an unexported package-local key type so external packages cannot collide on key usage".to_string(),
            ],
        });
    }

    findings
}

fn file_looks_cache_related(file: &ParsedFile) -> bool {
    let lower_path = file.path.to_string_lossy().to_ascii_lowercase();
    let package = file
        .package_name
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    lower_path.contains("/cache/")
        || lower_path.contains("_cache.")
        || lower_path.ends_with("/cache.go")
        || package.contains("cache")
        || file.imports.iter().any(|import| {
            CACHE_IMPORT_HINTS
                .iter()
                .any(|hint| import.path.contains(hint))
        })
}

fn interface_looks_cache_related(file: &ParsedFile, interface_name: &str) -> bool {
    file_looks_cache_related(file) || interface_name.to_ascii_lowercase().contains("cache")
}

fn cache_method_signature_without_context(signature: &str) -> bool {
    let trimmed = signature.trim();
    CACHE_METHOD_HINTS
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

fn function_looks_cache_related(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let receiver = function
        .fingerprint
        .receiver_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let name = function.fingerprint.name.to_ascii_lowercase();

    file_looks_cache_related(file)
        || receiver.contains("cache")
        || name.contains("cache")
        || CACHE_METHOD_HINTS
            .iter()
            .any(|hint| function.fingerprint.name.starts_with(hint))
}

fn function_uses_cache_client(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let lower_body = function.body_text.to_ascii_lowercase();
    file.imports.iter().any(|import| {
        CACHE_IMPORT_HINTS
            .iter()
            .any(|hint| import.path.contains(hint))
    }) || [
        ".get(",
        ".set(",
        ".del(",
        ".delete(",
        ".load(",
        ".store(",
        ".fetch(",
        ".remember(",
        ".invalidate(",
        ".evict(",
    ]
    .iter()
    .any(|marker| lower_body.contains(marker))
}

fn is_contextless_wrapper_call(import_path: &str, call_name: &str) -> bool {
    matches!(import_path, "net/http") && HTTP_CONTEXTLESS_CALLS.contains(&call_name)
        || matches!(import_path, "os/exec") && EXEC_CONTEXTLESS_CALLS.contains(&call_name)
        || matches!(import_path, "net") && NET_CONTEXTLESS_CALLS.contains(&call_name)
}

fn is_contextless_method_name(call_name: &str) -> bool {
    HTTP_CONTEXTLESS_CALLS.contains(&call_name)
        || EXEC_CONTEXTLESS_CALLS.contains(&call_name)
        || NET_CONTEXTLESS_CALLS.contains(&call_name)
}

fn has_documented_context_decoupling(function: &ParsedFunction) -> bool {
    let mut combined = function
        .doc_comment
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    combined.push('\n');
    combined.push_str(&function.body_text.to_ascii_lowercase());

    [
        "intentionally detached",
        "intentional detached",
        "intentionally decouple",
        "intentionally decoupled",
        "detached context",
        "background worker",
        "top-level producer",
        "independent of request context",
        "survive request cancellation",
    ]
    .iter()
    .any(|marker| combined.contains(marker))
}

fn body_lines(function: &ParsedFunction) -> Vec<(usize, String)> {
    function
        .body_text
        .lines()
        .enumerate()
        .map(|(offset, line)| (function.body_start_line + offset, line.trim().to_string()))
        .collect()
}

fn context_aliases(file: &ParsedFile) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| import.path == "context")
        .map(|import| import.alias.clone())
        .collect()
}

fn split_signature_params(signature: &str) -> Vec<String> {
    let Some(params_text) = signature_params_text(signature) else {
        return Vec::new();
    };
    split_top_level_csv(params_text)
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn signature_params_text(signature: &str) -> Option<&str> {
    let group_index = if signature.trim_start().starts_with("func (") {
        1
    } else {
        0
    };
    let mut depth = 0usize;
    let mut open = None;
    let mut groups = Vec::new();

    for (idx, ch) in signature.char_indices() {
        match ch {
            '(' => {
                if depth == 0 {
                    open = Some(idx);
                }
                depth += 1;
            }
            ')' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    groups.push((open?, idx));
                }
            }
            _ => {}
        }
    }

    groups
        .get(group_index)
        .map(|(start, end)| &signature[start + 1..*end])
}

fn split_top_level_csv(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;

    for ch in text.chars() {
        match ch {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    parts
}

fn parameter_mentions_context(param_text: &str, aliases: &[String]) -> bool {
    aliases
        .iter()
        .any(|alias| param_text.contains(&format!("{alias}.Context")))
}

fn compact_signature(signature: &str) -> String {
    signature.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn call_arguments(line: &str, call_name: &str) -> Option<Vec<String>> {
    let marker = format!("{call_name}(");
    let start = line.find(&marker)? + marker.len();
    let mut depth = 1usize;
    let mut end = None;

    for (offset, ch) in line[start..].char_indices() {
        match ch {
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

    let end = end?;
    Some(split_top_level_csv(&line[start..end]))
}

fn normalize_context_token(text: &str) -> String {
    text.to_ascii_lowercase()
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace() && *ch != '"' && *ch != '`')
        .collect()
}

fn context_key_looks_builtin(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with('"')
        || trimmed.starts_with('`')
        || trimmed.parse::<i64>().is_ok()
        || matches!(trimmed, "true" | "false" | "nil")
}

fn context_key_looks_plain_identifier(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.contains('.')
        && !trimmed.contains('{')
        && !trimmed.contains('(')
        && trimmed
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_lowercase())
        && trimmed.to_ascii_lowercase().ends_with("key")
}

fn has_waitgroup_or_errgroup(function: &ParsedFunction) -> bool {
    function.calls.iter().any(|call| {
        call.receiver.as_ref().is_some_and(|receiver| {
            matches!(
                receiver.as_str(),
                "wg" | "group" | "g" | "errGroup" | "errgroup"
            ) && matches!(call.name.as_str(), "Add" | "Wait" | "Go")
        })
    })
}
