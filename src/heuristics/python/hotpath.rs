use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

pub(super) fn regex_compile_in_hotpath_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .regex_in_hotpath_lines
        .iter()
        .map(|line| Finding {
            rule_id: "regex_compile_in_hot_path".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} compiles a regex inside a loop; move re.compile() to module level",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=re.compile_inside_loop".to_string(),
                "suggestion=precompile at module level and reuse the compiled pattern".to_string(),
            ],
        })
        .collect()
}

pub(super) fn json_repeated_call_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();
    let mut findings = Vec::new();

    for (key, line) in python.repeated_call_same_arg {
        let rule_id = if key.starts_with("json.loads(") || key.starts_with("json.load(") {
            "json_loads_same_payload_multiple_times"
        } else if key.starts_with("json.dumps(") {
            "repeated_json_dumps_same_object"
        } else {
            continue;
        };

        findings.push(Finding {
            rule_id: rule_id.to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} calls {key} multiple times; cache the result in a local variable",
                function.fingerprint.name
            ),
            evidence: vec![format!("repeated_call={key}")],
        });
    }

    findings
}

pub(super) fn sorted_first_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .sorted_first_lines
        .iter()
        .map(|line| Finding {
            rule_id: "sorted_only_for_first_element".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} sorts a collection only to read the first or last element; prefer min() or max()",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=sorted_subscript_0_or_minus_1".to_string(),
                "suggestion=use min() for [0] or max() for [-1] to avoid O(n log n) sort".to_string(),
            ],
        })
        .collect()
}

pub(super) fn len_comprehension_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .len_comprehension_lines
        .iter()
        .map(|line| Finding {
            rule_id: "list_comprehension_only_for_length".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} builds a list comprehension only to take its length; use sum(1 for ...) instead",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=len_of_list_comprehension".to_string(),
                "suggestion=sum(1 for x in iterable if condition) avoids full list allocation".to_string(),
            ],
        })
        .collect()
}

pub(super) fn readlines_then_iterate_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .readlines_then_iterate_lines
        .iter()
        .map(|line| Finding {
            rule_id: "readlines_then_iterate".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} calls .readlines() and iterates; iterate the file object directly instead",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=readlines_then_loop".to_string(),
                "suggestion=for line in file: avoids loading all lines into memory".to_string(),
            ],
        })
        .collect()
}

pub(super) fn read_splitlines_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .read_splitlines_lines
        .iter()
        .map(|line| Finding {
            rule_id: "read_then_splitlines".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} calls .read().splitlines(); iterate the file line-by-line instead",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=read_then_split_into_lines".to_string(),
                "suggestion=for line in file: avoids reading the entire file into memory"
                    .to_string(),
            ],
        })
        .collect()
}

pub(super) fn in_list_literal_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .in_list_literal_lines
        .iter()
        .map(|line| Finding {
            rule_id: "in_check_on_list_literal".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} checks membership in a list literal; use a set literal {{...}} for O(1) lookup",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=in_check_on_list_literal".to_string(),
                "suggestion=replace [a, b, c] with {a, b, c} for constant-time membership checks".to_string(),
            ],
        })
        .collect()
}

pub(super) fn startswith_chain_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .startswith_chain_lines
        .iter()
        .map(|line| Finding {
            rule_id: "string_startswith_endswith_chain".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} chains multiple .startswith() or .endswith() calls with or; use a tuple argument instead",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=startswith_or_endswith_chain".to_string(),
                "suggestion=s.startswith((a, b, c)) is cleaner and avoids repeated method calls".to_string(),
            ],
        })
        .collect()
}

pub(super) fn enumerate_range_len_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .enumerate_range_len_lines
        .iter()
        .map(|line| Finding {
            rule_id: "enumerate_on_range_len".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} uses enumerate(range(len(...))) or range(len(...)); use enumerate(collection) directly",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=enumerate_range_len_antipattern".to_string(),
                "suggestion=for i, item in enumerate(collection): is more Pythonic and equally fast".to_string(),
            ],
        })
        .collect()
}

pub(super) fn csv_flush_per_row_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .csv_flush_per_row_lines
        .iter()
        .map(|line| Finding {
            rule_id: "csv_writer_flush_per_row".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} flushes inside a write loop; flush once after all rows are written",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=flush_inside_write_loop".to_string(),
                "suggestion=move .flush() outside the loop to reduce I/O syscalls".to_string(),
            ],
        })
        .collect()
}

pub(super) fn write_in_loop_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .write_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "write_without_buffering_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} calls .write() inside a loop without visible buffering",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=unbuffered_write_in_loop".to_string(),
                "suggestion=collect output and write once, or use a BufferedWriter".to_string(),
            ],
        })
        .collect()
}

pub(super) fn repeated_open_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();
    let mut findings = Vec::new();

    for (path_arg, line) in python.repeated_open_same_file {
        findings.push(Finding {
            rule_id: "repeated_open_same_file_in_function".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} opens {} multiple times; read once and reuse the content",
                function.fingerprint.name, path_arg
            ),
            evidence: vec![format!("repeated_open={path_arg}")],
        });
    }

    findings
}

pub(super) fn dict_materialization_in_loop_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function {
        return Vec::new();
    }

    let python = function.python_evidence();

    python
        .dict_materialization_in_loop_lines
        .iter()
        .map(|line| Finding {
            rule_id: "dict_items_or_keys_materialized_in_loop".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: *line,
            end_line: *line,
            message: format!(
                "function {} materializes dict keys/values/items into a list inside a loop; iterate the view directly",
                function.fingerprint.name
            ),
            evidence: vec![
                "pattern=list_dict_keys_values_items_in_loop".to_string(),
                "suggestion=iterate d.keys(), d.values(), or d.items() directly without list()".to_string(),
            ],
        })
        .collect()
}
