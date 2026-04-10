use crate::analysis::{ParsedFile, ParsedFunction};

pub(super) fn should_skip_print_rule(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let name = &function.fingerprint.name;
    name == "main"
        || name.starts_with("print_")
        || name.starts_with("log_")
        || name.starts_with("display_")
        || name.starts_with("show_")
        || name.starts_with("report_")
        || name.starts_with("dump_")
        || looks_like_tooling_context(file, function)
        || file
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == "__main__.py")
}

fn looks_like_tooling_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let name = function.fingerprint.name.to_ascii_lowercase();
    let tool_name_markers = [
        "run", "load", "validate", "build", "render", "sync", "list", "resolve",
    ];
    if !tool_name_markers.iter().any(|marker| name.contains(marker)) {
        return false;
    }

    file.imports.iter().any(|import| {
        matches!(
            import.path.as_str(),
            "argparse" | "json" | "shutil" | "subprocess" | "sys" | "pathlib"
        )
    })
}

pub(super) fn looks_like_hardcoded_path(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value
            .chars()
            .nth(1)
            .is_some_and(|character| character == ':')
            && value.contains('\\')
        || (value.contains('/') && has_path_like_suffix(value))
}

fn has_path_like_suffix(value: &str) -> bool {
    [
        ".json", ".yaml", ".yml", ".txt", ".csv", ".db", ".sqlite", ".ini", ".cfg", ".conf",
        ".pem", ".log",
    ]
    .iter()
    .any(|suffix| value.ends_with(suffix))
}

pub(super) fn is_commented_code(text: &str) -> bool {
    let normalized = text.trim();
    normalized.starts_with("if ")
        || normalized.starts_with("for ")
        || normalized.starts_with("while ")
        || normalized.starts_with("return ")
        || normalized.starts_with("def ")
        || normalized.starts_with("class ")
        || normalized.starts_with("try:")
        || normalized.starts_with("except ")
        || (normalized.contains('=')
            && normalized.contains('(')
            && normalized
                .chars()
                .any(|character| character.is_ascii_alphabetic()))
}

pub(super) fn looks_like_business_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let markers = [
        "eligib",
        "discount",
        "pricing",
        "price",
        "risk",
        "approve",
        "approval",
        "tier",
        "quota",
        "commission",
        "policy",
        "status",
        "fraud",
        "score",
    ];
    function_or_path_matches(file, function, &markers)
}

pub(super) fn looks_like_boundary_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let markers = [
        "handler",
        "endpoint",
        "route",
        "view",
        "controller",
        "cli",
        "command",
        "main",
        "sync",
        "fetch",
        "publish",
        "process",
        "ingest",
        "import",
        "export",
        "job",
        "startup",
        "bootstrap",
        "config",
    ];
    function_or_path_matches(file, function, &markers)
}

pub(super) fn looks_like_startup_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let markers = ["config", "settings", "startup", "bootstrap", "main", "env"];
    function_or_path_matches(file, function, &markers)
}

pub(super) fn is_input_boundary(file: &ParsedFile, function: &ParsedFunction) -> bool {
    let markers = [
        "cli", "command", "handler", "request", "ingest", "import", "parse",
    ];
    function_or_path_matches(file, function, &markers)
}

fn function_or_path_matches(
    file: &ParsedFile,
    function: &ParsedFunction,
    markers: &[&str],
) -> bool {
    let function_name = function.fingerprint.name.to_ascii_lowercase();
    if markers.iter().any(|marker| function_name.contains(marker)) {
        return true;
    }

    if file.package_name.as_deref().is_some_and(|name| {
        markers
            .iter()
            .any(|marker| name.to_ascii_lowercase().contains(marker))
    }) {
        return true;
    }

    file.path.components().any(|component| {
        let part = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        markers.iter().any(|marker| part.contains(marker))
    })
}

pub(super) fn collect_branch_literals(body_text: &str) -> Vec<String> {
    body_text
        .lines()
        .map(str::trim)
        .filter(|line| is_branch_line(line))
        .flat_map(|line| {
            let mut literals = extract_string_literals(line);
            literals.extend(extract_numeric_literals(line));
            literals
        })
        .collect()
}

fn is_branch_line(line: &str) -> bool {
    line.starts_with("if ")
        || line.starts_with("elif ")
        || line.starts_with("case ")
        || line.starts_with("match ")
}

fn extract_string_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let characters = line.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < characters.len() {
        let quote = characters[index];
        if quote != '\'' && quote != '"' {
            index += 1;
            continue;
        }

        index += 1;
        let start = index;
        while index < characters.len() && characters[index] != quote {
            if characters[index] == '\\' {
                index += 1;
            }
            index += 1;
        }

        if index > start {
            let literal = characters
                .get(start..index)
                .unwrap_or(&[])
                .iter()
                .collect::<String>();
            if !literal.trim().is_empty() {
                literals.push(literal);
            }
        }
        index += 1;
    }

    literals
}

fn extract_numeric_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut current = String::new();

    for character in line.chars() {
        if character.is_ascii_digit() || character == '.' {
            current.push(character);
        } else {
            flush_numeric_literal(&mut current, &mut literals);
        }
    }
    flush_numeric_literal(&mut current, &mut literals);

    literals
}

fn flush_numeric_literal(current: &mut String, literals: &mut Vec<String>) {
    let token = current.trim_matches('.');
    if !token.is_empty()
        && token
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
        && token.chars().any(|character| character.is_ascii_digit())
    {
        literals.push(token.to_string());
    }
    current.clear();
}

pub(super) fn is_policy_literal(literal: &str) -> bool {
    let lower = literal.to_ascii_lowercase();
    lower
        .parse::<f64>()
        .is_ok_and(|value| value >= 0.0 && (value.fract() != 0.0 || value >= 20.0))
        || matches!(
            lower.as_str(),
            "approved"
                | "rejected"
                | "manual_review"
                | "priority"
                | "standard"
                | "premium"
                | "enterprise"
                | "eligible"
                | "blocked"
                | "pending"
        )
}

pub(super) fn is_magic_literal(literal: &str) -> bool {
    let lower = literal.to_ascii_lowercase();
    if let Ok(value) = lower.parse::<f64>() {
        return value.fract() != 0.0 || value >= 20.0;
    }

    lower.len() >= 5 && !matches!(lower.as_str(), "false" | "true" | "none")
}

pub(super) fn count_prefixed_lines(body_text: &str, prefix: &str) -> usize {
    body_text
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with(prefix))
        .count()
}

pub(super) fn http_boundary_calls(file: &ParsedFile, function: &ParsedFunction) -> Vec<String> {
    let alias_lookup = file
        .imports
        .iter()
        .map(|import| (import.alias.as_str(), import.path.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    const HTTP_METHODS: &[&str] = &[
        "get", "post", "put", "patch", "delete", "request", "head", "options", "send",
    ];

    function
        .calls
        .iter()
        .filter_map(|call| {
            let receiver = call.receiver.as_deref().unwrap_or(call.name.as_str());
            let import_path = alias_lookup.get(receiver).copied().unwrap_or(receiver);
            let direct_http_call =
                call.receiver.is_none() && HTTP_METHODS.contains(&call.name.as_str());
            let imported_http_call = import_path.starts_with("requests")
                || import_path.starts_with("httpx")
                || import_path.starts_with("urllib")
                || import_path.starts_with("aiohttp");

            ((direct_http_call || imported_http_call) && HTTP_METHODS.contains(&call.name.as_str()))
                .then(|| call.name.clone())
        })
        .collect()
}

pub(super) fn is_env_lookup_line(line: &str) -> bool {
    line.contains("os.getenv(") || line.contains("os.environ.get(") || line.contains("os.environ[")
}

pub(super) fn env_lookup_has_default(line: &str) -> bool {
    (line.contains("os.getenv(") || line.contains("os.environ.get("))
        && line
            .split_once('(')
            .and_then(|(_, tail)| tail.split_once(')'))
            .is_some_and(|(args, _)| args.contains(','))
        || line.contains(" or ")
}

pub(super) fn has_validation_markers(function: &ParsedFunction, lower_body: &str) -> bool {
    !function.python_evidence().exception_handlers.is_empty()
        || lower_body.contains("if not ")
        || lower_body.contains("if len(")
        || lower_body.contains(" is none")
        || lower_body.contains("validate")
        || lower_body.contains("assert ")
        || lower_body.contains("raise ")
        || lower_body.contains("schema")
        || lower_body.contains("pydantic")
}
