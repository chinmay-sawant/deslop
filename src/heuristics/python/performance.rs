use std::collections::BTreeMap;

use crate::analysis::{CallSite, ImportSpec, ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

const REQUEST_METHODS: &[&str] = &[
    "get", "post", "put", "patch", "delete", "head", "options", "request",
];
const SUBPROCESS_CALLS: &[&str] = &["run", "call", "check_call", "check_output", "Popen"];
const PATH_READ_CALLS: &[&str] = &["read_text", "read_bytes"];
const PATH_WRITE_CALLS: &[&str] = &["write_text", "write_bytes"];

pub(super) fn string_concat_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .concat_loops
        .iter()
        .map(|line| Finding {
            rule_id: "string_concat_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} concatenates strings inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=loop_local_string_concatenation".to_string(),
                "suggestion=collect parts and join once after the loop".to_string(),
            ],
        })
        .collect()
}

pub(super) fn blocking_sync_io_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let python = function.python_evidence();

    if function.is_test_function || !python.is_async {
        return Vec::new();
    }

    let alias_lookup = import_alias_lookup(&file.imports);
    let mut findings = Vec::new();

    for call in &function.calls {
        let Some(evidence) = blocking_sync_io_evidence(call, &alias_lookup) else {
            continue;
        };

        findings.push(Finding {
            rule_id: "blocking_sync_io_in_async".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "async function {} performs synchronous I/O inside the event loop",
                function.fingerprint.name
            ),
            evidence: vec![format!("blocking_call={evidence}")],
        });
    }

    findings
}

pub(super) fn full_dataset_load_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    function
        .calls
        .iter()
        .filter_map(|call| {
            let evidence = if call
                .receiver
                .as_deref()
                .is_some_and(|receiver| receiver.starts_with("open("))
                && matches!(call.name.as_str(), "read" | "readlines")
            {
                Some(format!(
                    "{}.{}() reads the full file into memory",
                    call.receiver.as_deref().unwrap_or("open(...)"),
                    call.name
                ))
            } else if call
                .receiver
                .as_deref()
                .is_some_and(|receiver| receiver.starts_with("Path("))
                && PATH_READ_CALLS.contains(&call.name.as_str())
            {
                Some(format!(
                    "{}.{}() materializes the full file payload",
                    call.receiver.as_deref().unwrap_or("Path(...)"),
                    call.name
                ))
            } else {
                None
            }?;

            Some(Finding {
                rule_id: "full_dataset_load".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} materializes an entire file payload in memory",
                    function.fingerprint.name
                ),
                evidence: vec![format!("full_read={evidence}")],
            })
        })
        .collect()
}

pub(super) fn list_materialization_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .list_materialization_lines
        .iter()
        .map(|line| Finding {
            rule_id: "list_materialization_first_element".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} materializes a list just to read the first element",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=list_materialization_for_first_element".to_string(),
                "suggestion=prefer next(iter(...), default) for first-item access".to_string(),
            ],
        })
        .collect()
}

pub(super) fn deque_candidate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .deque_operation_lines
        .iter()
        .map(|line| Finding {
            rule_id: "deque_candidate_queue".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} performs queue-style list operations that may want collections.deque",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=list_queue_operations".to_string(),
                "suggestion=pop(0) or insert(0, ...) may want collections.deque".to_string(),
            ],
        })
        .collect()
}

pub(super) fn temp_collection_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .temp_collection_lines
        .iter()
        .map(|line| Finding {
            rule_id: "temporary_collection_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} allocates a temporary collection inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=loop_local_scratch_collection".to_string(),
                "impact=repeated list or dict allocation inside the loop body".to_string(),
            ],
        })
        .collect()
}

pub(super) fn recursive_traversal_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let python = function.python_evidence();

    if function.is_test_function
        || python.recursive_call_lines.is_empty()
        || function.fingerprint.line_count < 12
    {
        return Vec::new();
    }

    python
        .recursive_call_lines
        .iter()
        .map(|line| Finding {
            rule_id: "recursive_traversal_risk".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses direct recursion and may need an iterative traversal for deep inputs",
                function.fingerprint.name
            ),
            evidence: vec![format!(
                "recursive_calls={}",
                python.recursive_call_lines.len()
            ), format!("line_count={}", function.fingerprint.line_count)],
        })
        .collect()
}

pub(super) fn list_membership_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .list_membership_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "list_membership_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} performs list-style membership checks inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=list_membership_inside_loop".to_string(),
                "suggestion=prefer a set when order is irrelevant".to_string(),
            ],
        })
        .collect()
}

pub(super) fn repeated_len_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .repeated_len_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "repeated_len_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} repeats len(...) checks inside a loop",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=repeated_len_checks_inside_loop".to_string(),
                "suggestion=cache the length locally when the container is unchanged".to_string(),
            ],
        })
        .collect()
}

fn blocking_sync_io_evidence(
    call: &CallSite,
    alias_lookup: &BTreeMap<String, String>,
) -> Option<String> {
    if call.receiver.is_none() && call.name == "open" {
        return Some("open() performs blocking file I/O inside async code".to_string());
    }

    if let Some(receiver) = &call.receiver {
        if receiver.starts_with("open(")
            && matches!(call.name.as_str(), "read" | "readlines" | "write")
        {
            return Some(format!(
                "{receiver}.{}() performs blocking file I/O",
                call.name
            ));
        }

        if receiver.starts_with("Path(")
            && (PATH_READ_CALLS.contains(&call.name.as_str())
                || PATH_WRITE_CALLS.contains(&call.name.as_str()))
        {
            return Some(format!(
                "{receiver}.{}() performs blocking file I/O",
                call.name
            ));
        }

        if let Some(import_path) = alias_lookup.get(receiver) {
            if import_path == "requests" && REQUEST_METHODS.contains(&call.name.as_str()) {
                return Some(format!("{receiver}.{} resolves to requests", call.name));
            }
            if import_path == "subprocess" && SUBPROCESS_CALLS.contains(&call.name.as_str()) {
                return Some(format!("{receiver}.{} resolves to subprocess", call.name));
            }
            if import_path == "time" && call.name == "sleep" {
                return Some(format!("{receiver}.sleep resolves to time.sleep"));
            }
        }
    }

    if let Some(import_path) = alias_lookup.get(&call.name) {
        if import_path.starts_with("requests.") && REQUEST_METHODS.contains(&call.name.as_str()) {
            return Some(format!("{}() was imported from {import_path}", call.name));
        }
        if import_path.starts_with("subprocess.") && SUBPROCESS_CALLS.contains(&call.name.as_str())
        {
            return Some(format!("{}() was imported from {import_path}", call.name));
        }
        if import_path == "time.sleep" {
            return Some("sleep() was imported from time.sleep".to_string());
        }
    }

    None
}

fn import_alias_lookup(imports: &[ImportSpec]) -> BTreeMap<String, String> {
    imports
        .iter()
        .map(|import| (import.alias.clone(), import.path.clone()))
        .collect()
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

pub(super) fn project_agnostic_performance_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let mut findings = Vec::new();
    let body = &function.body_text;
    let lower_body = body.to_ascii_lowercase();
    let line = function.fingerprint.start_line;

    let push = |rule_id: &str, severity: Severity, message: String| Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message,
        evidence: vec![format!("function={}", function.fingerprint.name)],
    };

    if lower_body.matches("open(").count() >= 2 {
        findings.push(push(
            "repeated_file_open_for_same_resource_within_single_operation",
            Severity::Info,
            format!(
                "function {} reopens files repeatedly within one operation",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(
        body,
        &["read()", "read_text()", "read_bytes()", "readlines()"],
    ) && !contains_any(body, &["for line in", "iter(", "yield"])
        && !contains_any(&lower_body, &["subprocess", "popen", "command"])
    {
        findings.push(push(
            "eager_full_file_or_stream_read_when_incremental_iteration_suffices",
            Severity::Info,
            format!(
                "function {} eagerly reads full file or stream payloads where incremental iteration may suffice",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(body, &[".decode()", ".encode()"])
        && !contains_any(body, &["replace(", "strip(", "split("])
    {
        findings.push(push(
            "bytes_text_bytes_roundtrip_without_transformation",
            Severity::Info,
            format!(
                "function {} round-trips bytes to text and back without clear transformation",
                function.fingerprint.name
            ),
        ));
    }

    if body.contains("+=")
        && contains_any(body, &["\"", "'"])
        && (body.contains("for ") || body.contains("while "))
    {
        findings.push(push(
            "quadratic_string_building_via_plus_equals",
            Severity::Warning,
            format!(
                "function {} grows strings incrementally with += inside a loop",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("re.").count() >= 2 || lower_body.matches("regex").count() >= 2 {
        findings.push(push(
            "multiple_regex_passes_over_same_text_without_precompiled_plan",
            Severity::Info,
            format!(
                "function {} performs multiple regex passes over the same text",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["json.dumps(", "csv.writer(", "bytesio("])
        && !contains_any(&lower_body, &["yield", "stream"])
    {
        findings.push(push(
            "full_response_or_export_buffered_before_incremental_consumer_use",
            Severity::Info,
            format!(
                "function {} buffers full output before handing it to a downstream consumer",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["tempfile.", "namedtemporaryfile", "mkdtemp"])
        && !contains_any(
            &lower_body,
            &["external", "subprocess", "popen", "run(", "command", "whisper", "ffmpeg"],
        )
    {
        findings.push(push(
            "temporary_file_used_for_pure_in_memory_transformation",
            Severity::Info,
            format!(
                "function {} uses temporary files for a transform that may be in-memory",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(
        &lower_body,
        &["threadpoolexecutor(", "processpoolexecutor("],
    ) {
        findings.push(push(
            "thread_pool_or_process_pool_created_and_destroyed_per_call",
            Severity::Info,
            format!(
                "function {} creates an executor per call instead of reusing one",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["deepcopy(", ".copy("])
        && !contains_any(&lower_body, &["mutate", "update", "append", "pop"])
    {
        findings.push(push(
            "large_object_cloned_before_read_only_operation",
            Severity::Info,
            format!(
                "function {} clones data before a read-only flow",
                function.fingerprint.name
            ),
        ));
    }

    if (lower_body.matches("os.stat(").count()
        + lower_body.matches(".exists()").count()
        + lower_body.matches("is_file(").count()
        + lower_body.matches("is_dir(").count()
        >= 2)
        && lower_body.matches("open(").count() >= 1
    {
        findings.push(push(
            "repeated_stat_or_exists_calls_before_single_followup_operation",
            Severity::Info,
            format!(
                "function {} performs repeated stat/exists checks before one follow-up action",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["write(", "save(", "insert("])
        && !contains_any(&lower_body, &[".extend(", "extend(", "bulk_", "batch_", "executemany"])
        && (lower_body.matches("for ").count() >= 1 || lower_body.matches("while ").count() >= 1)
        && (lower_body.matches("write(").count()
            + lower_body.matches("save(").count()
            + lower_body.matches("insert(").count()
            >= 2)
    {
        findings.push(push(
            "batchable_writes_executed_one_at_a_time",
            Severity::Info,
            format!(
                "function {} executes writes one at a time on an iteration path",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("for ").count() >= 2
        && contains_any(&lower_body, &["normalize", "strip", "lower", "parse"])
    {
        findings.push(push(
            "same_dataset_normalized_in_multiple_full_passes",
            Severity::Info,
            format!(
                "function {} appears to normalize the same dataset in multiple passes",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(body, &["list(", "tuple("])
        && contains_any(&lower_body, &["len(", "if values", "if items"])
    {
        findings.push(push(
            "generator_materialized_to_tuple_or_list_only_for_len_or_truthiness",
            Severity::Info,
            format!(
                "function {} materializes generator output only to inspect size or truthiness",
                function.fingerprint.name
            ),
        ));
    }

    if lower_body.matches("sorted(").count() >= 1
        && contains_any(&lower_body, &["[:", "min(", "max(", "next("])
    {
        findings.push(push(
            "full_collection_sorted_when_partial_order_or_selection_suffices",
            Severity::Info,
            format!(
                "function {} sorts whole collections where partial selection may suffice",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["compress(", "hashlib.", ".encode("])
        && contains_any(&lower_body, &["if not", "if value is none", "return"])
    {
        findings.push(push(
            "compression_hashing_or_encoding_performed_before_cheap_reject_checks",
            Severity::Info,
            format!(
                "function {} performs expensive transforms before cheap reject checks",
                function.fingerprint.name
            ),
        ));
    }

    if function.python_evidence().is_async
        && contains_any(&lower_body, &["json.dumps(", "hashlib.", "sorted(", "sum("])
    {
        findings.push(push(
            "event_loop_path_executes_cpu_bound_transformation_synchronously",
            Severity::Warning,
            format!(
                "async function {} performs CPU-heavy transformation work inline",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["write(", "send("])
        && lower_body.matches("for ").count() >= 1
        && !contains_any(&lower_body, &[".extend(", "buffer", "chunk", "flush"])
        && function.fingerprint.line_count >= 10
    {
        findings.push(push(
            "repeated_small_writes_without_buffering_or_join",
            Severity::Info,
            format!(
                "function {} performs repeated small writes without visible buffering",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["dict(", ".copy("])
        && !contains_any(&lower_body, &["update(", "pop(", "setdefault(", "[", "del ", "hydrat"])
        && !contains_any(&lower_body, &["dict(row", "dict(r)", "sqlite", "cursor", "fetchone", "fetchall", "row_factory"])
    {
        findings.push(push(
            "copy_of_mapping_created_only_to_read_values",
            Severity::Info,
            format!(
                "function {} copies mappings only to read them",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["json.dumps(", "pickle.dumps("])
        && contains_any(&lower_body, &["==", "hash(", "cache_key"])
    {
        findings.push(push(
            "serialization_cost_paid_only_to_compare_or_hash_intermediate_state",
            Severity::Info,
            format!(
                "function {} serializes data only to compare or hash intermediate state",
                function.fingerprint.name
            ),
        ));
    }

    if contains_any(&lower_body, &["list(", "dict(", "set("])
        && lower_body.matches("append(").count() >= 3
    {
        findings.push(push(
            "large_in_memory_intermediate_created_where_streaming_pipeline_would_do",
            Severity::Info,
            format!(
                "function {} builds large in-memory intermediates where streaming may suffice",
                function.fingerprint.name
            ),
        ));
    }

    findings
}
