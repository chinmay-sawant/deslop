use super::*;

pub(crate) fn celery_task_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !is_celery_task(function, file) {
        return Vec::new();
    }

    let body = &function.body_text;
    let mut findings = Vec::new();

    let loop_lines = collect_loop_lines(body, function.fingerprint.start_line);
    let has_canvas_escape = body.contains("group(")
        || body.contains("chord(")
        || body.contains("chunks(")
        || body.contains("starmap(")
        || body.contains(".map(");
    if !has_canvas_escape
        && let Some((line, _)) = loop_lines
            .iter()
            .find(|(_, text)| text.contains(".delay(") || text.contains(".apply_async("))
    {
        findings.push(make_finding(
            "celery_delay_in_loop_without_canvas",
            Severity::Warning,
            file,
            function,
            *line,
            "dispatches Celery tasks inside a loop without an obvious canvas primitive like group() or chord()",
        ));
    }

    let mut async_result_bindings = Vec::<(String, usize)>::new();
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if (trimmed.contains(".delay(") || trimmed.contains(".apply_async("))
            && let Some(binding) = python_binding_name(trimmed)
        {
            async_result_bindings.push((binding.to_string(), function.fingerprint.start_line + i));
        }
    }
    for (binding, assigned_line) in async_result_bindings {
        if let Some(line) = find_line(
            body,
            &format!("{binding}.get("),
            function.fingerprint.start_line,
        ) {
            findings.push(Finding {
                rule_id: "celery_result_get_inside_task".to_string(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: line,
                end_line: line,
                message: format!(
                    "function {} waits on a Celery result inside a task",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("AsyncResult-like binding {binding} assigned at line {assigned_line}"),
                    format!("{binding}.get(...) observed at line {line}"),
                ],
            });
            break;
        }
    }

    let env_reads = env_lookup_lines(body, function.fingerprint.start_line);
    if env_reads.len() >= 3 {
        findings.push(Finding {
            rule_id: "celery_task_reads_env_per_invocation".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: env_reads[0],
            end_line: env_reads[0],
            message: format!(
                "function {} reads environment configuration repeatedly inside a Celery task",
                function.fingerprint.name
            ),
            evidence: vec![format!("environment reads observed at lines {env_reads:?}")],
        });
    }

    findings
}

pub(crate) fn click_typer_command_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !is_click_or_typer_command(function, file) {
        return Vec::new();
    }

    let body = &function.body_text;
    let mut findings = Vec::new();

    for pattern in &[
        "json.load(",
        "yaml.safe_load(",
        "toml.load(",
        "configparser.",
        ".read_text()",
    ] {
        if body.contains(pattern)
            && let Some(line) = find_line(body, pattern, function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "click_typer_config_file_loaded_per_command",
                Severity::Info,
                file,
                function,
                line,
                "loads config or settings files inside a click/typer command; centralize config bootstrap instead",
            ));
            break;
        }
    }

    let env_reads = env_lookup_lines(body, function.fingerprint.start_line);
    if env_reads.len() >= 3 {
        findings.push(Finding {
            rule_id: "click_typer_env_lookup_per_command".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: env_reads[0],
            end_line: env_reads[0],
            message: format!(
                "function {} scatters environment lookups across one command invocation",
                function.fingerprint.name
            ),
            evidence: vec![format!("environment reads observed at lines {env_reads:?}")],
        });
    }

    for pattern in &[
        "requests.Session(",
        "httpx.Client(",
        "httpx.AsyncClient(",
        "aiohttp.ClientSession(",
    ] {
        if body.contains(pattern)
            && let Some(line) = find_line(body, pattern, function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "click_typer_http_client_created_per_command",
                Severity::Info,
                file,
                function,
                line,
                "creates an HTTP client inside a click/typer command instead of reusing a shared client factory",
            ));
            break;
        }
    }

    findings
}
