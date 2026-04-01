use std::collections::BTreeMap;

use tree_sitter::Node;

/// Detect `sorted(x)[0]` or `sorted(x)[-1]` patterns.
pub(super) fn collect_sorted_first_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_sorted_first(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_sorted_first(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "subscript"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if (trimmed.ends_with("[0]") || trimmed.ends_with("[-1]")) && trimmed.starts_with("sorted(")
        {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_sorted_first(child, source, lines);
    }
}

/// Detect `len([x for x in ... if ...])` patterns — list built only for length.
pub(super) fn collect_len_comprehension_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_len_comprehension(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_len_comprehension(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if trimmed.starts_with("len(")
            && trimmed.ends_with(')')
            && (trimmed.contains(" for ") && trimmed.contains(" in "))
        {
            // Verify the argument is a list comprehension
            let inner = &trimmed[4..trimmed.len() - 1];
            if inner.starts_with('[') && inner.ends_with(']') {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_len_comprehension(child, source, lines);
    }
}

/// Detect `x in [literal, literal, ...]` membership on list literals.
pub(super) fn collect_in_list_literal_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_in_list_literal(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_in_list_literal(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    // Match comparison operators: `x in [...]`
    if node.kind() == "comparison_operator"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        // Look for `<expr> in [<elements>]` but not `for <x> in [...]`
        if let Some(in_idx) = trimmed.find(" in [") {
            let before_in = &trimmed[..in_idx];
            // Make sure it's not a `for x in [...]` — check no `for` prefix
            if !before_in.trim_start().starts_with("for ")
                && !before_in.trim_start().starts_with("not ")
            {
                let after_in = &trimmed[in_idx + 4..]; // skip " in "
                if after_in.starts_with('[') && after_in.ends_with(']') {
                    // Count elements — only flag if 3+ items (trivial lists don't matter)
                    let inner = &after_in[1..after_in.len() - 1];
                    let element_count = inner.split(',').count();
                    if element_count >= 3 {
                        lines.push(node.start_position().row + 1);
                    }
                }
            }
        }
        // Also check `<expr> not in [<elements>]`
        if let Some(in_idx) = trimmed.find(" not in [") {
            let after_not_in = &trimmed[in_idx + 8..]; // skip " not in "
            if after_not_in.starts_with('[') && after_not_in.ends_with(']') {
                let inner = &after_not_in[1..after_not_in.len() - 1];
                let element_count = inner.split(',').count();
                if element_count >= 3 {
                    lines.push(node.start_position().row + 1);
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_in_list_literal(child, source, lines);
    }
}

/// Detect `.startswith(a) or .startswith(b)` chains where a tuple form is better.
pub(super) fn collect_startswith_chain_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_startswith_chains(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_startswith_chains(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "boolean_operator"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        // Count .startswith(...) or .endswith(...) calls joined by `or`
        let parts: Vec<&str> = trimmed.split(" or ").collect();
        if parts.len() >= 2 {
            let starts_count = parts
                .iter()
                .filter(|part| {
                    let p = part.trim();
                    p.contains(".startswith(") || p.contains(".endswith(")
                })
                .count();
            if starts_count >= 2 {
                // Verify they are on the same receiver
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_startswith_chains(child, source, lines);
    }
}

/// Detect `for i, x in enumerate(range(len(collection)))` anti-patterns.
pub(super) fn collect_enumerate_range_len_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_enumerate_range_len(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_enumerate_range_len(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "for_statement"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        // Detect `for ... in enumerate(range(len(...))):` and `for ... in range(len(...)):`
        if let Some(in_idx) = trimmed.find(" in ") {
            let iterable = &trimmed[in_idx + 4..];
            if iterable.starts_with("enumerate(range(len(") || iterable.starts_with("range(len(") {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_enumerate_range_len(child, source, lines);
    }
}

/// Detect `list(d.keys())`, `list(d.values())`, `list(d.items())` inside loops.
pub(super) fn collect_dict_materialization_in_loop_lines(
    body_node: Node<'_>,
    source: &str,
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_dict_materialization_in_loop(body_node, source, false, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_dict_materialization_in_loop(
    node: Node<'_>,
    source: &str,
    inside_loop: bool,
    lines: &mut Vec<usize>,
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
        if trimmed.starts_with("list(")
            && (trimmed.contains(".keys()")
                || trimmed.contains(".values()")
                || trimmed.contains(".items()"))
        {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_dict_materialization_in_loop(child, source, next_inside_loop, lines);
    }
}

/// Detect repeated calls to the same callee with the same first argument in one function.
/// Returns (callee_key, arg_key, lines) triples for duplicates.
pub(super) fn collect_repeated_call_same_arg_lines(
    body_node: Node<'_>,
    source: &str,
    callees: &[&str],
) -> Vec<(String, usize)> {
    let mut call_map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    visit_repeated_calls(body_node, source, callees, &mut call_map);

    let mut results = Vec::new();
    for (key, call_lines) in &call_map {
        if call_lines.len() >= 2 {
            for line in call_lines {
                results.push((key.clone(), *line));
            }
        }
    }
    results
}

fn visit_repeated_calls(
    node: Node<'_>,
    source: &str,
    callees: &[&str],
    call_map: &mut BTreeMap<String, Vec<usize>>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        for &callee in callees {
            if trimmed.starts_with(&format!("{callee}(")) {
                // Extract first argument (simplified: up to first comma or closing paren)
                let after_open = &trimmed[callee.len() + 1..];
                if let Some(end) = after_open.find([',', ')']) {
                    let first_arg = after_open[..end].trim();
                    if !first_arg.is_empty() && looks_like_binding(first_arg) {
                        let key = format!("{callee}({first_arg})");
                        call_map
                            .entry(key)
                            .or_default()
                            .push(node.start_position().row + 1);
                    }
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_repeated_calls(child, source, callees, call_map);
    }
}

/// Detect `file.readlines()` followed by iteration (readlines then iterate).
pub(super) fn collect_readlines_then_iterate_lines(
    body_node: Node<'_>,
    source: &str,
) -> Vec<usize> {
    let mut lines = Vec::new();
    // Look for readlines() calls and flag them if the result is iterated
    visit_readlines_iterate(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_readlines_iterate(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    // Detect patterns like:
    //   lines = f.readlines()
    //   for line in lines:
    // Or: for line in f.readlines():
    if node.kind() == "for_statement"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if let Some(in_idx) = trimmed.find(" in ") {
            let iterable_part = &trimmed[in_idx + 4..];
            if iterable_part.contains(".readlines()") {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    // Also detect: x = f.readlines() (standalone call suggesting full materialization)
    if matches!(node.kind(), "assignment" | "annotated_assignment")
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if trimmed.contains(".readlines()") && trimmed.contains('=') {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_readlines_iterate(child, source, lines);
    }
}

/// Detect `f.read().splitlines()` or `f.read().split('\n')`.
pub(super) fn collect_read_splitlines_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_read_splitlines(body_node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_read_splitlines(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    if (node.kind() == "call" || matches!(node.kind(), "assignment" | "expression_statement"))
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if trimmed.contains(".read().splitlines()")
            || trimmed.contains(".read().split('\\n')")
            || trimmed.contains(".read().split(\"\\n\")")
        {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_read_splitlines(child, source, lines);
    }
}

/// Detect `file.write(...)` inside tight loops without buffering.
pub(super) fn collect_write_in_loop_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_write_in_loop(body_node, source, false, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_write_in_loop(node: Node<'_>, source: &str, inside_loop: bool, lines: &mut Vec<usize>) {
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
        if trimmed.contains(".write(")
            && !trimmed.contains("writer.write")
            && !trimmed.contains("csv_writer")
        {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_write_in_loop(child, source, next_inside_loop, lines);
    }
}

/// Detect multiple `open(same_path, ...)` calls in one function.
pub(super) fn collect_repeated_open_lines(
    body_node: Node<'_>,
    source: &str,
) -> Vec<(String, usize)> {
    let mut open_calls: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    visit_repeated_opens(body_node, source, &mut open_calls);

    let mut results = Vec::new();
    for (path_arg, call_lines) in &open_calls {
        if call_lines.len() >= 2 {
            for line in call_lines {
                results.push((path_arg.clone(), *line));
            }
        }
    }
    results
}

fn visit_repeated_opens(
    node: Node<'_>,
    source: &str,
    open_calls: &mut BTreeMap<String, Vec<usize>>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    if node.kind() == "call"
        && let Some(text) = source.get(node.byte_range())
    {
        let trimmed = text.trim();
        if let Some(after_open) = trimmed.strip_prefix("open(") {
            // Extract first argument
            if let Some(end) = after_open.find([',', ')']) {
                let first_arg = after_open[..end].trim();
                if !first_arg.is_empty() && looks_like_binding(first_arg) {
                    open_calls
                        .entry(first_arg.to_string())
                        .or_default()
                        .push(node.start_position().row + 1);
                }
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_repeated_opens(child, source, open_calls);
    }
}

/// Detect `csv.writer().writerow()` with flush per row or `writer.flush()` inside loops.
pub(super) fn collect_csv_flush_per_row_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_csv_flush_per_row(body_node, source, false, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_csv_flush_per_row(
    node: Node<'_>,
    source: &str,
    inside_loop: bool,
    lines: &mut Vec<usize>,
) {
    if should_skip_nested_scope(node) {
        return;
    }

    let next_inside_loop =
        inside_loop || matches!(node.kind(), "for_statement" | "while_statement");

    if next_inside_loop && let Some(text) = source.get(node.byte_range()) {
        let trimmed = text.trim();
        if trimmed.contains(".flush()") && !trimmed.starts_with('#') {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_csv_flush_per_row(child, source, next_inside_loop, lines);
    }
}

/// Detect `re.compile(...)` inside loops or functions that look like handlers.
pub(super) fn collect_regex_in_hotpath_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_regex_compile(body_node, source, false, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_regex_compile(node: Node<'_>, source: &str, inside_loop: bool, lines: &mut Vec<usize>) {
    if should_skip_nested_scope(node) {
        return;
    }

    let next_inside_loop =
        inside_loop || matches!(node.kind(), "for_statement" | "while_statement");

    if let Some(text) = source.get(node.byte_range()) {
        let trimmed = text.trim();
        // Flag re.compile inside loops specifically
        if next_inside_loop
            && (trimmed.starts_with("re.compile(") || trimmed.starts_with("regex.compile("))
        {
            lines.push(node.start_position().row + 1);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_regex_compile(child, source, next_inside_loop, lines);
    }
}

fn looks_like_binding(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Accept identifiers (potentially with dots for attribute access)
    // Reject string literals, function calls with parens, etc.
    if trimmed.starts_with('"') || trimmed.starts_with('\'') || trimmed.starts_with('(') {
        return false;
    }
    trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '[' || c == ']')
}

fn should_skip_nested_scope(node: Node<'_>) -> bool {
    matches!(node.kind(), "function_definition" | "class_definition")
}
