use super::*;

pub(crate) fn django_queryset_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    if (body.contains(".count() > 0")
        || body.contains(".count() >= 1")
        || body.contains(".count() != 0"))
        && let Some(line) = find_line(body, ".count()", function.fingerprint.start_line)
    {
        findings.push(Finding {
            rule_id: "django_queryset_count_then_exists".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} uses .count() > 0; prefer .exists() to avoid full count",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=count_then_exists".to_string()],
        });
    }

    for call in &function.calls {
        if call.name == "len" && body.contains("len(") && body.contains(".objects") {
            findings.push(Finding {
                rule_id: "django_queryset_len_instead_of_count".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: call.line,
                end_line: call.line,
                message: format!(
                    "function {} uses len(queryset) which loads all objects; prefer .count()",
                    function.fingerprint.name
                ),
                evidence: vec!["pattern=len_on_queryset".to_string()],
            });
            break;
        }
    }

    if is_handler_or_view(function, file)
        && body.contains(".objects.all()")
        && !body.contains("[:")
        && !body.contains(".first()")
        && !body.contains("paginate")
        && !body.contains("Paginator")
        && let Some(line) = find_line(body, ".objects.all()", function.fingerprint.start_line)
    {
        findings.push(Finding {
            rule_id: "django_all_without_limit_in_view".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} loads all() without pagination or limit in a view",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=unbounded_queryset_in_view".to_string()],
        });
    }

    if (body.contains(".order_by('?')") || body.contains(".order_by(\"?\")"))
        && let Some(line) = find_line(body, ".order_by(", function.fingerprint.start_line)
    {
        findings.push(Finding {
            rule_id: "django_queryset_order_by_random".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "function {} uses .order_by('?') which causes ORDER BY RANDOM() full table scan",
                function.fingerprint.name
            ),
            evidence: vec!["pattern=order_by_random".to_string()],
        });
    }

    findings
}

pub(crate) fn django_n_plus_one_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    let has_select_related = body.contains("select_related") || body.contains("prefetch_related");
    if !has_select_related {
        let lines: Vec<&str> = body.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("for ")
                && trimmed.contains(".objects")
                && trimmed.contains(".filter(")
            {
                findings.push(Finding {
                    rule_id: "django_n_plus_one_no_select_related".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: function.fingerprint.start_line + i,
                    end_line: function.fingerprint.start_line + i,
                    message: format!(
                        "function {} iterates over a queryset without select_related/prefetch_related",
                        function.fingerprint.name
                    ),
                    evidence: vec!["pattern=n_plus_one_query_risk".to_string()],
                });
            }
        }
    }

    findings
}

pub(crate) fn django_loop_db_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();
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
        if loop_indent.is_none() || trimmed.is_empty() {
            continue;
        }
        let sl = function.fingerprint.start_line + i;

        if trimmed.contains(".save(") && !trimmed.contains("update_fields") {
            findings.push(make_finding(
                "django_save_full_model_in_loop",
                Severity::Info,
                file,
                function,
                sl,
                "saves a full model inside a loop without update_fields; consider bulk_update()",
            ));
        }
        if trimmed.contains(".objects.create(")
            || (trimmed.ends_with(".save()") && trimmed.contains("("))
        {
            findings.push(make_finding(
                "django_create_single_in_loop",
                Severity::Info,
                file,
                function,
                sl,
                "creates a single model inside a loop; consider bulk_create()",
            ));
        }
        if trimmed.contains(".delete()") && !trimmed.contains(".objects") {
            findings.push(make_finding(
                "django_delete_single_in_loop",
                Severity::Info,
                file,
                function,
                sl,
                "deletes instances in a loop; consider QuerySet.delete()",
            ));
        }
        if trimmed.contains(".execute(") && (body.contains("cursor") || body.contains(".raw(")) {
            findings.push(make_finding(
                "django_raw_sql_in_loop",
                Severity::Info,
                file,
                function,
                sl,
                "executes raw SQL inside a loop; consider batching",
            ));
        }
    }

    findings
}

pub(crate) fn django_values_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

    let lines: Vec<&str> = body.lines().collect();
    let mut qs_vars: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for line in &lines {
        let trimmed = line.trim();
        if let Some(eq_idx) = trimmed.find(" = ") {
            let rhs = &trimmed[eq_idx + 3..];
            if rhs.contains(".objects.") || rhs.contains(".filter(") || rhs.contains(".all()") {
                let var = trimmed[..eq_idx].trim().to_string();
                qs_vars.insert(var, 0);
            }
        }
    }
    let qs_keys: Vec<String> = qs_vars.keys().cloned().collect();
    for line in &lines {
        let trimmed = line.trim();
        for var in &qs_keys {
            if trimmed.starts_with("for ") && trimmed.contains(&format!(" in {var}"))
                || trimmed.contains(&format!("len({var})"))
                || trimmed.contains(&format!("list({var})"))
                || trimmed.contains(&format!("if {var}:"))
            {
                *qs_vars.entry(var.clone()).or_default() += 1;
            }
        }
    }
    for (var, count) in &qs_vars {
        if *count >= 2
            && let Some(line) = find_line(body, var, function.fingerprint.start_line)
        {
            findings.push(make_finding(
                "django_queryset_evaluated_multiple_times",
                Severity::Warning,
                file,
                function,
                line,
                &format!(
                    "queryset '{var}' appears to be evaluated multiple times, causing duplicate SQL"
                ),
            ));
        }
    }

    findings
}

pub(crate) fn django_extra_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if function.is_test_function || !has_import(file, "django") {
        return Vec::new();
    }
    let body = &function.body_text;
    let mut findings = Vec::new();

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
        if loop_indent.is_some()
            && !trimmed.is_empty()
            && trimmed.contains(".update(")
            && trimmed.contains(".objects.filter(")
        {
            findings.push(make_finding(
                "django_update_single_in_loop",
                Severity::Info,
                file,
                function,
                function.fingerprint.start_line + i,
                "updates single objects in a loop; consider bulk_update() or QuerySet.update()",
            ));
        }
    }

    if is_handler_or_view(function, file) {
        for pattern in &["migrate", "makemigrations", "schema_editor", "RunPython"] {
            if body.contains(pattern)
                && let Some(line) = find_line(body, pattern, function.fingerprint.start_line)
            {
                findings.push(make_finding(
                    "django_migration_code_in_view",
                    Severity::Warning,
                    file,
                    function,
                    line,
                    "references migration/schema operations in a view; these belong in migration files",
                ));
            }
        }
    }

    if body.contains(".objects.filter(") || body.contains(".objects.all()") {
        let has_values =
            body.contains(".values(") || body.contains(".values_list(") || body.contains(".only(");
        if !has_values {
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("for ") && trimmed.contains(".objects.") {
                    findings.push(make_finding(
                        "django_values_vs_full_model_in_loop",
                        Severity::Info,
                        file,
                        function,
                        function.fingerprint.start_line + i,
                        "hydrates full models; use .values() or .only() if only a few fields are needed",
                    ));
                }
            }
        }
    }

    findings
}
