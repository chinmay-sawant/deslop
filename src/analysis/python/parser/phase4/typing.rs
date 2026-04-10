use tree_sitter::Node;

fn split_top_level_commas(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;

    for (index, ch) in text.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                parts.push(text[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    let tail = text[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }

    parts
}

pub(crate) fn has_complete_type_hints(function_node: Node<'_>, source: &str) -> bool {
    let Some(parameters_node) = function_node.child_by_field_name("parameters") else {
        return false;
    };
    let Some(function_text) = source.get(function_node.byte_range()) else {
        return false;
    };
    let Some(parameters_text) = source.get(parameters_node.byte_range()) else {
        return false;
    };

    let has_return_annotation = function_text.contains(") ->");
    let all_parameters_annotated = split_top_level_commas(
        parameters_text
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')'),
    )
    .into_iter()
    .filter(|entry| !entry.is_empty() && *entry != "/" && *entry != "*")
    .filter(|entry| {
        let trimmed = entry.trim_start_matches('*').trim();
        !matches!(trimmed, "self" | "cls")
    })
    .all(|entry| {
        let trimmed = entry.trim_start_matches('*').trim();
        let subject = trimmed.split('=').next().unwrap_or(trimmed).trim();
        subject.contains(':')
    });

    has_return_annotation && all_parameters_annotated
}

pub(crate) fn parameter_flags(function_node: Node<'_>, source: &str) -> (bool, bool) {
    let Some(parameters_node) = function_node.child_by_field_name("parameters") else {
        return (false, false);
    };
    let Some(parameters_text) = source.get(parameters_node.byte_range()) else {
        return (false, false);
    };

    let mut has_varargs = false;
    let mut has_kwargs = false;
    for entry in parameters_text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        if entry.starts_with("**") {
            has_kwargs = true;
        } else if entry.starts_with('*') && entry != "*" {
            has_varargs = true;
        }
    }

    (has_varargs, has_kwargs)
}
