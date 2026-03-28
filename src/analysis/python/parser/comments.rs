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

            let comment_start = comment_start_in_line(line)?;
            let text = line.get(comment_start + 1..)?.trim();
            (!text.is_empty()).then(|| CommentSummary {
                line: index + 1,
                text: text.to_string(),
            })
        })
        .collect()
}

/// Find the byte offset of a `#` that starts a comment, skipping over `#`
/// characters that appear inside single- or double-quoted string literals
/// (including triple-quoted). Returns `None` when no comment `#` exists.
fn comment_start_in_line(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut in_string: Option<u8> = None; // quote char we're inside
    let mut triple = false;

    while i < len {
        let ch = bytes[i];

        if let Some(q) = in_string {
            if ch == b'\\' {
                // skip the escaped character
                i += 2;
                continue;
            }
            if triple {
                if ch == q && i + 2 < len && bytes[i + 1] == q && bytes[i + 2] == q {
                    in_string = None;
                    triple = false;
                    i += 3;
                    continue;
                }
            } else if ch == q {
                in_string = None;
            }
        } else {
            match ch {
                b'\'' | b'"' => {
                    let q = ch;
                    if i + 2 < len && bytes[i + 1] == q && bytes[i + 2] == q {
                        in_string = Some(q);
                        triple = true;
                        i += 3;
                        continue;
                    }
                    in_string = Some(q);
                }
                b'#' => return Some(i),
                _ => {}
            }
        }
        i += 1;
    }
    None
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
