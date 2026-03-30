use std::collections::BTreeSet;
use std::path::Path;

use tree_sitter::{Node, Parser};

use crate::analysis::{
    AnalysisResult, CallSite, DeclaredSymbol, Error, FieldSummary, ImportSpec, Language, MacroCall,
    NamedLiteral, ParsedFile, ParsedFunction, RuntimeCall, RustEnumSummary, RustStaticSummary,
    StructSummary, UnsafePattern, UnsafePatternKind,
};
use crate::model::{FunctionFingerprint, SymbolKind};

pub(super) fn parse_file(path: &Path, source: &str) -> AnalysisResult<ParsedFile> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_err(|error| Error::parser_configuration("Rust", error.to_string()))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| Error::missing_parse_tree("Rust"))?;

    let root = tree.root_node();
    let is_test_file = is_test_file(path);
    let imports = collect_imports(root, source);
    let package_string_literals = collect_pkg_strings(root, source);
    let default_impls = collect_trait_impls(root, source, "Default");
    let functions = collect_functions(root, source, is_test_file);
    let symbols = collect_symbols(root, source, &functions, &imports);
    let rust_statics = collect_static_summaries(root, source);
    let rust_enums = collect_enum_summaries(root, source);
    let structs = collect_struct_summaries(root, source, &default_impls);

    Ok(ParsedFile {
        language: Language::Rust,
        path: path.to_path_buf(),
        package_name: module_name_for_path(path),
        is_test_file,
        syntax_error: root.has_error(),
        line_count: source.lines().count(),
        byte_size: source.len(),
        pkg_strings: package_string_literals,
        comments: Vec::new(),
        struct_tags: Vec::new(),
        functions,
        imports,
        symbols,
        class_summaries: Vec::new(),
        package_vars: Vec::new(),
        interfaces: Vec::new(),
        go_structs: Vec::new(),
        module_scope_calls: Vec::new(),
        top_level_bindings: Vec::new(),
        python_models: Vec::new(),
        rust_statics,
        rust_enums,
        structs,
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

            imports.push(build_rust_import_spec(
                alias,
                path,
                is_public,
                node.start_position().row + 1,
            ));
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
                line: node.start_position().row + 1,
                group_line: node.start_position().row + 1,
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
            imports.push(build_rust_import_spec(
                alias,
                path,
                is_public,
                node.start_position().row + 1,
            ));
        }
    }
}

fn build_rust_import_spec(alias: String, path: String, is_public: bool, line: usize) -> ImportSpec {
    let (namespace_path, imported_name) = rust_import_segments(&path);

    ImportSpec {
        line,
        group_line: line,
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
    let body_text = source
        .get(body_node.byte_range())
        .unwrap_or_default()
        .to_string();
    let kind = function_kind(node, source);
    let receiver_type = if kind == "method" {
        enclosing_impl_type(node, source)
    } else {
        None
    };
    let is_test_function = function_is_test_only(node, source, is_test_file);
    let safety_comment_lines = collect_safety_comments(source, node);
    let unsafe_lines = collect_unsafe_lines(node, body_node, source);
    let is_async = function_is_async(node, body_node, source);
    let await_points = collect_await_points(body_node);
    let macro_calls = collect_macro_calls(&calls);
    let spawn_calls =
        collect_named_runtime_calls(&calls, &["spawn", "spawn_local", "spawn_blocking"]);
    let lock_calls = collect_named_runtime_calls(&calls, &["lock", "lock_owned", "read", "write"]);
    let permit_acquires =
        collect_named_runtime_calls(&calls, &["acquire", "acquire_owned", "reserve", "get"]);
    let futures_created = collect_future_creations(body_node, source);
    let blocking_calls = collect_blocking_calls(body_node, source, &calls);
    let select_macro_lines = macro_calls
        .iter()
        .filter(|call| call.name == "select!")
        .map(|call| call.line)
        .collect::<Vec<_>>();
    let drop_impl = inside_trait_impl(node, source, "Drop");
    let (write_loops, line_iteration_loops, default_hasher_lines) =
        collect_loop_operation_lines(body_node, source);
    let boxed_container_lines = collect_boxed_container_lines(body_node, source);
    let unsafe_soundness = collect_unsafe_patterns(body_node, source);
    let fingerprint = build_function_fingerprint(node, source, kind, receiver_type, calls.len())?;

    Some(ParsedFunction {
        fingerprint,
        signature_text: source
            .get(node.start_byte()..body_node.start_byte())
            .unwrap_or_default()
            .to_string(),
        body_start_line: body_node.start_position().row + 1,
        calls,
        exception_handlers: Vec::new(),
        has_context_parameter: false,
        is_test_function,
        local_binding_names,
        doc_comment,
        body_text,
        local_strings: local_string_literals,
        normalized_body: String::new(),
        validation_signature: None,
        exception_block_signatures: Vec::new(),
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
        none_comparison_lines: Vec::new(),
        side_effect_comprehension_lines: Vec::new(),
        redundant_return_none_lines: Vec::new(),
        list_materialization_lines: Vec::new(),
        deque_operation_lines: Vec::new(),
        temp_collection_lines: Vec::new(),
        recursive_call_lines: Vec::new(),
        list_membership_loop_lines: Vec::new(),
        repeated_len_loop_lines: Vec::new(),
        builtin_candidate_lines: Vec::new(),
        missing_context_manager_lines: Vec::new(),
        has_complete_type_hints: false,
        has_varargs: false,
        has_kwargs: false,
        is_async,
        await_points,
        macro_calls,
        spawn_calls,
        lock_calls,
        permit_acquires,
        futures_created,
        blocking_calls,
        select_macro_lines,
        drop_impl,
        write_loops,
        line_iteration_loops,
        default_hasher_lines,
        boxed_container_lines,
        unsafe_soundness,
    })
}

fn collect_trait_impls(root: Node<'_>, source: &str, trait_name: &str) -> BTreeSet<String> {
    let mut impls = BTreeSet::new();
    visit_trait_impls(root, source, trait_name, &mut impls);
    impls
}

fn visit_trait_impls(node: Node<'_>, source: &str, trait_name: &str, impls: &mut BTreeSet<String>) {
    if node.kind() == "impl_item"
        && let Some(type_name) = trait_impl_type(node, source, trait_name)
    {
        impls.insert(type_name);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_trait_impls(child, source, trait_name, impls);
    }
}

fn trait_impl_type(node: Node<'_>, source: &str, trait_name: &str) -> Option<String> {
    let normalized = source
        .get(node.byte_range())?
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let prefix = format!("impl{trait_name}for");
    let remainder = normalized.strip_prefix(&prefix)?;
    let type_name = remainder
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    if type_name.is_empty() {
        None
    } else {
        Some(type_name)
    }
}

fn collect_struct_summaries(
    root: Node<'_>,
    source: &str,
    default_impls: &BTreeSet<String>,
) -> Vec<StructSummary> {
    let mut structs = Vec::new();
    visit_for_struct_summaries(root, source, default_impls, &mut structs);
    structs.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    structs
}

fn visit_for_struct_summaries(
    node: Node<'_>,
    source: &str,
    default_impls: &BTreeSet<String>,
    structs: &mut Vec<StructSummary>,
) {
    if node.kind() == "struct_item"
        && let Some(summary) = build_struct_summary(node, source, default_impls)
    {
        structs.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_struct_summaries(child, source, default_impls, structs);
    }
}

fn build_struct_summary(
    node: Node<'_>,
    source: &str,
    default_impls: &BTreeSet<String>,
) -> Option<StructSummary> {
    let name_node = node.child_by_field_name("name")?;
    let name = source.get(name_node.byte_range())?.trim().to_string();
    let fields = node
        .child_by_field_name("body")
        .or_else(|| named_child_by_kind(node, "field_declaration_list"))
        .map(|body| collect_struct_fields(body, source))
        .unwrap_or_default();
    let derives = parse_derive_names(&leading_attributes(node), source);
    let visibility_pub = source.get(node.byte_range()).is_some_and(|text| {
        text.trim_start().starts_with("pub ") || text.trim_start().starts_with("pub(")
    });

    Some(StructSummary {
        line: node.start_position().row + 1,
        name: name.clone(),
        fields,
        has_debug_derive: derives.iter().any(|derive| derive == "Debug"),
        has_default_derive: derives.iter().any(|derive| derive == "Default"),
        has_serialize_derive: derives.iter().any(|derive| derive == "Serialize"),
        has_deserialize_derive: derives.iter().any(|derive| derive == "Deserialize"),
        visibility_pub,
        derives,
        attributes: parse_attribute_texts(&leading_attributes(node), source),
        impl_default: default_impls.contains(&name),
    })
}

fn named_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}

fn collect_struct_fields(node: Node<'_>, source: &str) -> Vec<FieldSummary> {
    let mut fields = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() != "field_declaration" {
            continue;
        }

        let Some(name_node) = child.child_by_field_name("name") else {
            continue;
        };
        let Some(type_node) = child.child_by_field_name("type") else {
            continue;
        };
        let name = source
            .get(name_node.byte_range())
            .unwrap_or("")
            .trim()
            .to_string();
        let type_text = source
            .get(type_node.byte_range())
            .unwrap_or("")
            .trim()
            .to_string();
        let normalized_type = type_text
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let primitive_name = normalized_type
            .trim_start_matches('&')
            .trim_start_matches("mut")
            .trim_start_matches('&');
        let is_primitive = matches!(
            primitive_name,
            "bool"
                | "str"
                | "String"
                | "usize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "isize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "f32"
                | "f64"
        );

        fields.push(FieldSummary {
            line: child.start_position().row + 1,
            name,
            attributes: parse_attribute_texts(&leading_attributes(child), source),
            is_pub: source.get(child.byte_range()).is_some_and(|text| {
                text.trim_start().starts_with("pub ") || text.trim_start().starts_with("pub(")
            }),
            is_option: normalized_type.starts_with("Option<")
                || normalized_type.contains("::Option<")
                || normalized_type.starts_with("std::option::Option<"),
            is_bool: primitive_name == "bool",
            is_primitive,
            type_text,
        });
    }
    fields
}

fn parse_derive_names(attributes: &[Node<'_>], source: &str) -> Vec<String> {
    let mut derives = Vec::new();

    for attribute in attributes {
        let Some(text) = source.get(attribute.byte_range()) else {
            continue;
        };
        let Some(start) = text.find("derive(") else {
            continue;
        };
        let Some(derive_text) = text.get(start + "derive(".len()..) else {
            continue;
        };
        let Some(end) = derive_text.find(')') else {
            continue;
        };
        for derive in derive_text.get(..end).unwrap_or("").split(',') {
            let cleaned = derive.trim().trim_matches(']');
            if cleaned.is_empty() {
                continue;
            }
            let simple = cleaned.rsplit("::").next().unwrap_or(cleaned).to_string();
            derives.push(simple);
        }
    }

    derives.sort();
    derives.dedup();
    derives
}

fn parse_attribute_texts(attributes: &[Node<'_>], source: &str) -> Vec<String> {
    let mut parsed = Vec::new();

    for attribute in attributes {
        let Some(text) = source.get(attribute.byte_range()) else {
            continue;
        };
        let normalized = text
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        if !normalized.is_empty() {
            parsed.push(normalized);
        }
    }

    parsed.sort();
    parsed.dedup();
    parsed
}

fn collect_static_summaries(root: Node<'_>, source: &str) -> Vec<RustStaticSummary> {
    let mut statics = Vec::new();
    visit_for_static_summaries(root, source, &mut statics);
    statics.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    statics
}

fn visit_for_static_summaries(node: Node<'_>, source: &str, statics: &mut Vec<RustStaticSummary>) {
    if node.kind() == "static_item"
        && let Some(summary) = build_static_summary(node, source)
    {
        statics.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_static_summaries(child, source, statics);
    }
}

fn build_static_summary(node: Node<'_>, source: &str) -> Option<RustStaticSummary> {
    let name = node
        .child_by_field_name("name")
        .and_then(|name_node| source.get(name_node.byte_range()))?
        .trim()
        .to_string();
    let type_text = node
        .child_by_field_name("type")
        .and_then(|type_node| source.get(type_node.byte_range()))?
        .trim()
        .to_string();
    let value_text = node
        .child_by_field_name("value")
        .and_then(|value_node| source.get(value_node.byte_range()))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    Some(RustStaticSummary {
        line: node.start_position().row + 1,
        name,
        type_text,
        value_text,
        visibility_pub: source.get(node.byte_range()).is_some_and(|text| {
            text.trim_start().starts_with("pub ") || text.trim_start().starts_with("pub(")
        }),
    })
}

fn collect_enum_summaries(root: Node<'_>, source: &str) -> Vec<RustEnumSummary> {
    let mut enums = Vec::new();
    visit_for_enum_summaries(root, source, &mut enums);
    enums.sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    enums
}

fn visit_for_enum_summaries(node: Node<'_>, source: &str, enums: &mut Vec<RustEnumSummary>) {
    if node.kind() == "enum_item"
        && let Some(summary) = build_enum_summary(node, source)
    {
        enums.push(summary);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_enum_summaries(child, source, enums);
    }
}

fn build_enum_summary(node: Node<'_>, source: &str) -> Option<RustEnumSummary> {
    let name = node
        .child_by_field_name("name")
        .and_then(|name_node| source.get(name_node.byte_range()))?
        .trim()
        .to_string();
    let derives = parse_derive_names(&leading_attributes(node), source);
    let attributes = parse_attribute_texts(&leading_attributes(node), source);
    let variant_count = node
        .child_by_field_name("body")
        .or_else(|| named_child_by_kind(node, "enum_variant_list"))
        .map(|body| {
            let mut cursor = body.walk();
            body.named_children(&mut cursor)
                .filter(|child| child.kind() == "enum_variant")
                .count()
        })
        .unwrap_or(0);

    Some(RustEnumSummary {
        line: node.start_position().row + 1,
        name,
        variant_count,
        has_serialize_derive: derives.iter().any(|derive| derive == "Serialize"),
        has_deserialize_derive: derives.iter().any(|derive| derive == "Deserialize"),
        derives,
        attributes,
        visibility_pub: source.get(node.byte_range()).is_some_and(|text| {
            text.trim_start().starts_with("pub ") || text.trim_start().starts_with("pub(")
        }),
    })
}

fn function_is_async(function_node: Node<'_>, body_node: Node<'_>, source: &str) -> bool {
    source
        .get(function_node.start_byte()..body_node.start_byte())
        .unwrap_or("")
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == "async")
}

fn collect_await_points(node: Node<'_>) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_for_await_points(node, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_for_await_points(node: Node<'_>, lines: &mut Vec<usize>) {
    if node.kind() == "await_expression" {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_await_points(child, lines);
    }
}

fn collect_macro_calls(calls: &[CallSite]) -> Vec<MacroCall> {
    calls
        .iter()
        .filter(|call| call.name.ends_with('!'))
        .map(|call| MacroCall {
            line: call.line,
            name: call.name.clone(),
        })
        .collect()
}

fn collect_named_runtime_calls(calls: &[CallSite], names: &[&str]) -> Vec<RuntimeCall> {
    calls
        .iter()
        .filter(|call| names.contains(&call.name.as_str()))
        .map(|call| RuntimeCall {
            line: call.line,
            name: call.name.clone(),
            receiver: call.receiver.clone(),
        })
        .collect()
}

fn collect_future_creations(node: Node<'_>, source: &str) -> Vec<RuntimeCall> {
    let mut futures = Vec::new();
    visit_for_future_creations(node, source, &mut futures);
    futures
}

fn visit_for_future_creations(node: Node<'_>, source: &str, futures: &mut Vec<RuntimeCall>) {
    if node.kind() == "let_declaration"
        && let Some(value_node) = node.child_by_field_name("value")
        && let Some(value_text) = source.get(value_node.byte_range())
    {
        let trimmed = value_text.trim();
        if trimmed.starts_with("async ")
            || trimmed.contains(".fuse()")
            || trimmed.contains("Future")
        {
            futures.push(RuntimeCall {
                line: node.start_position().row + 1,
                name: "future".to_string(),
                receiver: None,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_future_creations(child, source, futures);
    }
}

fn collect_blocking_calls(node: Node<'_>, source: &str, calls: &[CallSite]) -> Vec<RuntimeCall> {
    let mut blocking_calls = calls
        .iter()
        .filter(|call| is_blocking_call(call))
        .map(|call| RuntimeCall {
            line: call.line,
            name: call.name.clone(),
            receiver: call.receiver.clone(),
        })
        .collect::<Vec<_>>();

    visit_textual_blocking_calls(node, source, &mut blocking_calls);
    blocking_calls
        .sort_by(|left, right| left.line.cmp(&right.line).then(left.name.cmp(&right.name)));
    blocking_calls.dedup_by(|left, right| {
        left.line == right.line && left.name == right.name && left.receiver == right.receiver
    });
    blocking_calls
}

fn is_blocking_call(call: &CallSite) -> bool {
    let receiver = call.receiver.as_deref().unwrap_or_default();
    matches!(
        call.name.as_str(),
        "read_to_string"
            | "read"
            | "read_to_end"
            | "write"
            | "write_all"
            | "open"
            | "create"
            | "metadata"
            | "sleep"
            | "join"
            | "block_on"
    ) || receiver.contains("std::fs")
        || receiver.contains("fs")
        || receiver.contains("std::thread")
        || receiver.contains("File")
}

fn visit_textual_blocking_calls(
    node: Node<'_>,
    source: &str,
    blocking_calls: &mut Vec<RuntimeCall>,
) {
    if let Some(text) = source.get(node.byte_range()) {
        let blocking_name = if text.contains("std::thread::sleep") {
            Some("sleep")
        } else if text.contains("std::fs::") || text.contains("fs::read_to_string") {
            Some("fs")
        } else if text.contains("block_on(") {
            Some("block_on")
        } else {
            None
        };

        if let Some(name) = blocking_name {
            blocking_calls.push(RuntimeCall {
                line: node.start_position().row + 1,
                name: name.to_string(),
                receiver: None,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_textual_blocking_calls(child, source, blocking_calls);
    }
}

fn collect_loop_operation_lines(
    node: Node<'_>,
    source: &str,
) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let mut write_loops = Vec::new();
    let mut line_iteration_loops = Vec::new();
    let mut default_hasher_lines = Vec::new();
    visit_loop_operation_lines(
        node,
        source,
        false,
        &mut write_loops,
        &mut line_iteration_loops,
        &mut default_hasher_lines,
    );
    (write_loops, line_iteration_loops, default_hasher_lines)
}

fn visit_loop_operation_lines(
    node: Node<'_>,
    source: &str,
    in_loop: bool,
    write_loops: &mut Vec<usize>,
    line_iteration_loops: &mut Vec<usize>,
    default_hasher_lines: &mut Vec<usize>,
) {
    let child_in_loop = in_loop || is_loop_node(node.kind());

    if child_in_loop
        && node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
    {
        let target = render_call_target(function_node, source);
        let (_, name) = split_call_target(&target);
        let line = node.start_position().row + 1;
        if matches!(name.as_str(), "write" | "write_all") || target.contains("File::write") {
            write_loops.push(line);
        }
        if name == "lines" {
            line_iteration_loops.push(line);
        }
        if is_default_hashmap_target(&target) {
            default_hasher_lines.push(line);
        }
    }

    if child_in_loop
        && node.kind() == "macro_invocation"
        && let Some(macro_node) = node.child_by_field_name("macro")
        && let Some(macro_text) = source.get(macro_node.byte_range())
        && matches!(macro_text.trim(), "write" | "writeln")
    {
        write_loops.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_loop_operation_lines(
            child,
            source,
            child_in_loop,
            write_loops,
            line_iteration_loops,
            default_hasher_lines,
        );
    }
}

fn is_loop_node(kind: &str) -> bool {
    matches!(
        kind,
        "for_expression" | "while_expression" | "loop_expression"
    )
}

fn collect_boxed_container_lines(node: Node<'_>, source: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    visit_boxed_vec_lines(node, source, &mut lines);
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn visit_boxed_vec_lines(node: Node<'_>, source: &str, lines: &mut Vec<usize>) {
    if node.kind() == "let_declaration"
        && let Some(text) = source.get(node.byte_range())
        && contains_boxed_vec_type(text)
    {
        lines.push(node.start_position().row + 1);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_boxed_vec_lines(child, source, lines);
    }
}

fn contains_boxed_vec_type(text: &str) -> bool {
    let Some(vec_start) = text.find("Vec<") else {
        return false;
    };
    text[vec_start + 4..].contains("Box<")
}

fn is_default_hashmap_target(target: &str) -> bool {
    target.contains("HashMap::") && (target.ends_with("::new") || target.ends_with("::default"))
}

fn collect_unsafe_patterns(node: Node<'_>, source: &str) -> Vec<UnsafePattern> {
    let mut patterns = Vec::new();
    visit_for_unsafe_patterns(node, source, &mut patterns);
    patterns.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.detail.cmp(&right.detail))
    });
    patterns.dedup_by(|left, right| {
        left.line == right.line && left.kind == right.kind && left.detail == right.detail
    });
    patterns
}

fn visit_for_unsafe_patterns(node: Node<'_>, source: &str, patterns: &mut Vec<UnsafePattern>) {
    if node.kind() == "call_expression"
        && let Some(function_node) = node.child_by_field_name("function")
    {
        let target = render_call_target(function_node, source);
        let (_, name) = split_call_target(&target);
        let kind = match name.as_str() {
            "get_unchecked" | "get_unchecked_mut" => Some(UnsafePatternKind::GetUnchecked),
            "from_raw_parts" | "from_raw_parts_mut" => Some(UnsafePatternKind::RawParts),
            "set_len" => Some(UnsafePatternKind::SetLen),
            "assume_init" => Some(UnsafePatternKind::AssumeInit),
            "transmute" => Some(UnsafePatternKind::Transmute),
            _ => None,
        };

        if let Some(kind) = kind {
            patterns.push(UnsafePattern {
                line: node.start_position().row + 1,
                kind,
                detail: target,
            });
        }
    }

    if matches!(node.kind(), "cast_expression" | "type_cast_expression")
        && let Some(text) = source.get(node.byte_range())
        && (text.contains(" as *const ")
            || text.contains(" as *mut ")
            || text.ends_with(" as *const")
            || text.ends_with(" as *mut"))
    {
        patterns.push(UnsafePattern {
            line: node.start_position().row + 1,
            kind: UnsafePatternKind::RawPointerCast,
            detail: text.trim().to_string(),
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_for_unsafe_patterns(child, source, patterns);
    }
}

fn inside_trait_impl(node: Node<'_>, source: &str, trait_name: &str) -> bool {
    let mut current = node.parent();

    while let Some(parent) = current {
        if parent.kind() == "impl_item" && trait_impl_type(parent, source, trait_name).is_some() {
            return true;
        }
        current = parent.parent();
    }

    false
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
        return text.get(1..text.len() - 1).map(ToOwned::to_owned);
    }

    if text.starts_with('r') {
        let start = text.find('"')?;
        let end = text.rfind('"')?;
        if end > start {
            return text.get(start + 1..end).map(ToOwned::to_owned);
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

    for line in text.split('\n') {
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

    for line in text.split('\n') {
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
    fn test_collects_advanceplan2_rust_summaries() {
        let source = r#"
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestConfig {
    #[serde(default)]
    pub mode: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum WireValue {
    Text(String),
    Count(u64),
}

pub static CACHE: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
"#;

        let parsed = parse_file(Path::new("src/lib.rs"), source)
            .expect("rust source should parse successfully");

        let config = parsed
            .structs
            .iter()
            .find(|summary| summary.name == "RequestConfig")
            .expect("RequestConfig should be summarized");
        assert!(
            config
                .attributes
                .iter()
                .any(|attribute| attribute.contains("deny_unknown_fields"))
        );
        assert!(
            config.fields[0]
                .attributes
                .iter()
                .any(|attribute| attribute.contains("serde(default)"))
        );
        assert!(
            config.fields[1]
                .attributes
                .iter()
                .any(|attribute| attribute.contains("serde(flatten)"))
        );

        assert_eq!(parsed.rust_enums.len(), 1);
        assert_eq!(parsed.rust_enums[0].name, "WireValue");
        assert_eq!(parsed.rust_enums[0].variant_count, 2);
        assert!(
            parsed.rust_enums[0]
                .attributes
                .iter()
                .any(|attribute| attribute.contains("untagged"))
        );

        assert_eq!(parsed.rust_statics.len(), 1);
        assert_eq!(parsed.rust_statics[0].name, "CACHE");
        assert!(parsed.rust_statics[0].type_text.contains("OnceLock"));
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
