use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{Language, ParsedFile, ParsedFunction};
use crate::index::{ImportResolution, RepositoryIndex};
use crate::model::{Finding, Severity};

const GOD_FUNCTION_LINE_THRESHOLD: usize = 45;
const GOD_FUNCTION_COMPLEXITY_THRESHOLD: usize = 6;
const GOD_FUNCTION_CALL_THRESHOLD: usize = 8;
const GOD_CLASS_METHOD_THRESHOLD: usize = 8;
const GOD_CLASS_PUBLIC_METHOD_THRESHOLD: usize = 5;
const GOD_CLASS_ATTRIBUTE_THRESHOLD: usize = 8;
const MONOLITHIC_INIT_BYTE_THRESHOLD: usize = 1200;
const MONOLITHIC_INIT_IMPORT_THRESHOLD: usize = 6;
const MONOLITHIC_INIT_FUNCTION_THRESHOLD: usize = 4;
const MONOLITHIC_MODULE_LINE_THRESHOLD: usize = 1500;
const MONOLITHIC_MODULE_BYTE_THRESHOLD: usize = 30_000;
const MONOLITHIC_MODULE_IMPORT_THRESHOLD: usize = 12;
const MONOLITHIC_MODULE_DECLARATION_THRESHOLD: usize = 12;
const MONOLITHIC_MODULE_FUNCTION_THRESHOLD: usize = 8;
const MONOLITHIC_MODULE_ORCHESTRATION_LINE_THRESHOLD: usize = 20;
const MONOLITHIC_MODULE_ORCHESTRATION_CALL_THRESHOLD: usize = 8;
const MONOLITHIC_MODULE_ORCHESTRATION_FUNCTION_THRESHOLD: usize = 2;
const MONOLITHIC_MODULE_CONCERN_THRESHOLD: usize = 2;
const INSTANCE_ATTRIBUTE_THRESHOLD: usize = 10;
const INSTANCE_ATTRIBUTE_ESCALATION_THRESHOLD: usize = 20;
const INSTANCE_ATTRIBUTE_METHOD_THRESHOLD: usize = 3;
const EAGER_CONSTRUCTOR_THRESHOLD: usize = 3;

pub(super) fn god_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let fingerprint = &function.fingerprint;
    if fingerprint.line_count < GOD_FUNCTION_LINE_THRESHOLD
        || fingerprint.complexity_score < GOD_FUNCTION_COMPLEXITY_THRESHOLD
        || fingerprint.call_count < GOD_FUNCTION_CALL_THRESHOLD
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "god_function".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: Some(fingerprint.name.clone()),
        start_line: fingerprint.start_line,
        end_line: fingerprint.end_line,
        message: format!(
            "function {} concentrates too much control flow and behavior",
            fingerprint.name
        ),
        evidence: vec![
            format!("line_count={}", fingerprint.line_count),
            format!("complexity_score={}", fingerprint.complexity_score),
            format!("call_count={}", fingerprint.call_count),
        ],
    }]
}

pub(super) fn monolithic_init_module_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file
        || file
            .path
            .file_name()
            .and_then(|name| name.to_str())
            != Some("__init__.py")
    {
        return Vec::new();
    }

    if file.functions.len() < MONOLITHIC_INIT_FUNCTION_THRESHOLD
        || (file.imports.len() < MONOLITHIC_INIT_IMPORT_THRESHOLD
            && file.byte_size < MONOLITHIC_INIT_BYTE_THRESHOLD)
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "monolithic_init_module".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: "__init__.py is carrying enough imports and behavior to look monolithic"
            .to_string(),
        evidence: vec![
            format!("functions={}", file.functions.len()),
            format!("imports={}", file.imports.len()),
            format!("byte_size={}", file.byte_size),
        ],
    }]
}

pub(super) fn too_many_instance_attributes_findings(file: &ParsedFile) -> Vec<Finding> {
    file.class_summaries
        .iter()
        .filter(|summary| {
            summary.instance_attribute_count >= INSTANCE_ATTRIBUTE_THRESHOLD
                && summary.method_count >= INSTANCE_ATTRIBUTE_METHOD_THRESHOLD
        })
        .map(|summary| {
            let escalated = summary.instance_attribute_count >= INSTANCE_ATTRIBUTE_ESCALATION_THRESHOLD;
            Finding {
                rule_id: "too_many_instance_attributes".to_string(),
                severity: if escalated {
                    Severity::Warning
                } else {
                    Severity::Info
                },
                path: file.path.clone(),
                function_name: None,
                start_line: summary.line,
                end_line: summary.line,
                message: if escalated {
                    format!(
                        "class {} assigns 20 or more instance attributes and looks unusually state-heavy",
                        summary.name
                    )
                } else {
                    format!(
                        "class {} assigns an unusually large number of instance attributes",
                        summary.name
                    )
                },
                evidence: vec![
                    format!("instance_attribute_count={}", summary.instance_attribute_count),
                    format!("method_count={}", summary.method_count),
                    format!(
                        "tier={}",
                        if escalated { "20_plus" } else { "10_plus" }
                    ),
                ],
            }
        })
        .collect()
}

pub(super) fn monolithic_module_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file
        || file
            .path
            .file_name()
            .and_then(|name| name.to_str())
            == Some("__init__.py")
    {
        return Vec::new();
    }

    let declaration_count = file.functions.len() + file.class_summaries.len();
    let mixed_concern_functions = file
        .functions
        .iter()
        .filter(|function| !classify_infrastructure_concerns(file, function).is_empty())
        .filter(|function| {
            let concerns = classify_infrastructure_concerns(file, function);
            concerns.len() >= MONOLITHIC_MODULE_CONCERN_THRESHOLD
                && function.fingerprint.call_count >= MONOLITHIC_MODULE_ORCHESTRATION_CALL_THRESHOLD
        })
        .count();
    let orchestration_functions = file
        .functions
        .iter()
        .filter(|function| {
            !function.is_test_function
                && function.fingerprint.line_count >= MONOLITHIC_MODULE_ORCHESTRATION_LINE_THRESHOLD
                && function.fingerprint.call_count >= MONOLITHIC_MODULE_ORCHESTRATION_CALL_THRESHOLD
        })
        .count();
    let module_concerns = file
        .functions
        .iter()
        .flat_map(|function| classify_infrastructure_concerns(file, function).into_iter())
        .collect::<BTreeSet<_>>();

    if file.line_count < MONOLITHIC_MODULE_LINE_THRESHOLD
        || file.byte_size < MONOLITHIC_MODULE_BYTE_THRESHOLD
        || declaration_count < MONOLITHIC_MODULE_DECLARATION_THRESHOLD
        || file.functions.len() < MONOLITHIC_MODULE_FUNCTION_THRESHOLD
        || file.imports.len() < MONOLITHIC_MODULE_IMPORT_THRESHOLD
        || module_concerns.len() < MONOLITHIC_MODULE_CONCERN_THRESHOLD
        || (mixed_concern_functions == 0
            && orchestration_functions < MONOLITHIC_MODULE_ORCHESTRATION_FUNCTION_THRESHOLD)
    {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "monolithic_module".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: "module is unusually large and still concentrates imports, orchestration, and mixed concerns"
            .to_string(),
        evidence: vec![
            format!("line_count={}", file.line_count),
            format!("functions={}", file.functions.len()),
            format!("classes={}", file.class_summaries.len()),
            format!("imports={}", file.imports.len()),
            format!("byte_size={}", file.byte_size),
            format!("orchestration_functions={}", orchestration_functions),
            format!("mixed_concern_functions={}", mixed_concern_functions),
            format!(
                "concern_categories={}",
                module_concerns.into_iter().collect::<Vec<_>>().join(",")
            ),
        ],
    }]
}

pub(super) fn god_class_findings(file: &ParsedFile) -> Vec<Finding> {
    file.class_summaries
        .iter()
        .filter(|summary| {
            summary.method_count >= GOD_CLASS_METHOD_THRESHOLD
                && summary.public_method_count >= GOD_CLASS_PUBLIC_METHOD_THRESHOLD
                && summary.instance_attribute_count >= GOD_CLASS_ATTRIBUTE_THRESHOLD
        })
        .map(|summary| Finding {
            rule_id: "god_class".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: None,
            start_line: summary.line,
            end_line: summary.end_line,
            message: format!(
                "class {} concentrates too much behavior and mutable state",
                summary.name
            ),
            evidence: vec![
                format!("method_count={}", summary.method_count),
                format!("public_method_count={}", summary.public_method_count),
                format!("instance_attribute_count={}", summary.instance_attribute_count),
            ],
        })
        .collect()
}

pub(super) fn eager_constructor_collaborator_findings(file: &ParsedFile) -> Vec<Finding> {
    file.class_summaries
        .iter()
        .filter(|summary| summary.constructor_collaborator_count >= EAGER_CONSTRUCTOR_THRESHOLD)
        .map(|summary| Finding {
            rule_id: "eager_constructor_collaborators".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: summary.line,
            end_line: summary.line,
            message: format!(
                "class {} builds several collaborators eagerly in __init__",
                summary.name
            ),
            evidence: vec![format!(
                "constructor_collaborator_count={}",
                summary.constructor_collaborator_count
            )],
        })
        .collect()
}

pub(super) fn over_abstracted_wrapper_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut methods_by_class = BTreeMap::<String, Vec<&ParsedFunction>>::new();
    for function in &file.functions {
        if let Some(receiver) = &function.fingerprint.receiver_type {
            methods_by_class
                .entry(receiver.clone())
                .or_default()
                .push(function);
        }
    }

    file.class_summaries
        .iter()
        .filter_map(|summary| {
            let methods = methods_by_class.get(&summary.name)?;
            let shape = classify_over_abstracted_shape(summary, methods)?;

            Some(Finding {
                rule_id: "over_abstracted_wrapper".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: None,
                start_line: summary.line,
                end_line: summary.end_line,
                message: format!(
                    "class {} looks ceremonial enough that a function or dataclass may suffice",
                    summary.name
                ),
                evidence: vec![
                    format!("shape={shape}"),
                    format!("method_count={}", summary.method_count),
                    format!("instance_attribute_count={}", summary.instance_attribute_count),
                ],
            })
        })
        .collect()
}

pub(super) fn mixed_concern_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || function.fingerprint.complexity_score < 4 {
        return Vec::new();
    }

    let categories = classify_infrastructure_concerns(file, function);

    if categories.len() < 2 || function.fingerprint.call_count < 6 {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "mixed_concerns_function".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.end_line,
        message: format!(
            "function {} mixes multiple infrastructure concerns in one body",
            function.fingerprint.name
        ),
        evidence: vec![format!(
            "concern_categories={}",
            categories.into_iter().collect::<Vec<_>>().join(", ")
        )],
    }]
}

pub(super) fn name_responsibility_mismatch_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let lower_name = function.fingerprint.name.to_ascii_lowercase();
    let mutating_call = function
        .calls
        .iter()
        .find(|call| is_mutating_call(&call.name));
    let concerns = classify_infrastructure_concerns(file, function);

    let (message, evidence_line, line) = if is_read_style_name(&lower_name) {
        let Some(call) = mutating_call else {
            return Vec::new();
        };
        (
            format!(
                "function {} has a read-style name but performs a mutating operation",
                function.fingerprint.name
            ),
            format!("mutating call: {}", call.name),
            call.line,
        )
    } else if is_transformation_style_name(&lower_name)
        && (mutating_call.is_some()
            || concerns.contains("http")
            || concerns.contains("persistence"))
    {
        let line = mutating_call
            .map(|call| call.line)
            .unwrap_or(function.fingerprint.start_line);
        (
            format!(
                "function {} is named like a pure transformation but coordinates boundary side effects",
                function.fingerprint.name
            ),
            format!(
                "concern_categories={}{}",
                concerns.iter().copied().collect::<Vec<_>>().join(","),
                mutating_call
                    .map(|call| format!(" mutating_call={}", call.name))
                    .unwrap_or_default()
            ),
            line,
        )
    } else if is_utility_style_name(&lower_name)
        && concerns.len() >= 2
        && function.fingerprint.call_count >= 5
    {
        (
            format!(
                "function {} uses a utility-style name but owns multiple infrastructure concerns",
                function.fingerprint.name
            ),
            format!(
                "concern_categories={}",
                concerns.iter().copied().collect::<Vec<_>>().join(",")
            ),
            function.fingerprint.start_line,
        )
    } else {
        return Vec::new();
    };

    vec![Finding {
        rule_id: "name_responsibility_mismatch".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message,
        evidence: vec![evidence_line],
    }]
}

pub(super) fn module_name_responsibility_mismatch_findings(file: &ParsedFile) -> Vec<Finding> {
    if file.is_test_file {
        return Vec::new();
    }

    let module_name = file.package_name.as_deref().unwrap_or_default().to_ascii_lowercase();
    if !module_name.contains("helper")
        && !module_name.contains("util")
        && !module_name.contains("client")
    {
        return Vec::new();
    }

    let concerns = file
        .functions
        .iter()
        .flat_map(|function| classify_infrastructure_concerns(file, function).into_iter())
        .collect::<BTreeSet<_>>();
    let orchestration_functions = file
        .functions
        .iter()
        .filter(|function| !function.is_test_function && function.fingerprint.call_count >= 6)
        .count();
    if concerns.len() < 2 || orchestration_functions == 0 {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "name_responsibility_mismatch".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: format!(
            "module {} uses a utility-style name but coordinates multiple infrastructure concerns",
            file.package_name.as_deref().unwrap_or("<module>")
        ),
        evidence: vec![format!(
            "concern_categories={}",
            concerns.into_iter().collect::<Vec<_>>().join(",")
        )],
    }]
}

pub(super) fn deep_inheritance_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut class_map = BTreeMap::<String, (&ParsedFile, usize, Vec<String>)>::new();
    for file in files {
        for summary in &file.class_summaries {
            class_map.insert(
                summary.name.clone(),
                (file, summary.line, summary.base_classes.clone()),
            );
        }
    }

    let mut findings = Vec::new();
    for (name, (file, line, bases)) in &class_map {
        let depth = inheritance_depth(name, bases, &class_map);
        if depth < 3 {
            continue;
        }

        findings.push(Finding {
            rule_id: "deep_inheritance_hierarchy".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: *line,
            end_line: *line,
            message: format!("class {name} sits in a deep local inheritance chain"),
            evidence: vec![format!("local_inheritance_depth={depth}")],
        });
    }

    findings
}

pub(super) fn tight_module_coupling_findings(
    files: &[&ParsedFile],
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for file in files {
        if file.is_test_file {
            continue;
        }

        let mut local_imports = BTreeSet::new();
        for import in &file.imports {
            match index.resolve_import_path(Language::Python, &import.path) {
                ImportResolution::Resolved(package) => {
                    local_imports.insert(format!(
                        "{}:{}",
                        package.directory_display(),
                        package.package_name
                    ));
                }
                ImportResolution::Ambiguous(_) | ImportResolution::Unresolved => {}
            }
        }

        if local_imports.len() < 4 {
            continue;
        }

        findings.push(Finding {
            rule_id: "tight_module_coupling".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "module depends on a large number of repository-local modules".to_string(),
            evidence: vec![format!("local_module_dependencies={}", local_imports.len())],
        });
    }

    findings
}

fn inheritance_depth(
    name: &str,
    bases: &[String],
    class_map: &BTreeMap<String, (&ParsedFile, usize, Vec<String>)>,
) -> usize {
    let mut visited = BTreeSet::from([name.to_string()]);
    inheritance_depth_inner(bases, class_map, &mut visited)
}

fn inheritance_depth_inner(
    bases: &[String],
    class_map: &BTreeMap<String, (&ParsedFile, usize, Vec<String>)>,
    visited: &mut BTreeSet<String>,
) -> usize {
    let mut best = 0;
    for base in bases {
        let simple_name = base.rsplit('.').next().unwrap_or(base);
        if !visited.insert(simple_name.to_string()) {
            continue;
        }
        let depth = if let Some((_, _, parent_bases)) = class_map.get(simple_name) {
            1 + inheritance_depth_inner(parent_bases, class_map, visited)
        } else {
            1
        };
        best = best.max(depth);
        visited.remove(simple_name);
    }
    best
}

fn classify_infrastructure_concerns(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> BTreeSet<&'static str> {
    let alias_lookup = file
        .imports
        .iter()
        .map(|import| (import.alias.as_str(), import.path.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut categories = BTreeSet::new();

    for call in &function.calls {
        if call.receiver.is_none() && call.name == "open" {
            categories.insert("filesystem");
            continue;
        }

        let receiver = call.receiver.as_deref().unwrap_or(call.name.as_str());
        let import_path = alias_lookup.get(receiver).copied().unwrap_or(receiver);
        if is_http_like(import_path, &call.name) {
            categories.insert("http");
        }
        if is_persistence_like(import_path, &call.name) {
            categories.insert("persistence");
        }
        if is_filesystem_like(import_path, &call.name) {
            categories.insert("filesystem");
        }
    }

    categories
}

fn is_http_like(import_path: &str, call_name: &str) -> bool {
    import_path.starts_with("requests")
        || import_path.starts_with("httpx")
        || import_path.starts_with("urllib")
        || matches!(call_name, "get" | "post" | "put" | "patch" | "delete" | "request")
}

fn is_persistence_like(import_path: &str, call_name: &str) -> bool {
    import_path.starts_with("sqlite3")
        || import_path.starts_with("sqlalchemy")
        || import_path.starts_with("psycopg")
        || import_path.starts_with("pymongo")
        || import_path.starts_with("redis")
        || matches!(call_name, "execute" | "query" | "commit" | "fetchall" | "fetchone")
}

fn is_filesystem_like(import_path: &str, call_name: &str) -> bool {
    import_path.starts_with("pathlib")
        || matches!(call_name, "open" | "read" | "read_text" | "write" | "write_text")
}

fn classify_over_abstracted_shape(
    summary: &crate::analysis::ClassSummary,
    methods: &[&ParsedFunction],
) -> Option<&'static str> {
    if !summary.base_classes.is_empty()
        || summary.method_count == 0
        || summary.method_count > 3
        || summary.public_method_count > 1
        || summary.instance_attribute_count > 2
        || summary.constructor_collaborator_count > 1
        || summary.end_line.saturating_sub(summary.line) > 40
    {
        return None;
    }

    if methods.iter().any(|method| {
        method.fingerprint.kind.starts_with("async")
            || method.fingerprint.complexity_score > 2
            || !method.exception_handlers.is_empty()
            || matches!(
                method.fingerprint.name.as_str(),
                "__enter__"
                    | "__exit__"
                    | "__aenter__"
                    | "__aexit__"
                    | "start"
                    | "stop"
                    | "close"
                    | "open"
                    | "connect"
                    | "disconnect"
            )
    }) {
        return None;
    }

    let behavior_methods = methods
        .iter()
        .filter(|method| method.fingerprint.name != "__init__")
        .copied()
        .collect::<Vec<_>>();
    if behavior_methods.len() == 1
        && behavior_methods[0].fingerprint.line_count <= 8
        && behavior_methods[0].fingerprint.call_count >= 1
        && behavior_methods[0].fingerprint.call_count <= 3
    {
        return Some("thin_wrapper_or_dataclass");
    }

    None
}

fn is_mutating_call(call_name: &str) -> bool {
    matches!(
        call_name,
        "write" | "save" | "delete" | "update" | "post" | "put" | "commit" | "execute"
    )
}

fn is_read_style_name(name: &str) -> bool {
    name.starts_with("get_") || name.starts_with("load_") || name.starts_with("is_")
}

fn is_transformation_style_name(name: &str) -> bool {
    ["parse_", "normalize_", "format_", "render_", "serialize_", "decode_"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn is_utility_style_name(name: &str) -> bool {
    name.starts_with("helper_") || name.starts_with("util_") || name == "helper" || name == "util"
}