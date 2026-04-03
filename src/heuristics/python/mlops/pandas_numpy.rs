use super::*;

pub(crate) fn pandas_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_any_import(file, &["pandas", "pd"]) {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    if body.contains(".iterrows()")
        && let Some(line) = find_line(body, ".iterrows()", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "pandas_iterrows_in_loop",
            Severity::Info,
            file,
            function,
            line,
            "uses df.iterrows() which is very slow; prefer vectorized ops, .apply(), or .itertuples()",
        ));
    }

    if body.contains(".apply(lambda") {
        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains(".apply(lambda") {
                let has_simple_op = trimmed.contains("x + ")
                    || trimmed.contains("x - ")
                    || trimmed.contains("x * ")
                    || trimmed.contains("x / ")
                    || trimmed.contains("x.lower()")
                    || trimmed.contains("x.upper()")
                    || trimmed.contains("x.strip()")
                    || trimmed.contains("str(x)")
                    || trimmed.contains("int(x)")
                    || trimmed.contains("float(x)");
                if has_simple_op {
                    findings.push(make_finding(
                        "pandas_apply_with_simple_vectorizable_op",
                        Severity::Info,
                        file,
                        function,
                        function.fingerprint.start_line + i,
                        "uses .apply(lambda) for a simple operation with a vectorized equivalent",
                    ));
                }
            }
        }
    }

    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && (trimmed.contains("pd.concat(")
                || trimmed.contains("df.append(")
                || trimmed.contains(".append(") && trimmed.contains("DataFrame"))
        {
            findings.push(make_finding(
                "pandas_concat_in_loop",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "concatenates DataFrames inside a loop; collect all and concat once",
            ));
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
    }

    if body.contains("pd.read_csv(") || body.contains("read_csv(") {
        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("read_csv(") && !trimmed.contains("dtype") {
                findings.push(make_finding(
                    "pandas_read_csv_without_dtypes",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "reads CSV without dtype parameter; specify dtypes to avoid double-pass type inference",
                ));
            }
        }
    }

    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("']['") && trimmed.contains(" = ") && !trimmed.contains("==") {
            findings.push(make_finding(
                "pandas_chain_assignment_warning",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "uses chained indexing which may cause SettingWithCopyWarning; use .loc[] instead",
            ));
        }
    }

    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if (trimmed.contains(".drop(")
            || trimmed.contains(".rename(")
            || trimmed.contains(".fillna(")
            || trimmed.contains(".dropna("))
            && !trimmed.contains("inplace=True")
            && !trimmed.contains(" = ")
            && !trimmed.starts_with("return")
        {
            findings.push(make_finding(
                "pandas_inplace_false_reassignment_missing",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "calls DataFrame method without assigning result or inplace=True; result is silently discarded",
            ));
        }
    }

    loop_indent = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some() && !trimmed.is_empty() && trimmed.contains(".to_dict(") {
            findings.push(make_finding(
                "pandas_to_dict_records_in_loop",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "calls .to_dict() inside a loop; use vectorized access or .itertuples()",
            ));
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
    }

    if body.contains(".merge(") || body.contains("pd.merge(") {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains(".merge(") && !trimmed.contains("validate") {
                findings.push(make_finding(
                    "pandas_merge_without_validation",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "merges DataFrames without validate parameter; risk of silent row duplication",
                ));
            }
        }
    }

    if !function.is_test_function {
        for (i, line) in body.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("print(df")
                || trimmed.starts_with("display(df")
                || trimmed.contains(".to_string()"))
                && !file.path.ends_with("notebook.py")
                && !file.path.to_string_lossy().contains("notebook")
            {
                findings.push(make_finding(
                    "pandas_full_dataframe_print_in_production",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "prints/displays a full DataFrame in production code; use logging or head()",
                ));
            }
        }
    }

    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if (trimmed.contains(".eval(f") || trimmed.contains(".query(f"))
            && (trimmed.contains("{") || trimmed.contains("format("))
        {
            findings.push(make_finding(
                "pandas_eval_string_manipulation",
                Severity::Warning,
                file,
                function,
                function.fingerprint.start_line + i,
                "uses f-string in .eval()/.query() which risks injection; use parameterized operations",
            ));
        }
    }

    loop_indent = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && trimmed.contains(".copy()")
            && (trimmed.contains("df") || trimmed.contains("DataFrame"))
        {
            findings.push(make_finding(
                "pandas_copy_in_loop",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "copies a DataFrame inside a loop; consider using views or method chaining",
            ));
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
    }

    findings
}

pub(crate) fn numpy_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_any_import(file, &["numpy", "np"]) {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ")
            && trimmed.ends_with(':')
            && (trimmed.contains("np.")
                || trimmed.contains("array")
                || trimmed.contains("range(len("))
        {
            findings.push(make_finding(
                "numpy_python_loop_over_array",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "uses Python loop over array; prefer vectorized NumPy operations",
            ));
        }
    }

    let mut loop_indent: Option<usize> = None;
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if loop_indent.is_some() && !trimmed.is_empty() && trimmed.contains("np.append(") {
            findings.push(make_finding(
                "numpy_append_in_loop",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "uses np.append() in a loop; pre-allocate with np.zeros/np.empty and fill",
            ));
        }
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && (trimmed.contains("np.vstack(")
                || trimmed.contains("np.hstack(")
                || trimmed.contains("np.concatenate("))
        {
            findings.push(make_finding(
                "numpy_vstack_hstack_in_loop",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "stacks arrays in a loop; collect and stack once after the loop",
            ));
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
    }

    if body.contains(".tolist()") {
        let in_loop = body.lines().any(|line| {
            let t = line.trim();
            t.contains(".tolist()") && (body.contains("for ") || body.contains("while "))
        });
        if in_loop && let Some(line) = find_line(body, ".tolist()", function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "numpy_tolist_in_hot_path",
                Severity::Info,
                file,
                function,
                line,
                "calls .tolist() in hot path; keep data as NumPy arrays to avoid Python object overhead",
            ));
        }
    }

    findings
}
