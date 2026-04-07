use tree_sitter::Node;

use crate::analysis::BlockFingerprint;

pub(crate) fn collect_validation_signature(
    body_node: Node<'_>,
    source: &str,
) -> Option<BlockFingerprint> {
    let mut signature_parts = Vec::new();
    let mut cursor = body_node.walk();

    for child in body_node.named_children(&mut cursor) {
        if child.kind() != "if_statement" {
            if signature_parts.is_empty() {
                continue;
            }
            break;
        }

        let text = source.get(child.byte_range())?.trim();
        if !(text.contains("return ") || text.contains("raise ") || text.ends_with("return")) {
            break;
        }

        signature_parts.push(normalize_shape(text));
        if signature_parts.len() >= 4 {
            break;
        }
    }

    (!signature_parts.is_empty()).then_some(BlockFingerprint {
        line: body_node.start_position().row + 1,
        signature: signature_parts.join(" | "),
    })
}

pub(crate) fn collect_exception_block_signatures(
    body_node: Node<'_>,
    source: &str,
) -> Vec<BlockFingerprint> {
    let mut signatures = Vec::new();
    visit_exception_signatures(body_node, source, &mut signatures);
    signatures.sort_by(|left, right| left.line.cmp(&right.line));
    signatures.dedup_by(|left, right| left.line == right.line && left.signature == right.signature);
    signatures
}

pub(crate) fn normalize_body(body_node: Node<'_>, source: &str) -> String {
    source
        .get(body_node.byte_range())
        .map(normalize_shape)
        .unwrap_or_default()
}

fn visit_exception_signatures(
    node: Node<'_>,
    source: &str,
    signatures: &mut Vec<BlockFingerprint>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "except_clause"
        && let Some(text) = source.get(node.byte_range())
    {
        let normalized_lines = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .take(3)
            .collect::<Vec<_>>();
        if !normalized_lines.is_empty() {
            signatures.push(BlockFingerprint {
                line: node.start_position().row + 1,
                signature: normalize_shape(&normalized_lines.join(" ")),
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_exception_signatures(child, source, signatures);
    }
}

fn normalize_shape(text: &str) -> String {
    let mut output = String::new();
    let mut token = String::new();
    let mut in_string = false;
    let mut quote_char = '\0';

    for character in text.chars() {
        if in_string {
            if character == quote_char {
                output.push_str("STR");
                in_string = false;
            }
            continue;
        }

        if matches!(character, '\'' | '"') {
            flush_shape_token(&mut output, &mut token);
            in_string = true;
            quote_char = character;
            continue;
        }

        if character.is_ascii_alphanumeric() || character == '_' {
            token.push(character);
            continue;
        }

        flush_shape_token(&mut output, &mut token);
        if !character.is_whitespace() {
            output.push(character);
        }
    }

    flush_shape_token(&mut output, &mut token);
    output
}

fn flush_shape_token(output: &mut String, token: &mut String) {
    if token.is_empty() {
        return;
    }

    let replacement = if token.chars().all(|character| character.is_ascii_digit()) {
        "NUM"
    } else if is_python_keyword(token) {
        token.as_str()
    } else {
        "ID"
    };
    output.push_str(replacement);
    token.clear();
}

fn is_python_keyword(token: &str) -> bool {
    matches!(
        token,
        "and"
            | "as"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "elif"
            | "else"
            | "except"
            | "False"
            | "finally"
            | "for"
            | "from"
            | "if"
            | "import"
            | "in"
            | "is"
            | "None"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "True"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}

fn should_skip_nested_scope(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_definition" | "class_definition")
}
