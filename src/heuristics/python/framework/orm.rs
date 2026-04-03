use super::*;

pub(crate) fn sqlalchemy_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !(has_import(file, "sqlalchemy") || has_import(file, "Session"))
    {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    if body.contains("Session(")
        && !body.contains("with ")
        && !body.contains(".close()")
        && let Some(line) = find_line(body, "Session(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "sqlalchemy_session_not_closed",
            Severity::Warning,
            file,
            function,
            line,
            "creates a Session without context manager or .close(); use 'with Session() as session:'",
        ));
    }

    if is_handler_or_view(function, file)
        && body.contains("create_engine(")
        && let Some(line) = find_line(body, "create_engine(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "sqlalchemy_create_engine_per_request",
            Severity::Warning,
            file,
            function,
            line,
            "creates engine per request; reuse a process-level engine",
        ));
    }

    let lines: Vec<&str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") && trimmed.ends_with(':') {
            loop_indent = Some(indent_level(line));
            continue;
        }
        if let Some(li) = loop_indent
            && !trimmed.is_empty()
            && indent_level(line) <= li
            && !trimmed.starts_with('#')
        {
            loop_indent = None;
        }
        if loop_indent.is_some() && !trimmed.is_empty() {
            if trimmed.contains("session.commit()") || trimmed.contains(".commit()") {
                findings.push(make_finding(
                    "sqlalchemy_commit_per_row_in_loop",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "commits inside a loop; batch changes and commit once after the loop",
                ));
            }
            if trimmed.contains("session.query(") || trimmed.contains("session.execute(") {
                findings.push(make_finding(
                    "sqlalchemy_query_in_loop",
                    Severity::Info,
                    file,
                    function,
                    function.fingerprint.start_line + i,
                    "queries inside a loop; batch with .in_() or bulk operations",
                ));
            }
        }
    }

    if body.contains("session.query(")
        && !body.contains("joinedload")
        && !body.contains("subqueryload")
        && !body.contains("selectinload")
    {
        let has_loop_access = lines.iter().enumerate().any(|(i, line)| {
            let t = line.trim();
            i > 0
                && t.starts_with("for ")
                && lines[0..i].iter().any(|l| l.contains("session.query("))
        });
        if has_loop_access
            && let Some(line) = find_line(body, "session.query(", function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "sqlalchemy_n_plus_one_lazy_load",
                Severity::Info,
                file,
                function,
                line,
                "queries without eager loading; add joinedload/subqueryload to prevent N+1",
            ));
        }
    }

    let python = function.python_evidence();
    if python.is_async
        && body.contains("Session(")
        && !body.contains("expire_on_commit=False")
        && let Some(line) = find_line(body, "Session(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "sqlalchemy_expire_on_commit_default_in_async",
            Severity::Info,
            file,
            function,
            line,
            "async session uses expire_on_commit=True (default); set False to avoid implicit I/O",
        ));
    }

    findings
}

pub(crate) fn sqlmodel_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "sqlmodel") {
        return Vec::new();
    }

    let body = &function.body_text;
    let mut findings = Vec::new();

    for (line, trimmed) in collect_loop_lines(body, function.fingerprint.start_line) {
        if trimmed.contains(".exec(") {
            findings.push(make_finding(
                "sqlmodel_session_exec_in_loop",
                Severity::Info,
                file,
                function,
                line,
                "calls Session.exec(...) inside a loop; batch the query or fetch rows in one statement",
            ));
            break;
        }
    }

    for (line, trimmed) in collect_loop_lines(body, function.fingerprint.start_line) {
        if trimmed.contains(".commit(") {
            findings.push(make_finding(
                "sqlmodel_commit_per_row_in_loop",
                Severity::Info,
                file,
                function,
                line,
                "commits inside a loop; accumulate changes and commit once after the loop",
            ));
            break;
        }
    }

    if is_handler_or_view(function, file)
        && body.contains(".exec(")
        && (body.contains(".all(") || body.contains(".all()"))
        && body.contains("select(")
        && !body.contains(".limit(")
        && let Some(line) = find_line(body, ".exec(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "sqlmodel_unbounded_select_in_handler",
            Severity::Warning,
            file,
            function,
            line,
            "executes a SQLModel select().all() path in a handler without an obvious limit or pagination boundary",
        ));
    }

    findings
}

pub(crate) fn pydantic_v2_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "pydantic") {
        return Vec::new();
    }

    let body = &function.body_text;
    let mut findings = Vec::new();

    if body.contains("json.loads(")
        && body.contains("model_validate(")
        && !body.contains("model_validate_json(")
        && let Some(line) = find_line(body, "json.loads(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "pydantic_model_validate_after_json_loads",
            Severity::Info,
            file,
            function,
            line,
            "parses JSON with json.loads(...) before Pydantic validation; prefer model_validate_json(...) when validating raw JSON payloads",
        ));
    }

    if body.contains("model_dump(")
        && body.contains("json.dumps(")
        && !body.contains("model_dump_json(")
        && let Some(line) = find_line(body, "json.dumps(", function.fingerprint.start_line)
    {
        findings.push(make_finding(
            "pydantic_model_dump_then_json_dumps",
            Severity::Info,
            file,
            function,
            line,
            "serializes model_dump() through json.dumps(...); prefer model_dump_json(...) when producing JSON directly",
        ));
    }

    findings
}
