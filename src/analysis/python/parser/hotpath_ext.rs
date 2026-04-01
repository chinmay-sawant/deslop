use tree_sitter::Node;

/// Detect `list(x)`, `x[:]`, `x.copy()`, `dict(x)`, `{**x}`, `set(x)` inside loops
/// on the same source — allocation churn.
pub(super) fn collect_copy_in_loop_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_copy_in_loop(body_node, source, false, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_copy_in_loop(node: Node<'_>, source: &str, inside_loop: bool, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    let next_inside_loop =
        inside_loop || matches!(node.kind(), "for_statement" | "while_statement");

    if next_inside_loop && let Some(text) = source.get(node.byte_range()) {
        let trimmed = text.trim();
        // list(source) or dict(source) or set(source) inside loop
        if (trimmed.starts_with("list(")
            || trimmed.starts_with("dict(")
            || trimmed.starts_with("set("))
            && node.kind() == "call"
        {
            lines.push(node.start_position().row + 1);
        }
        // x.copy() inside loop
        if trimmed.ends_with(".copy()") && node.kind() == "call" {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_copy_in_loop(child, source, next_inside_loop, lines);
    }
}

/// Detect `urlparse(...)`, `urlsplit(...)`, `Path.resolve()`, `Path.expanduser()`,
/// `os.path.abspath(...)` on invariant values inside loops.
pub(super) fn collect_invariant_call_in_loop_lines(
    body_node: Node<'_>,
    source: &str,
) -> Vec<(String, usize)> {
    let mut results = Vec::new();
    visit_invariant_call_in_loop(body_node, source, false, &mut results);
    results
}

const INVARIANT_CALLS: &[&str] = &[
    "urlparse(",
    "urlsplit(",
    "parse.urlparse(",
    "parse.urlsplit(",
    "urllib.parse.urlparse(",
    "urllib.parse.urlsplit(",
    ".resolve()",
    ".expanduser()",
    "os.path.abspath(",
    "os.path.realpath(",
    "os.path.expanduser(",
    "codecs.lookup(",
    "locale.getlocale(",
];

fn visit_invariant_call_in_loop(
    node: Node<'_>,
    source: &str,
    inside_loop: bool,
    results: &mut Vec<(String, usize)>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    let next_inside_loop =
        inside_loop || matches!(node.kind(), "for_statement" | "while_statement");

    if next_inside_loop
        && node.kind() == "call"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        for pattern in INVARIANT_CALLS {
            if trimmed.contains(pattern) {
                let callee = pattern.trim_end_matches('(').trim_end_matches(')');
                results.push((callee.to_string(), node.start_position().row + 1));
                break;
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_invariant_call_in_loop(child, source, next_inside_loop, results);
    }
}

/// Detect `.index(value)` calls inside loops.
pub(super) fn collect_index_in_loop_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_index_in_loop(body_node, source, false, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_index_in_loop(node: Node<'_>, source: &str, inside_loop: bool, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    let next_inside_loop =
        inside_loop || matches!(node.kind(), "for_statement" | "while_statement");

    if next_inside_loop
        && node.kind() == "call"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if trimmed.contains(".index(") && !trimmed.starts_with('#') {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_index_in_loop(child, source, next_inside_loop, lines);
    }
}

/// Detect `.append(...)` followed by `.sort()` in the same loop body.
pub(super) fn collect_append_sort_in_loop_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_append_sort_in_loop(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_append_sort_in_loop(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if matches!(node.kind(), "for_statement" | "while_statement")
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if trimmed.contains(".append(") && trimmed.contains(".sort(") {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_append_sort_in_loop(child, source, lines);
    }
}

/// Detect `"sep".join([list comprehension])` where a generator would avoid allocation.
pub(super) fn collect_join_list_comp_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_join_list_comp(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_join_list_comp(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if trimmed.contains(".join(") {
            // Check if the argument is a list comprehension [... for ... in ...]
            if let Some(join_idx) = trimmed.find(".join(") {
                let after_join = &trimmed[join_idx + 6..];
                if after_join.starts_with('[')
                    && after_join.contains(" for ")
                    && after_join.contains(" in ")
                {
                    lines.push(node.start_position().row + 1);
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_join_list_comp(child, source, lines);
    }
}

/// Detect repeated `d[key]` or `d.get(key)` for the same key within a function.
/// Heuristic: look for subscript patterns that repeat.
pub(super) fn collect_repeated_subscript_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    let mut seen_gets: std::collections::BTreeMap<String, Vec<usize>> =
        std::collections::BTreeMap::new();
    visit_repeated_subscripts(body_node, source, &mut seen_gets);

    for key_lines in seen_gets.values() {
        if key_lines.len() >= 3 {
            for line in key_lines {
                lines.push(*line);
            }
        }
    }
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_repeated_subscripts(
    node: Node<'_>,
    source: &str,
    seen: &mut std::collections::BTreeMap<String, Vec<usize>>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        // Detect d.get(key) pattern
        if trimmed.contains(".get(")
            && let Some(get_idx) = trimmed.find(".get(")
        {
            let receiver = &trimmed[..get_idx];
            let after = &trimmed[get_idx + 5..];
            if let Some(end) = after.find([',', ')']) {
                let key = after[..end].trim();
                if !key.is_empty() {
                    let full_key = format!("{receiver}.get({key})");
                    seen.entry(full_key)
                        .or_default()
                        .push(node.start_position().row + 1);
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_repeated_subscripts(child, source, seen);
    }
}

fn should_skip_nested_scope(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_definition" | "class_definition")
}
