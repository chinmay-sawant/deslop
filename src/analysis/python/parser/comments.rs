use tree_sitter::Node;

use crate::analysis::CommentSummary;

pub(super) fn collect_comment_summaries(source: &str) -> Vec<CommentSummary> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim();
            if trimmed.starts_with("#!") {
                return None;
            }

            let comment_start = line.find('#')?;
            let text = line.get(comment_start + 1..)?.trim();
            (!text.is_empty()).then(|| CommentSummary {
                line: index + 1,
                text: text.to_string(),
            })
        })
        .collect()
}

pub(super) fn extract_docstring(body_node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = body_node.walk();
    let first_statement = body_node.named_children(&mut cursor).next()?;
    if first_statement.kind() != "expression_statement" {
        return None;
    }

    let expression = first_statement.named_child(0)?;
    if !is_string_node(expression) {
        return None;
    }

    string_literal_value(expression, source)
}

pub(super) fn string_literal_value(node: Node<'_>, source: &str) -> Option<String> {
    let text = source.get(node.byte_range())?;
    parse_string_literal_text(text)
}

pub(super) fn parse_string_literal_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let quote_index = trimmed.find(['\'', '"'])?;
    let prefix = trimmed.get(..quote_index)?;
    if !prefix
        .chars()
        .all(|character| matches!(character, 'r' | 'R' | 'u' | 'U' | 'b' | 'B' | 'f' | 'F'))
    {
        return None;
    }

    let quoted = trimmed.get(quote_index..)?;
    if let Some(value) = strip_quoted(quoted, "\"\"\"") {
        return Some(value);
    }
    if let Some(value) = strip_quoted(quoted, "'''") {
        return Some(value);
    }
    if let Some(value) = strip_single_quoted(quoted, '"') {
        return Some(value);
    }
    strip_single_quoted(quoted, '\'')
}

fn is_string_node(node: Node<'_>) -> bool {
    matches!(node.kind(), "string" | "concatenated_string")
}

fn strip_quoted(text: &str, quote: &str) -> Option<String> {
    if !text.starts_with(quote) {
        return None;
    }

    let suffix = text.get(quote.len()..)?;
    let end_index = suffix.find(quote)?;
    suffix.get(..end_index).map(ToOwned::to_owned)
}

fn strip_single_quoted(text: &str, quote: char) -> Option<String> {
    if !text.starts_with(quote) {
        return None;
    }

    let mut escaped = false;
    for (index, character) in text.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }

        if character == '\\' {
            escaped = true;
            continue;
        }

        if character == quote {
            return text.get(1..index).map(ToOwned::to_owned);
        }
    }

    None
}
