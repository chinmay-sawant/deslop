mod celery_click;
mod django;
mod orm;
mod web;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

pub(super) use self::celery_click::{celery_task_findings, click_typer_command_findings};
pub(super) use self::django::{
    django_extra_findings, django_loop_db_findings, django_n_plus_one_findings,
    django_queryset_findings, django_values_findings,
};
pub(super) use self::orm::{pydantic_v2_findings, sqlalchemy_findings, sqlmodel_findings};
pub(super) use self::web::{
    fastapi_handler_findings, flask_handler_findings, handler_fanout_findings, middleware_findings,
    response_extra_findings, template_response_findings,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

// ── Import-gating helpers ─────────────────────────────────────────────────────

fn has_import(file: &ParsedFile, module: &str) -> bool {
    file.imports
        .iter()
        .any(|imp| imp.path.contains(module) || imp.alias.contains(module))
}

fn is_handler_or_view(function: &ParsedFunction, file: &ParsedFile) -> bool {
    // Check body_text of the function for common decorator patterns
    let sig = &function.signature_text;
    sig.contains("@app.route")
        || sig.contains("@bp.route")
        || sig.contains("@router.")
        || sig.contains("@api_view")
        || sig.contains("@action")
        || sig.contains("@app.get")
        || sig.contains("@app.post")
        || sig.contains("@app.put")
        || sig.contains("@app.delete")
        || (has_import(file, "django")
            && (function.fingerprint.name == "get"
                || function.fingerprint.name == "post"
                || function.fingerprint.name == "put"
                || function.fingerprint.name == "delete"
                || function.fingerprint.name == "list"
                || function.fingerprint.name == "create"
                || function.fingerprint.name == "update"
                || function.fingerprint.name == "destroy"))
}

fn is_middleware(function: &ParsedFunction) -> bool {
    let sig = &function.signature_text;
    sig.contains("@app.before_request")
        || sig.contains("@app.after_request")
        || sig.contains("@app.middleware")
        || sig.contains("process_request")
        || sig.contains("process_response")
        || sig.contains("process_view")
}

fn is_celery_task(function: &ParsedFunction, file: &ParsedFile) -> bool {
    if !has_import(file, "celery") {
        return false;
    }

    let sig = &function.signature_text;
    sig.contains("@shared_task")
        || sig.contains(".task(")
        || sig.contains(".task\n")
        || sig.contains(".task\r\n")
}

fn is_click_or_typer_command(function: &ParsedFunction, file: &ParsedFile) -> bool {
    if !(has_import(file, "click") || has_import(file, "typer")) {
        return false;
    }

    let sig = &function.signature_text;
    sig.contains("@click.command")
        || sig.contains("@click.group")
        || sig.contains(".command(")
        || sig.contains(".callback(")
}

fn env_lookup_lines(body: &str, base_line: usize) -> Vec<usize> {
    body.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let trimmed = line.trim();
            (trimmed.contains("os.getenv(")
                || trimmed.contains("os.environ[")
                || trimmed.contains("os.environ.get("))
            .then_some(base_line + i)
        })
        .collect()
}

fn collect_loop_lines<'a>(body: &'a str, base_line: usize) -> Vec<(usize, &'a str)> {
    let lines: Vec<&'a str> = body.lines().collect();
    let mut loop_indent: Option<usize> = None;
    let mut loop_lines = Vec::new();

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
            loop_lines.push((base_line + i, trimmed));
        }
    }

    loop_lines
}

fn python_binding_name(text: &str) -> Option<&str> {
    let (left, _) = text.split_once(" = ")?;
    let binding = left.trim().split(',').next()?.trim();
    (!binding.is_empty()
        && binding
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric()))
    .then_some(binding)
}

fn find_line(body: &str, needle: &str, base_line: usize) -> Option<usize> {
    for (i, line) in body.lines().enumerate() {
        if line.contains(needle) {
            return Some(base_line + i);
        }
    }
    None
}

fn make_finding(
    rule_id: &str,
    severity: Severity,
    file: &ParsedFile,
    function: &ParsedFunction,
    line: usize,
    msg_suffix: &str,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!("function {} {msg_suffix}", function.fingerprint.name),
        evidence: vec![format!("rule={rule_id}")],
    }
}
