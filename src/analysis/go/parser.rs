use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use tree_sitter::{Node, Parser};

use crate::analysis::go::fingerprint::build_function_fingerprint;
use crate::analysis::{
    CallSite, DeclaredSymbol, FormattedErrorCall, ImportSpec, ParsedFile, ParsedFunction,
};
use crate::model::SymbolKind;

pub(super) fn parse_file(path: &Path, source: &str) -> Result<ParsedFile> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_go::LANGUAGE.into())
        .map_err(|error| anyhow!(error.to_string()))
        .context("failed to configure Go parser")?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("tree-sitter returned no parse tree"))?;

    let root = tree.root_node();
    let package_name = find_package_name(root, source);
    let imports = collect_imports(root, source);
    let symbols = collect_symbols(root, source);
    let functions = collect_functions(root, source, &imports);

    Ok(ParsedFile {
        path: path.to_path_buf(),
        package_name,
        syntax_error: root.has_error(),
        byte_size: source.len(),
        functions,
        imports,
        symbols,
    })
}

fn collect_functions(root: Node<'_>, source: &str, imports: &[ImportSpec]) -> Vec<ParsedFunction> {
    let mut functions = Vec::new();
    visit_for_functions(root, source, imports, &mut functions);
    functions.sort_by(|left, right| {
        left.fingerprint
            .start_line
            .cmp(&right.fingerprint.start_line)
            .then(left.fingerprint.name.cmp(&right.fingerprint.name))
    });
    functions
}

fn visit_for_functions(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
    functions: &mut Vec<ParsedFunction>,
) {
    if matches!(node.kind(), "function_declaration" | "method_declaration") {
        if let Some(parsed_function) = parse_function_node(node, source, imports) {
            functions.push(parsed_function);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_functions(child, source, imports, functions);
    }
}

fn parse_function_node(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Option<ParsedFunction> {
    let body_node = node.child_by_field_name("body")?;
    let calls = collect_calls(body_node, source);
    let type_assertion_count = count_descendants(body_node, "type_assertion_expression");
    let has_context_parameter = function_has_context_parameter(node, source, imports);
    let doc_comment = extract_doc_comment(source, node.start_position().row);
    let dropped_error_lines = collect_dropped_error_lines(body_node, source);
    let panic_on_error_lines = collect_panic_on_error_lines(body_node, source);
    let errorf_calls = collect_errorf_calls(body_node, source);
    let goroutine_launch_lines = collect_goroutine_launch_lines(body_node);
    let sleep_in_loop_lines = collect_sleep_in_loop_lines(body_node, source, imports);
    let string_concat_in_loop_lines = collect_string_concat_in_loop_lines(body_node, source);
    let receiver_type = node
        .child_by_field_name("receiver")
        .and_then(|receiver| extract_receiver_type(receiver, source));
    let fingerprint = build_function_fingerprint(
        node,
        source,
        receiver_type,
        type_assertion_count,
        calls.len(),
    )?;

    Some(ParsedFunction {
        fingerprint,
        calls,
        has_context_parameter,
        doc_comment,
        dropped_error_lines,
        panic_on_error_lines,
        errorf_calls,
        goroutine_launch_lines,
        sleep_in_loop_lines,
        string_concat_in_loop_lines,
    })
}

fn collect_calls(body_node: Node<'_>, source: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    visit_for_calls(body_node, source, &mut calls);
    calls
}

fn visit_for_calls(node: Node<'_>, source: &str, calls: &mut Vec<CallSite>) {
    if node.kind() == "call_expression" {
        if let Some(function_node) = node.child_by_field_name("function") {
            if let Some((receiver, name)) = extract_call_target(function_node, source) {
                calls.push(CallSite {
                    receiver,
                    name,
                    line: node.start_position().row + 1,
                });
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_calls(child, source, calls);
    }
}

fn extract_call_target(function_node: Node<'_>, source: &str) -> Option<(Option<String>, String)> {
    let text = source.get(function_node.byte_range())?.trim();
    if text.is_empty() {
        return None;
    }

    if let Some((receiver, name)) = text.rsplit_once('.') {
        return Some((Some(receiver.trim().to_string()), name.trim().to_string()));
    }

    Some((None, text.to_string()))
}

fn collect_imports(root: Node<'_>, source: &str) -> Vec<ImportSpec> {
    let mut imports = Vec::new();
    visit_for_imports(root, source, &mut imports);
    imports.sort_by(|left, right| {
        left.alias
            .cmp(&right.alias)
            .then(left.path.cmp(&right.path))
    });
    imports
}

fn visit_for_imports(node: Node<'_>, source: &str, imports: &mut Vec<ImportSpec>) {
    if node.kind() == "import_spec" {
        if let Some(import_spec) = parse_import_spec(node, source) {
            imports.push(import_spec);
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_imports(child, source, imports);
    }
}

fn parse_import_spec(node: Node<'_>, source: &str) -> Option<ImportSpec> {
    let text = source.get(node.byte_range())?.trim();
    let mut parts = text.split_whitespace().collect::<Vec<_>>();
    let path_literal = parts.pop()?;
    let path = path_literal.trim_matches('"').to_string();
    let alias = parts
        .first()
        .map(|alias| alias.to_string())
        .unwrap_or_else(|| package_alias_from_import_path(&path));

    Some(ImportSpec { alias, path })
}

fn collect_symbols(root: Node<'_>, source: &str) -> Vec<DeclaredSymbol> {
    let mut symbols = Vec::new();
    visit_for_symbols(root, source, &mut symbols);
    symbols.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    symbols
}

fn visit_for_symbols(node: Node<'_>, source: &str, symbols: &mut Vec<DeclaredSymbol>) {
    match node.kind() {
        "function_declaration" => {
            if let Some(symbol) = parse_function_symbol(node, source) {
                symbols.push(symbol);
            }
        }
        "method_declaration" => {
            if let Some(symbol) = parse_method_symbol(node, source) {
                symbols.push(symbol);
            }
        }
        "type_spec" => {
            if let Some(symbol) = parse_type_symbol(node, source) {
                symbols.push(symbol);
            }
        }
        "var_spec" => {
            symbols.extend(parse_package_var_symbols(node, source));
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_symbols(child, source, symbols);
    }
}

fn parse_package_var_symbols(node: Node<'_>, source: &str) -> Vec<DeclaredSymbol> {
    if !is_package_scope(node) {
        return Vec::new();
    }

    let Some(name_node) = find_var_name_node(node) else {
        return Vec::new();
    };
    let names = collect_identifiers(name_node, source);
    if names.is_empty() {
        return Vec::new();
    }

    let is_function_typed = node
        .child_by_field_name("type")
        .is_some_and(|type_node| type_node.kind() == "function_type");

    if is_function_typed {
        return names
            .into_iter()
            .map(|(name, line)| DeclaredSymbol {
                name,
                kind: SymbolKind::Function,
                receiver_type: None,
                line,
            })
            .collect();
    }

    let Some(value_node) = find_var_value_node(node) else {
        return Vec::new();
    };
    let values = collect_expression_nodes(value_node);

    names
        .into_iter()
        .enumerate()
        .filter_map(|(index, (name, line))| {
            let value = values.get(index)?;
            is_callable_var_value(*value).then_some(DeclaredSymbol {
                name,
                kind: SymbolKind::Function,
                receiver_type: None,
                line,
            })
        })
        .collect()
}

fn is_package_scope(node: Node<'_>) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "function_declaration" | "method_declaration" | "func_literal" => return false,
            "source_file" => return true,
            _ => current = parent.parent(),
        }
    }

    false
}

fn find_var_name_node(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("name")
        .or_else(|| first_named_child_of_kind(node, "identifier_list"))
        .or_else(|| first_named_child_of_kind(node, "identifier"))
}

fn find_var_value_node(node: Node<'_>) -> Option<Node<'_>> {
    node.child_by_field_name("value")
        .or_else(|| first_named_child_of_kind(node, "expression_list"))
        .or_else(|| {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find(|child| is_expression_node_kind(child.kind()))
        })
}

fn first_named_child_of_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}

fn collect_identifiers(node: Node<'_>, source: &str) -> Vec<(String, usize)> {
    if node.kind() == "identifier" {
        return source
            .get(node.byte_range())
            .map(|name| vec![(name.to_string(), node.start_position().row + 1)])
            .unwrap_or_default();
    }

    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .filter(|child| child.kind() == "identifier")
        .filter_map(|child| {
            source
                .get(child.byte_range())
                .map(|name| (name.to_string(), child.start_position().row + 1))
        })
        .collect()
}

fn collect_expression_nodes(node: Node<'_>) -> Vec<Node<'_>> {
    if node.kind() != "expression_list" {
        return vec![node];
    }

    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

fn is_callable_var_value(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "identifier"
            | "selector_expression"
            | "func_literal"
            | "parenthesized_expression"
            | "index_expression"
            | "slice_expression"
    )
}

fn is_expression_node_kind(kind: &str) -> bool {
    matches!(
        kind,
        "identifier"
            | "selector_expression"
            | "func_literal"
            | "parenthesized_expression"
            | "call_expression"
            | "unary_expression"
            | "binary_expression"
            | "index_expression"
            | "slice_expression"
            | "type_assertion_expression"
            | "composite_literal"
            | "literal_value"
            | "int_literal"
            | "float_literal"
            | "imaginary_literal"
            | "rune_literal"
            | "raw_string_literal"
            | "interpreted_string_literal"
    )
}

fn parse_function_symbol(node: Node<'_>, source: &str) -> Option<DeclaredSymbol> {
    let name_node = node.child_by_field_name("name")?;
    Some(DeclaredSymbol {
        name: source.get(name_node.byte_range())?.to_string(),
        kind: SymbolKind::Function,
        receiver_type: None,
        line: node.start_position().row + 1,
    })
}

fn parse_method_symbol(node: Node<'_>, source: &str) -> Option<DeclaredSymbol> {
    let name_node = node.child_by_field_name("name")?;
    let receiver_type = node
        .child_by_field_name("receiver")
        .and_then(|receiver| extract_receiver_type(receiver, source));
    Some(DeclaredSymbol {
        name: source.get(name_node.byte_range())?.to_string(),
        kind: SymbolKind::Method,
        receiver_type,
        line: node.start_position().row + 1,
    })
}

fn parse_type_symbol(node: Node<'_>, source: &str) -> Option<DeclaredSymbol> {
    let name_node = node.child_by_field_name("name")?;
    let type_node = node.child_by_field_name("type")?;
    let kind = match type_node.kind() {
        "struct_type" => SymbolKind::Struct,
        "interface_type" => SymbolKind::Interface,
        _ => SymbolKind::Type,
    };

    Some(DeclaredSymbol {
        name: source.get(name_node.byte_range())?.to_string(),
        kind,
        receiver_type: None,
        line: node.start_position().row + 1,
    })
}

fn find_package_name(root: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        if child.kind() != "package_clause" {
            continue;
        }

        let mut package_cursor = child.walk();
        for package_child in child.named_children(&mut package_cursor) {
            if package_child.kind() == "package_identifier" || package_child.kind() == "identifier"
            {
                return source
                    .get(package_child.byte_range())
                    .map(ToOwned::to_owned);
            }
        }
    }

    None
}

fn extract_receiver_type(receiver_node: Node<'_>, source: &str) -> Option<String> {
    let text = source.get(receiver_node.byte_range())?;
    let sanitized = text
        .chars()
        .filter(|character| !matches!(character, '(' | ')' | '*' | ','))
        .collect::<String>();
    sanitized
        .split_whitespace()
        .last()
        .map(|receiver| receiver.to_string())
}

fn package_alias_from_import_path(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn count_descendants(node: Node<'_>, kind: &str) -> usize {
    let mut total = usize::from(node.kind() == kind);
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        total += count_descendants(child, kind);
    }
    total
}

fn function_has_context_parameter(node: Node<'_>, source: &str, imports: &[ImportSpec]) -> bool {
    let Some(parameters_node) = node.child_by_field_name("parameters") else {
        return false;
    };
    let Some(parameters_text) = source.get(parameters_node.byte_range()) else {
        return false;
    };

    imports
        .iter()
        .filter(|import| import.path == "context")
        .any(|import| parameters_text.contains(&format!("{}.Context", import.alias)))
}

fn collect_sleep_in_loop_lines(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_sleep_in_loop(body_node, source, imports, false, &mut lines);
    lines
}

fn visit_for_sleep_in_loop(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
    inside_loop: bool,
    lines: &mut Vec<usize>,
) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if next_inside_loop && node.kind() == "call_expression" {
        let function_node = node.child_by_field_name("function");
        if let Some(function_node) = function_node {
            let target = source.get(function_node.byte_range()).unwrap_or("").trim();
            if is_time_sleep_call(target, imports) {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_sleep_in_loop(child, source, imports, next_inside_loop, lines);
    }
}

fn is_time_sleep_call(target: &str, imports: &[ImportSpec]) -> bool {
    imports
        .iter()
        .filter(|import| import.path == "time")
        .any(|import| target == format!("{}.Sleep", import.alias))
}

fn collect_goroutine_launch_lines(body_node: Node<'_>) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_goroutine_launches(body_node, &mut lines);
    lines
}

fn visit_for_goroutine_launches(node: Node<'_>, lines: &mut Vec<usize>) {
    if node.kind() == "go_statement" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_goroutine_launches(child, lines);
    }
}

fn collect_string_concat_in_loop_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let string_variables = collect_explicit_string_variables(body_node, source);
    let mut lines = Vec::new();
    visit_for_string_concat_in_loop(body_node, source, &string_variables, false, &mut lines);
    lines
}

fn collect_explicit_string_variables(body_node: Node<'_>, source: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    visit_for_string_variables(body_node, source, &mut names);
    names
}

fn visit_for_string_variables(node: Node<'_>, source: &str, names: &mut BTreeSet<String>) {
    match node.kind() {
        "var_spec" => {
            let Some(type_node) = node.child_by_field_name("type") else {
                return;
            };
            if source
                .get(type_node.byte_range())
                .is_some_and(|text| text.trim() == "string")
            {
                if let Some(name_node) = find_var_name_node(node) {
                    for (name, _) in collect_identifiers(name_node, source) {
                        names.insert(name);
                    }
                }
            }
        }
        "short_var_declaration" | "assignment_statement" => {
            if let Some(text) = source.get(node.byte_range()) {
                if let Some((left, right)) = split_assignment(text) {
                    let left = left.trim();
                    if is_identifier_name(left) && contains_string_literal(right) {
                        names.insert(left.to_string());
                    }
                }
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_string_variables(child, source, names);
    }
}

fn visit_for_string_concat_in_loop(
    node: Node<'_>,
    source: &str,
    string_variables: &BTreeSet<String>,
    inside_loop: bool,
    lines: &mut Vec<usize>,
) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if next_inside_loop && node.kind() == "assignment_statement" {
        if let Some(text) = source.get(node.byte_range()) {
            if is_string_concat_assignment(text, string_variables) {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_string_concat_in_loop(child, source, string_variables, next_inside_loop, lines);
    }
}

fn is_string_concat_assignment(text: &str, string_variables: &BTreeSet<String>) -> bool {
    let compact = text.split_whitespace().collect::<String>();

    if let Some((left, right)) = compact.split_once("+=") {
        return is_identifier_name(left)
            && (string_variables.contains(left) || contains_string_literal(right));
    }

    let Some((left, right)) = compact.split_once('=') else {
        return false;
    };
    if !is_identifier_name(left) || !string_variables.contains(left) {
        return false;
    }

    right.starts_with(&format!("{left}+"))
        || right.contains(&format!("+\""))
        || right.contains("+`")
}

fn split_assignment(text: &str) -> Option<(&str, &str)> {
    text.split_once(":=").or_else(|| text.split_once('='))
}

fn is_identifier_name(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        && text
            .chars()
            .next()
            .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
}

fn contains_string_literal(text: &str) -> bool {
    text.contains('"') || text.contains('`')
}

fn extract_doc_comment(source: &str, function_start_row: usize) -> Option<String> {
    let lines = source.lines().collect::<Vec<_>>();
    if function_start_row == 0 || function_start_row > lines.len() {
        return None;
    }

    let mut index = function_start_row;
    let mut comment_lines = Vec::new();

    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.starts_with("//") {
            comment_lines.push(trimmed.trim_start_matches("//").trim().to_string());
            continue;
        }

        if trimmed.ends_with("*/") {
            let mut block_lines = vec![trimmed.to_string()];
            while index > 0 {
                index -= 1;
                block_lines.push(lines[index].trim().to_string());
                if lines[index].trim().starts_with("/*") {
                    break;
                }
            }
            block_lines.reverse();
            let normalized = block_lines
                .into_iter()
                .map(|line| {
                    line.trim_start_matches("/*")
                        .trim_end_matches("*/")
                        .trim_start_matches('*')
                        .trim()
                        .to_string()
                })
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();
            if normalized.is_empty() {
                return None;
            }
            return Some(normalized.join("\n"));
        }

        break;
    }

    if comment_lines.is_empty() {
        None
    } else {
        comment_lines.reverse();
        Some(comment_lines.join("\n"))
    }
}

fn collect_dropped_error_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_dropped_errors(body_node, source, &mut lines);
    lines
}

fn visit_for_dropped_errors(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if matches!(
        node.kind(),
        "assignment_statement" | "short_var_declaration"
    ) {
        if let Some(text) = source.get(node.byte_range()) {
            let compact = text.split_whitespace().collect::<String>();
            let drops_named_err = compact.starts_with("_=err")
                || compact.starts_with("_=ctx.Err()")
                || compact.contains(",_=err")
                || compact.contains(",_=ctx.Err()");
            if drops_named_err {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_dropped_errors(child, source, lines);
    }
}

fn collect_panic_on_error_lines(body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_panic_on_error(body_node, source, &mut lines);
    lines
}

fn visit_for_panic_on_error(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if node.kind() == "if_statement" {
        let condition = node
            .child_by_field_name("condition")
            .and_then(|condition| source.get(condition.byte_range()));
        let consequence = node
            .child_by_field_name("consequence")
            .and_then(|consequence| source.get(consequence.byte_range()));

        if let (Some(condition), Some(consequence)) = (condition, consequence) {
            let normalized_condition = condition.split_whitespace().collect::<String>();
            let panic_like = consequence.contains("panic(")
                || consequence.contains("log.Fatal(")
                || consequence.contains("log.Fatalf(")
                || consequence.contains("log.Fatalln(");
            if normalized_condition.contains("err!=nil") && panic_like {
                lines.push(node.start_position().row + 1);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_panic_on_error(child, source, lines);
    }
}

fn collect_errorf_calls(body_node: Node<'_>, source: &str) -> Vec<FormattedErrorCall> {
    let mut calls = Vec::new();
    visit_for_errorf_calls(body_node, source, &mut calls);
    calls
}

fn visit_for_errorf_calls(node: Node<'_>, source: &str, calls: &mut Vec<FormattedErrorCall>) {
    if node.kind() == "call_expression" {
        let function_node = node.child_by_field_name("function");
        let arguments_node = node.child_by_field_name("arguments");

        if let (Some(function_node), Some(arguments_node)) = (function_node, arguments_node) {
            let target = source.get(function_node.byte_range()).unwrap_or("");
            if target.trim() == "fmt.Errorf" {
                let arguments = source.get(arguments_node.byte_range()).unwrap_or("");
                let format_string = first_string_literal(arguments_node, source);
                calls.push(FormattedErrorCall {
                    line: node.start_position().row + 1,
                    format_string,
                    mentions_err: arguments.contains("err"),
                    uses_percent_w: arguments.contains("%w"),
                });
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_errorf_calls(child, source, calls);
    }
}

fn first_string_literal(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if matches!(
            child.kind(),
            "interpreted_string_literal" | "raw_string_literal"
        ) {
            let literal = source.get(child.byte_range())?;
            return Some(literal.trim_matches('"').trim_matches('`').to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{extract_doc_comment, package_alias_from_import_path, parse_file};
    use crate::model::SymbolKind;
    use std::path::Path;

    #[test]
    fn derives_import_alias_from_path() {
        assert_eq!(
            package_alias_from_import_path("github.com/acme/utils"),
            "utils"
        );
    }

    #[test]
    fn collects_package_level_function_alias_vars_as_symbols() {
        let source = r#"package pdf

import font "example.com/font"

var (
    IsCustomFont = font.IsCustomFont
    PlainValue = 42
)

func collectAllStandardFontsInTemplate() {
    IsCustomFont("Helvetica")
}
"#;

        let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
        assert!(parsed.symbols.iter().any(|symbol| {
            symbol.name == "IsCustomFont" && matches!(symbol.kind, SymbolKind::Function)
        }));
        assert!(
            !parsed
                .symbols
                .iter()
                .any(|symbol| symbol.name == "PlainValue")
        );
    }

    #[test]
    fn extracts_doc_comment_text() {
        let source = "// Run Processes The Input\n// This function does X by doing Y because Z\nfunc Run() {}\n";

        let comment = extract_doc_comment(source, 2).expect("doc comment should exist");
        assert_eq!(
            comment,
            "Run Processes The Input\nThis function does X by doing Y because Z"
        );
    }

    #[test]
    fn collects_error_handling_signals() {
        let source = r#"package sample

import (
    "fmt"
    "log"
)

func Run(err error) error {
    _ = err
    if err != nil {
        panic(err)
    }
    return fmt.Errorf("wrap: %v", err)
}

func LogOnly(err error) {
    if err != nil {
        log.Fatal(err)
    }
}
"#;

        let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
        let run = parsed
            .functions
            .iter()
            .find(|function| function.fingerprint.name == "Run")
            .expect("Run should be parsed");
        let log_only = parsed
            .functions
            .iter()
            .find(|function| function.fingerprint.name == "LogOnly")
            .expect("LogOnly should be parsed");

        assert_eq!(run.dropped_error_lines, vec![9]);
        assert_eq!(run.panic_on_error_lines, vec![10]);
        assert_eq!(run.errorf_calls.len(), 1);
        assert!(run.errorf_calls[0].mentions_err);
        assert!(!run.errorf_calls[0].uses_percent_w);
        assert_eq!(log_only.panic_on_error_lines, vec![17]);
    }

    #[test]
    fn collects_context_and_sleep_signals() {
        let source = r#"package sample

import (
    "context"
    "time"
)

func Poll(ctx context.Context) {
    for {
        time.Sleep(time.Second)
        _ = ctx
    }
}
"#;

        let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
        let poll = parsed
            .functions
            .iter()
            .find(|function| function.fingerprint.name == "Poll")
            .expect("Poll should be parsed");

        assert!(poll.has_context_parameter);
        assert_eq!(poll.sleep_in_loop_lines, vec![10]);
    }

    #[test]
    fn collects_string_concat_and_goroutine_signals() {
        let source = r#"package sample

func Build(parts []string) string {
    out := ""
    for _, part := range parts {
        out += part
        go notify(part)
    }
    return out
}

func notify(value string) {}
"#;

        let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
        let build = parsed
            .functions
            .iter()
            .find(|function| function.fingerprint.name == "Build")
            .expect("Build should be parsed");

        assert_eq!(build.string_concat_in_loop_lines, vec![6]);
        assert_eq!(build.goroutine_launch_lines, vec![7]);
    }
}
