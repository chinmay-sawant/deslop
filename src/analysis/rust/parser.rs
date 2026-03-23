use std::path::Path;

use anyhow::{Context, Result, anyhow};
use tree_sitter::{Node, Parser};

use crate::analysis::{
    CallSite, DeclaredSymbol, ImportSpec, Language, NamedLiteral, ParsedFile, ParsedFunction,
};
use crate::model::{FunctionFingerprint, SymbolKind};

pub(super) fn parse_file(path: &Path, source: &str) -> Result<ParsedFile> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|error| anyhow!(error.to_string()))
        .context("failed to configure Rust parser")?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("tree-sitter returned no parse tree"))?;

    let root = tree.root_node();
    let is_test_file = is_test_file(path);
    let imports = collect_imports(root, source);
    let package_string_literals = collect_pkg_strings(root, source);
    let functions = collect_functions(root, source, is_test_file);
    let symbols = collect_symbols(root, source, &functions, &imports);

    Ok(ParsedFile {
        language: Language::Rust,
        path: path.to_path_buf(),
        package_name: module_name_for_path(path),
        is_test_file,
        syntax_error: root.has_error(),
        byte_size: source.len(),
        pkg_strings: package_string_literals,
        struct_tags: Vec::new(),
        functions,
        imports,
        symbols,
    })
}

fn collect_imports(root: Node<'_>, source: &str) -> Vec<ImportSpec> {
    let mut imports = Vec::new();
    visit_for_imports(root, source, &mut imports);
    imports
}

fn visit_for_imports(node: Node<'_>, source: &str, imports: &mut Vec<ImportSpec>) {
    if node.kind() == "use_declaration"
        && let Some(argument) = node.child_by_field_name("argument")
    {
        let is_public = source
            .get(node.byte_range())
            .is_some_and(|text| text.starts_with("pub"));
        flatten_use_tree(argument, source, None, imports, is_public);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_imports(child, source, imports);
    }
}

fn flatten_use_tree(
    node: Node<'_>,
    source: &str,
    prefix: Option<String>,
    imports: &mut Vec<ImportSpec>,
    is_public: bool,
) {
    match node.kind() {
        "use_as_clause" => {
            let Some(path_node) = node.child_by_field_name("path") else {
                return;
            };
            let alias = node
                .child_by_field_name("alias")
                .and_then(|alias_node| source.get(alias_node.byte_range()))
                .map(str::trim)
                .filter(|alias| !alias.is_empty())
                .unwrap_or("*")
                .to_string();
            let path = combine_path_prefix(prefix.as_deref(), &render_use_path(path_node, source));

            imports.push(build_rust_import_spec(alias, path, is_public));
        }
        "scoped_use_list" => {
            let next_prefix = node
                .child_by_field_name("path")
                .map(|path_node| {
                    combine_path_prefix(prefix.as_deref(), &render_use_path(path_node, source))
                })
                .or(prefix);

            if let Some(list_node) = node.child_by_field_name("list") {
                flatten_use_tree(list_node, source, next_prefix, imports, is_public);
            }
        }
        "use_list" => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                flatten_use_tree(child, source, prefix.clone(), imports, is_public);
            }
        }
        "use_wildcard" => {
            let wildcard_path = node
                .named_child(0)
                .map(|child| {
                    combine_path_prefix(prefix.as_deref(), &render_use_path(child, source))
                })
                .unwrap_or_else(|| combine_path_prefix(prefix.as_deref(), "*"));

            imports.push(ImportSpec {
                alias: "*".to_string(),
                path: wildcard_path,
                namespace_path: prefix,
                imported_name: Some("*".to_string()),
                is_public,
            });
        }
        _ => {
            let (alias, path) = if node.kind() == "self" {
                let path = prefix.unwrap_or_else(|| render_use_path(node, source));
                (import_alias(&path), path)
            } else {
                let path = combine_path_prefix(prefix.as_deref(), &render_use_path(node, source));
                (import_alias(&path), path)
            };
            imports.push(build_rust_import_spec(alias, path, is_public));
        }
    }
}

fn build_rust_import_spec(alias: String, path: String, is_public: bool) -> ImportSpec {
    let (namespace_path, imported_name) = rust_import_segments(&path);

    ImportSpec {
        alias,
        path,
        namespace_path,
        imported_name,
        is_public,
    }
}

fn rust_import_segments(path: &str) -> (Option<String>, Option<String>) {
    if let Some((namespace_path, imported_name)) = path.rsplit_once("::") {
        return (
            Some(namespace_path.to_string()),
            Some(imported_name.to_string()),
        );
    }

    (None, None)
}

fn render_use_path(node: Node<'_>, source: &str) -> String {
    match node.kind() {
        "scoped_identifier" => {
            let path = node
                .child_by_field_name("path")
                .map(|path_node| render_use_path(path_node, source));
            let name = node
                .child_by_field_name("name")
                .and_then(|name_node| source.get(name_node.byte_range()))
                .map(str::trim)
                .unwrap_or("");
            combine_path_prefix(path.as_deref(), name)
        }
        _ => source
            .get(node.byte_range())
            .map(str::trim)
            .unwrap_or("")
            .to_string(),
    }
}

fn combine_path_prefix(prefix: Option<&str>, suffix: &str) -> String {
    match prefix.map(str::trim).filter(|prefix| !prefix.is_empty()) {
        Some(prefix) if !suffix.is_empty() => format!("{prefix}::{suffix}"),
        Some(prefix) => prefix.to_string(),
        None => suffix.to_string(),
    }
}

fn import_alias(path: &str) -> String {
    path.rsplit("::")
        .next()
        .filter(|segment| !segment.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn collect_symbols(
    root: Node<'_>,
    source: &str,
    functions: &[ParsedFunction],
    imports: &[ImportSpec],
) -> Vec<DeclaredSymbol> {
    let mut symbols = functions
        .iter()
        .map(|function| DeclaredSymbol {
            name: function.fingerprint.name.clone(),
            kind: if function.fingerprint.kind == "method" {
                SymbolKind::Method
            } else {
                SymbolKind::Function
            },
            receiver_type: function.fingerprint.receiver_type.clone(),
            receiver_is_pointer: None,
            line: function.fingerprint.start_line,
        })
        .collect::<Vec<_>>();

    // Add re-exports as symbols to allow resolution of public imports
    for import in imports {
        if import.is_public && import.alias != "*" {
            symbols.push(DeclaredSymbol {
                name: import.alias.clone(),
                kind: SymbolKind::Function, // Treat re-exports as functions for resolution
                receiver_type: None,
                receiver_is_pointer: None,
                line: 1, // Metadata only
            });
        }
    }

    visit_for_symbols(root, source, &mut symbols);
    symbols.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    symbols
}

fn visit_for_symbols(node: Node<'_>, source: &str, symbols: &mut Vec<DeclaredSymbol>) {
    let symbol_kind = match node.kind() {
        "struct_item" => Some(SymbolKind::Struct),
        "enum_item" | "type_item" => Some(SymbolKind::Type),
        "trait_item" => Some(SymbolKind::Interface),
        _ => None,
    };

    if let Some(kind) = symbol_kind
        && let Some(name_node) = node.child_by_field_name("name")
        && let Some(name) = source.get(name_node.byte_range())
    {
        symbols.push(DeclaredSymbol {
            name: name.trim().to_string(),
            kind,
            receiver_type: None,
            receiver_is_pointer: None,
            line: node.start_position().row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_symbols(child, source, symbols);
    }
}

fn collect_pkg_strings(root: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_pkg_strings(root, source, &mut literals);
    literals
}

fn visit_pkg_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if matches!(node.kind(), "const_item" | "static_item")
        && let Some(literal) = named_string_from_item(node, source)
        && !is_inside_function(node)
    {
        literals.push(literal);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_pkg_strings(child, source, literals);
    }
}

fn named_string_from_item(node: Node<'_>, source: &str) -> Option<NamedLiteral> {
    let name = node
        .child_by_field_name("name")
        .and_then(|name_node| source.get(name_node.byte_range()))?
        .trim()
        .to_string();
    let value_node = node.child_by_field_name("value")?;
    let value = string_literal_value(value_node, source)?;

    Some(NamedLiteral {
        line: node.start_position().row + 1,
        name,
        value,
    })
}

fn collect_functions(root: Node<'_>, source: &str, is_test_file: bool) -> Vec<ParsedFunction> {
    let mut functions = Vec::new();
    visit_for_functions(root, source, is_test_file, &mut functions);
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
    is_test_file: bool,
    functions: &mut Vec<ParsedFunction>,
) {
    if node.kind() == "function_item"
        && let Some(parsed_function) = parse_function_node(node, source, is_test_file)
    {
        functions.push(parsed_function);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_functions(child, source, is_test_file, functions);
    }
}

fn parse_function_node(node: Node<'_>, source: &str, is_test_file: bool) -> Option<ParsedFunction> {
    let body_node = node.child_by_field_name("body")?;
    let calls = collect_calls(body_node, source);
    let local_string_literals = collect_local_strings(body_node, source);
    let local_binding_names = collect_local_bindings(node, source);
    let doc_comment = extract_doc_comment(source, node.start_position().row);
    let kind = function_kind(node, source);
    let receiver_type = if kind == "method" {
        enclosing_impl_type(node, source)
    } else {
        None
    };
    let is_test_function = function_is_test_only(node, source, is_test_file);
    let safety_comment_lines = collect_safety_comments(source, node);
    let unsafe_lines = collect_unsafe_lines(node, body_node, source);
    let fingerprint = build_function_fingerprint(node, source, kind, receiver_type, calls.len())?;

    Some(ParsedFunction {
        fingerprint,
        calls,
        has_context_parameter: false,
        is_test_function,
        local_binding_names,
        doc_comment,
        local_strings: local_string_literals,
        test_summary: None,
        safety_comment_lines,
        unsafe_lines,
        dropped_errors: Vec::new(),
        panic_errors: Vec::new(),
        errorf_calls: Vec::new(),
        context_factory_calls: Vec::new(),
        goroutines: Vec::new(),
        loop_goroutines: Vec::new(),
        unmanaged_goroutines: Vec::new(),
        sleep_loops: Vec::new(),
        busy_wait_lines: Vec::new(),
        mutex_loops: Vec::new(),
        alloc_loops: Vec::new(),
        fmt_loops: Vec::new(),
        reflect_loops: Vec::new(),
        concat_loops: Vec::new(),
        json_loops: Vec::new(),
        db_query_calls: Vec::new(),
    })
}

fn function_kind(node: Node<'_>, source: &str) -> &'static str {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return "function";
    };

    let Some(parameters_text) = source.get(parameters.byte_range()) else {
        return "function";
    };

    if parameters_text.contains("self") {
        "method"
    } else {
        "function"
    }
}

fn enclosing_impl_type(node: Node<'_>, source: &str) -> Option<String> {
    let mut parent = node.parent();

    while let Some(current) = parent {
        if current.kind() == "impl_item" {
            return current
                .child_by_field_name("type")
                .and_then(|type_node| source.get(type_node.byte_range()))
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(ToOwned::to_owned);
        }

        parent = current.parent();
    }

    None
}

fn build_function_fingerprint(
    node: Node<'_>,
    source: &str,
    kind: &str,
    receiver_type: Option<String>,
    call_count: usize,
) -> Option<FunctionFingerprint> {
    let name_node = node.child_by_field_name("name")?;
    let body_node = node.child_by_field_name("body")?;
    let function_text = source.get(node.byte_range())?;
    let name = source.get(name_node.byte_range())?.to_string();
    let comment_lines =
        count_comment_lines(function_text) + count_doc_lines(source, node.start_position().row);
    let code_lines = count_code_lines(function_text);
    let complexity_score = 1 + count_control_nodes(body_node);
    let symmetry_score = compute_symmetry_score(body_node);
    let line_count = node.end_position().row - node.start_position().row + 1;
    let comment_to_code_ratio = if code_lines == 0 {
        0.0
    } else {
        comment_lines as f64 / code_lines as f64
    };

    Some(FunctionFingerprint {
        name,
        kind: kind.to_string(),
        receiver_type,
        start_line: node.start_position().row + 1,
        end_line: node.end_position().row + 1,
        line_count,
        comment_lines,
        code_lines,
        comment_to_code_ratio,
        complexity_score,
        symmetry_score,
        boilerplate_err_guards: 0,
        contains_any_type: false,
        contains_empty_interface: false,
        type_assertion_count: 0,
        call_count,
    })
}

fn collect_calls(node: Node<'_>, source: &str) -> Vec<CallSite> {
    let mut calls = Vec::new();
    visit_for_calls(node, source, &mut calls);
    calls
}

fn visit_for_calls(node: Node<'_>, source: &str, calls: &mut Vec<CallSite>) {
    if node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
    {
        let function_text = render_call_target(function_node, source);
        let (receiver, name) = split_call_target(&function_text);
        calls.push(CallSite {
            receiver,
            name,
            line: node.start_position().row + 1,
        });
    }

    if node.kind() == "macro_invocation"
        && let Some(macro_node) = node.child_by_field_name("macro")
        && let Some(macro_text) = source.get(macro_node.byte_range())
    {
        let (receiver, name) = split_call_target(macro_text);
        calls.push(CallSite {
            receiver,
            name: format!("{name}!"),
            line: node.start_position().row + 1,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_calls(child, source, calls);
    }
}

fn render_call_target(node: Node<'_>, source: &str) -> String {
    if node.kind() == "field_expression" {
        let value = node
            .child_by_field_name("value")
            .map(|value_node| render_call_target(value_node, source));
        let field = node
            .child_by_field_name("field")
            .and_then(|field_node| source.get(field_node.byte_range()))
            .map(str::trim)
            .unwrap_or("");

        return combine_path_prefix(value.as_deref(), field);
    }

    source
        .get(node.byte_range())
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

fn split_call_target(function_text: &str) -> (Option<String>, String) {
    let normalized = function_text.trim();

    if let Some((receiver, name)) = normalized.rsplit_once('.') {
        return (Some(receiver.trim().to_string()), name.trim().to_string());
    }

    if let Some((receiver, name)) = normalized.rsplit_once("::") {
        return (Some(receiver.trim().to_string()), name.trim().to_string());
    }

    (None, normalized.to_string())
}

fn module_name_for_path(path: &Path) -> Option<String> {
    if path.file_stem().and_then(|stem| stem.to_str()) == Some("mod") {
        return path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned);
    }

    path.file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .map(ToOwned::to_owned)
}

fn is_test_file(path: &Path) -> bool {
    let in_tests_directory = path
        .components()
        .any(|component| component.as_os_str() == "tests");
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    in_tests_directory || file_name == "tests.rs" || file_name.ends_with("_test.rs")
}

fn extract_doc_comment(source: &str, function_start_row: usize) -> Option<String> {
    let lines = source.lines().collect::<Vec<_>>();
    if function_start_row == 0 || function_start_row > lines.len() {
        return None;
    }

    let mut collected = Vec::new();
    let mut index = function_start_row;

    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            collected.push(
                trimmed
                    .trim_start_matches('/')
                    .trim_start_matches('!')
                    .trim()
                    .to_string(),
            );
            continue;
        }

        if trimmed.ends_with("*/") {
            let mut block_lines = vec![trimmed.trim_end_matches("*/").trim().to_string()];
            while index > 0 {
                index -= 1;
                let block_line = lines[index].trim();
                block_lines.push(block_line.trim_start_matches("/*").trim().to_string());
                if block_line.starts_with("/**") || block_line.starts_with("/*!") {
                    block_lines.reverse();
                    collected.extend(block_lines);
                    return Some(collected.join("\n").trim().to_string());
                }
            }
            break;
        }

        break;
    }

    if collected.is_empty() {
        None
    } else {
        collected.reverse();
        Some(collected.join("\n").trim().to_string())
    }
}

fn function_is_test_only(node: Node<'_>, source: &str, is_test_file: bool) -> bool {
    if is_test_file {
        return true;
    }

    let mut current = Some(node);
    while let Some(candidate) = current {
        if leading_attributes(candidate)
            .into_iter()
            .any(|attribute| attribute_marks_test(attribute, source))
        {
            return true;
        }

        current = candidate.parent();
    }

    false
}

fn leading_attributes(node: Node<'_>) -> Vec<Node<'_>> {
    let mut attributes = Vec::new();
    let mut current = node.prev_named_sibling();

    while let Some(sibling) = current {
        if sibling.kind() != "attribute_item" {
            break;
        }

        attributes.push(sibling);
        current = sibling.prev_named_sibling();
    }

    attributes
}

fn attribute_marks_test(node: Node<'_>, source: &str) -> bool {
    let normalized = source
        .get(node.byte_range())
        .unwrap_or("")
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    normalized.contains("cfg(test)")
        || normalized.starts_with("#[test]")
        || normalized.ends_with("::test]")
        || normalized.contains("::test(")
}

fn collect_unsafe_lines(function_node: Node<'_>, body_node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    let signature_text = source
        .get(function_node.start_byte()..body_node.start_byte())
        .unwrap_or("");

    if signature_text
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == "unsafe")
    {
        lines.push(function_node.start_position().row + 1);
    }

    visit_for_unsafe_lines(body_node, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn collect_safety_comments(source: &str, function_node: Node<'_>) -> Vec<usize> {
    let lines = source.lines().collect::<Vec<_>>();
    let start = function_node.start_position().row.saturating_sub(2);
    let end = function_node
        .end_position()
        .row
        .min(lines.len().saturating_sub(1));
    let mut safety_lines = Vec::new();

    for (index, line) in lines.iter().enumerate().take(end + 1).skip(start) {
        if line.contains("SAFETY:") {
            safety_lines.push(index + 1);
        }
    }

    safety_lines
}

fn visit_for_unsafe_lines(node: Node<'_>, lines: &mut Vec<usize>) {
    if node.kind() == "unsafe_block" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_unsafe_lines(child, lines);
    }
}

fn collect_local_strings(node: Node<'_>, source: &str) -> Vec<NamedLiteral> {
    let mut literals = Vec::new();
    visit_local_strings(node, source, &mut literals);
    literals
}

fn collect_local_bindings(function_node: Node<'_>, source: &str) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(parameters) = function_node.child_by_field_name("parameters") {
        collect_param_bindings(parameters, source, &mut names);
    }

    if let Some(body_node) = function_node.child_by_field_name("body") {
        visit_local_bindings(body_node, source, &mut names);
    }

    names.sort();
    names.dedup();
    names
}

fn collect_param_bindings(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "parameter" => {
                if let Some(pattern) = child.child_by_field_name("pattern") {
                    collect_ident_patterns(pattern, source, names);
                }
            }
            "self_parameter" => names.push("self".to_string()),
            _ => {}
        }
    }
}

fn visit_local_bindings(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    if node.kind() == "let_declaration"
        && let Some(pattern_node) = node.child_by_field_name("pattern")
    {
        collect_ident_patterns(pattern_node, source, names);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_local_bindings(child, source, names);
    }
}

fn collect_ident_patterns(node: Node<'_>, source: &str, names: &mut Vec<String>) {
    if matches!(node.kind(), "identifier" | "self")
        && let Some(name) = source.get(node.byte_range())
    {
        names.push(name.trim().to_string());
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_ident_patterns(child, source, names);
    }
}

fn visit_local_strings(node: Node<'_>, source: &str, literals: &mut Vec<NamedLiteral>) {
    if node.kind() == "let_declaration"
        && let Some(pattern_node) = node.child_by_field_name("pattern")
        && pattern_node.kind() == "identifier"
        && let Some(name) = source.get(pattern_node.byte_range())
        && let Some(value_node) = node.child_by_field_name("value")
        && let Some(value) = string_literal_value(value_node, source)
    {
        literals.push(NamedLiteral {
            line: node.start_position().row + 1,
            name: name.trim().to_string(),
            value,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_local_strings(child, source, literals);
    }
}

fn string_literal_value(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "string_literal" | "raw_string_literal" => {
            let text = source.get(node.byte_range())?.trim();
            unquote_rust_string(text)
        }
        _ => None,
    }
}

fn unquote_rust_string(text: &str) -> Option<String> {
    if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        return Some(text[1..text.len() - 1].to_string());
    }

    if text.starts_with('r') {
        let start = text.find('"')?;
        let end = text.rfind('"')?;
        if end > start {
            return Some(text[start + 1..end].to_string());
        }
    }

    None
}

fn is_inside_function(node: Node<'_>) -> bool {
    let mut current = node.parent();

    while let Some(parent) = current {
        if parent.kind() == "function_item" {
            return true;
        }

        current = parent.parent();
    }

    false
}

fn count_control_nodes(node: Node<'_>) -> usize {
    let mut total = 0;
    let mut cursor = node.walk();

    for child in node.named_children(&mut cursor) {
        total += usize::from(is_control_node(child.kind()));
        total += count_control_nodes(child);
    }

    total
}

fn is_control_node(kind: &str) -> bool {
    matches!(
        kind,
        "if_expression"
            | "for_expression"
            | "while_expression"
            | "loop_expression"
            | "match_expression"
    )
}

fn compute_symmetry_score(body_node: Node<'_>) -> f64 {
    let mut cursor = body_node.walk();
    let mut statement_kinds = Vec::new();

    for child in body_node.named_children(&mut cursor) {
        statement_kinds.push(child.kind().to_string());
    }

    if statement_kinds.len() < 2 {
        return 0.0;
    }

    statement_kinds.sort();

    let mut best_run = 1usize;
    let mut current_run = 1usize;

    for pair in statement_kinds.windows(2) {
        if pair[0] == pair[1] {
            current_run += 1;
            best_run = best_run.max(current_run);
        } else {
            current_run = 1;
        }
    }

    best_run as f64 / statement_kinds.len() as f64
}

fn count_comment_lines(text: &str) -> usize {
    let mut count = 0usize;
    let mut in_block_comment = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if in_block_comment {
            count += 1;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("//") {
            count += 1;
            continue;
        }

        if trimmed.starts_with("/*") {
            count += 1;
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
        }
    }

    count
}

fn count_code_lines(text: &str) -> usize {
    let mut count = 0usize;
    let mut in_block_comment = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
                let suffix = trimmed
                    .split_once("*/")
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or("");
                if !suffix.is_empty() {
                    count += 1;
                }
            }
            continue;
        }

        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("/*") {
            if trimmed.contains("*/") {
                let suffix = trimmed
                    .split_once("*/")
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or("");
                if !suffix.is_empty() {
                    count += 1;
                }
            } else {
                in_block_comment = true;
            }
            continue;
        }

        count += 1;
    }

    count
}

fn count_doc_lines(source: &str, function_start_row: usize) -> usize {
    let lines = source.lines().collect::<Vec<_>>();
    if function_start_row == 0 || function_start_row > lines.len() {
        return 0;
    }

    let mut count = 0usize;
    let mut index = function_start_row;

    while index > 0 {
        index -= 1;
        let trimmed = lines[index].trim();

        if trimmed.is_empty() {
            break;
        }

        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            count += 1;
            continue;
        }

        if trimmed.ends_with("*/") {
            count += 1;
            while index > 0 {
                index -= 1;
                count += 1;
                let block_line = lines[index].trim();
                if block_line.starts_with("/**") || block_line.starts_with("/*!") {
                    return count;
                }
            }
            return count;
        }

        break;
    }

    count
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::parse_file;
    use crate::analysis::Language;

    #[test]
    fn test_parse_functions() {
        let source = r#"
pub fn sum_pair(left: i32, right: i32) -> i32 {
    left + right
}

impl Runner {
    fn execute(&self) {
        sum_pair(1, 2);
    }
}
"#;

        let parsed = parse_file(Path::new("src/main.rs"), source)
            .expect("rust source should parse successfully");

        assert_eq!(parsed.language, Language::Rust);
        assert_eq!(parsed.package_name.as_deref(), Some("main"));
        assert!(!parsed.syntax_error);
        assert_eq!(parsed.functions.len(), 2);
        assert_eq!(parsed.functions[0].fingerprint.name, "sum_pair");
        assert_eq!(parsed.functions[1].fingerprint.kind, "method");
        assert_eq!(
            parsed.functions[1].fingerprint.receiver_type.as_deref(),
            Some("Runner")
        );
        assert!(!parsed.functions[0].is_test_function);
        assert_eq!(parsed.symbols.len(), 2);
    }

    #[test]
    fn test_extract_evidence() {
        let source = r#"
use std::fmt::{self, Display as FmtDisplay};
use crate::config::*;

const API_TOKEN: &str = "sk_test_1234567890";

pub struct Runner;
enum Mode {
    Fast,
}
trait Render {
    fn render(&self);
}
type Output = String;

impl Runner {
    pub unsafe fn execute(&self) {
        let password = "super-secret-value";
        dbg!(password);
        todo!();
        value.unwrap();
        unsafe {
            dangerous();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn detects_test_only_code() {
        let api_key = "top-secret-value";
        assert_eq!(api_key.len(), 16);
    }
}
"#;

        let parsed = parse_file(Path::new("src/lib.rs"), source)
            .expect("rust source should parse successfully");

        assert_eq!(parsed.imports.len(), 3);
        assert!(
            parsed
                .imports
                .iter()
                .any(|import| { import.alias == "fmt" && import.path.contains("std::fmt") })
        );
        assert!(parsed.imports.iter().any(|import| {
            import.alias == "FmtDisplay"
                && import.path == "std::fmt::Display"
                && import.namespace_path.as_deref() == Some("std::fmt")
                && import.imported_name.as_deref() == Some("Display")
        }));
        assert!(
            parsed
                .imports
                .iter()
                .any(|import| { import.alias == "*" && import.path.contains("crate::config") })
        );

        assert_eq!(parsed.pkg_strings.len(), 1);
        assert_eq!(parsed.pkg_strings[0].name, "API_TOKEN");

        assert!(parsed.symbols.iter().any(|symbol| {
            symbol.name == "Runner" && matches!(symbol.kind, crate::model::SymbolKind::Struct)
        }));
        assert!(parsed.symbols.iter().any(|symbol| {
            symbol.name == "Mode" && matches!(symbol.kind, crate::model::SymbolKind::Type)
        }));
        assert!(parsed.symbols.iter().any(|symbol| {
            symbol.name == "Render" && matches!(symbol.kind, crate::model::SymbolKind::Interface)
        }));
        assert!(parsed.symbols.iter().any(|symbol| {
            symbol.name == "Output" && matches!(symbol.kind, crate::model::SymbolKind::Type)
        }));

        let execute = parsed
            .functions
            .iter()
            .find(|function| function.fingerprint.name == "execute")
            .expect("execute should be parsed");
        assert!(!execute.is_test_function);
        assert!(
            execute
                .local_binding_names
                .iter()
                .any(|name| name == "self")
        );
        assert_eq!(execute.local_strings.len(), 1);
        assert_eq!(execute.local_strings[0].name, "password");
        assert!(
            execute
                .local_binding_names
                .iter()
                .any(|name| name == "password")
        );
        assert!(execute.calls.iter().any(|call| call.name == "dbg!"));
        assert!(execute.calls.iter().any(|call| call.name == "todo!"));
        assert!(
            execute
                .calls
                .iter()
                .any(|call| { call.receiver.as_deref() == Some("value") && call.name == "unwrap" })
        );
        assert!(execute.safety_comment_lines.is_empty());
        assert_eq!(execute.unsafe_lines.len(), 2);

        let test_fn = parsed
            .functions
            .iter()
            .find(|function| function.fingerprint.name == "detects_test_only_code")
            .expect("test function should be parsed");
        assert!(test_fn.is_test_function);
    }

    #[test]
    fn test_syntax_error() {
        let source = "pub fn broken( {\n    println!(\"oops\");\n}\n";

        let parsed = parse_file(Path::new("src/lib.rs"), source)
            .expect("tree-sitter should recover from syntax errors");

        assert!(parsed.syntax_error);
        assert_eq!(parsed.functions.len(), 1);
    }
}
