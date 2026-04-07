use tree_sitter::Node;

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
    let all_parameters_annotated = parameters_text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
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
