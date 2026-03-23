use tree_sitter::Node;

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
    let prefix = &trimmed[..quote_index];
    if !prefix
        .chars()
        .all(|character| matches!(character, 'r' | 'R' | 'u' | 'U' | 'b' | 'B' | 'f' | 'F'))
    {
        return None;
    }

    let quoted = &trimmed[quote_index..];
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

    let end_index = text[quote.len()..].find(quote)?;
    Some(text[quote.len()..quote.len() + end_index].to_string())
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
            return Some(text[1..index].to_string());
        }
    }

    None
}
