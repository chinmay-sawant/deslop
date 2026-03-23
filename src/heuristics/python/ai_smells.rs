use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity, SymbolKind};

pub(super) fn textbook_docstring_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    if function.is_test_function || function.fingerprint.receiver_type.is_some() {
        return Vec::new();
    }

    let Some(doc_comment) = function.doc_comment.as_deref() else {
        return Vec::new();
    };
    if function.fingerprint.line_count > 10
        || function.fingerprint.complexity_score > 1
        || function.fingerprint.call_count > 2
    {
        return Vec::new();
    }

    let doc_word_count = doc_comment.split_whitespace().count();
    if doc_word_count < 10 || doc_comment.lines().count() < 2 {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "textbook_docstring_small_helper".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: function.fingerprint.start_line,
        end_line: function.fingerprint.start_line,
        message: format!(
            "function {} has a textbook-style docstring for a very small helper",
            function.fingerprint.name
        ),
        evidence: vec![
            format!("doc_word_count={doc_word_count}"),
            format!("line_count={}", function.fingerprint.line_count),
            format!("complexity_score={}", function.fingerprint.complexity_score),
        ],
    }]
}

pub(super) fn mixed_naming_convention_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut snake_names = Vec::new();
    let mut camel_names = Vec::new();

    for symbol in &file.symbols {
        if !matches!(symbol.kind, SymbolKind::Function | SymbolKind::Method) {
            continue;
        }

        match naming_style(&symbol.name) {
            Some(NamingStyle::Snake) => snake_names.push((symbol.line, symbol.name.clone())),
            Some(NamingStyle::Camel) => camel_names.push((symbol.line, symbol.name.clone())),
            None => {}
        }
    }

    if snake_names.len() < 2 || camel_names.len() < 2 {
        return Vec::new();
    }

    let start_line = snake_names[0].0.min(camel_names[0].0);
    vec![Finding {
        rule_id: "mixed_naming_conventions".to_string(),
        severity: Severity::Info,
        path: file.path.clone(),
        function_name: None,
        start_line,
        end_line: start_line,
        message: "file mixes snake_case and camelCase function naming conventions"
            .to_string(),
        evidence: vec![
            format!(
                "snake_case examples: {}",
                snake_names
                    .iter()
                    .take(3)
                    .map(|(_, name)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            format!(
                "camelCase examples: {}",
                camel_names
                    .iter()
                    .take(3)
                    .map(|(_, name)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ],
    }]
}

enum NamingStyle {
    Snake,
    Camel,
}

fn naming_style(name: &str) -> Option<NamingStyle> {
    if name.starts_with('_') || name.chars().all(|character| !character.is_ascii_alphabetic()) {
        return None;
    }
    if name.chars().all(|character| !character.is_ascii_uppercase()) && name.contains('_') {
        return Some(NamingStyle::Snake);
    }

    let mut characters = name.chars();
    let first = characters.next()?;
    if first.is_ascii_lowercase() && characters.any(|character| character.is_ascii_uppercase()) {
        return Some(NamingStyle::Camel);
    }

    None
}