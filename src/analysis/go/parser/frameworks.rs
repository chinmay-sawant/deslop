use tree_sitter::Node;

use crate::analysis::{GinCallSummary, GormChainStep, GormQueryChain, ImportSpec, ParseInputCall};

use super::general::{
    collect_expression_nodes, extract_call_target, first_string_literal, is_identifier_name,
    split_assignment,
};

const GORM_TERMINAL_METHODS: &[&str] = &[
    "Count",
    "Create",
    "CreateInBatches",
    "Delete",
    "Find",
    "FindInBatches",
    "First",
    "FirstOrCreate",
    "Rows",
    "Save",
    "Scan",
    "Take",
    "Update",
    "UpdateColumn",
    "Updates",
];

const GORM_QUERY_METHODS: &[&str] = &[
    "Association",
    "Clauses",
    "Count",
    "Create",
    "CreateInBatches",
    "Delete",
    "Debug",
    "Distinct",
    "Find",
    "FindInBatches",
    "First",
    "FirstOrCreate",
    "Group",
    "Joins",
    "Limit",
    "Model",
    "Offset",
    "Omit",
    "Order",
    "Preload",
    "Raw",
    "Rows",
    "Save",
    "Scan",
    "Scopes",
    "Select",
    "Session",
    "Table",
    "Take",
    "Update",
    "UpdateColumn",
    "Updates",
    "Where",
    "WithContext",
];

const YAML_IMPORT_PATHS: &[&str] = &[
    "gopkg.in/yaml.v2",
    "gopkg.in/yaml.v3",
    "sigs.k8s.io/yaml",
];

const PROTO_IMPORT_PATHS: &[&str] = &[
    "google.golang.org/protobuf/proto",
    "github.com/golang/protobuf/proto",
];

pub(super) fn collect_gorm_query_chains(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<GormQueryChain> {
    if !imports.iter().any(|import| import.path == "gorm.io/gorm") {
        return Vec::new();
    }

    let mut chains = Vec::new();
    visit_gorm_query_chains(body_node, source, false, &mut chains);
    chains
}

fn visit_gorm_query_chains(
    node: Node<'_>,
    source: &str,
    inside_loop: bool,
    chains: &mut Vec<GormQueryChain>,
) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if node.kind() == "call_expression"
        && !is_chained_subcall(node)
        && let Some((root_text, steps)) = extract_call_chain(node, source)
        && is_gorm_chain_candidate(&steps)
    {
        let terminal_method = steps
            .last()
            .map(|step| step.method_name.clone())
            .unwrap_or_default();
        chains.push(GormQueryChain {
            line: node.start_position().row + 1,
            root_text,
            terminal_method,
            steps,
            in_loop: next_inside_loop,
        });
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_gorm_query_chains(child, source, next_inside_loop, chains);
    }
}

pub(super) fn collect_parse_input_calls(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<ParseInputCall> {
    let mut calls = Vec::new();
    visit_parse_input_calls(body_node, source, imports, false, &mut calls);
    calls
}

fn visit_parse_input_calls(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
    inside_loop: bool,
    calls: &mut Vec<ParseInputCall>,
) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if node.kind() == "call_expression"
        && let Some(call) = parse_input_call(node, source, imports, next_inside_loop)
    {
        calls.push(call);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_parse_input_calls(child, source, imports, next_inside_loop, calls);
    }
}

pub(super) fn collect_gin_calls(
    body_node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
) -> Vec<GinCallSummary> {
    if !imports
        .iter()
        .any(|import| import.path == "github.com/gin-gonic/gin")
    {
        return Vec::new();
    }

    let mut calls = Vec::new();
    visit_gin_calls(body_node, source, imports, false, &mut calls);
    calls
}

fn visit_gin_calls(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
    inside_loop: bool,
    calls: &mut Vec<GinCallSummary>,
) {
    let next_inside_loop = inside_loop || node.kind() == "for_statement";

    if node.kind() == "call_expression"
        && let Some(call) = parse_gin_call(node, source, imports, next_inside_loop)
    {
        calls.push(call);
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_gin_calls(child, source, imports, next_inside_loop, calls);
    }
}

fn parse_input_call(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
    in_loop: bool,
) -> Option<ParseInputCall> {
    let function_node = node.child_by_field_name("function")?;
    let arguments_node = node.child_by_field_name("arguments")?;
    let (receiver, name) = extract_call_target(function_node, source)?;
    let import_path = receiver
        .as_deref()
        .and_then(|receiver_name| resolve_import_path(receiver_name, imports));

    let parser_family = match (import_path, name.as_str()) {
        (Some("encoding/json"), "Unmarshal") => "json_unmarshal",
        (Some("encoding/xml"), "Unmarshal") => "xml_unmarshal",
        (Some(path), "Unmarshal" | "UnmarshalStrict") if YAML_IMPORT_PATHS.contains(&path) => {
            "yaml_unmarshal"
        }
        (Some(path), "Unmarshal") if PROTO_IMPORT_PATHS.contains(&path) => "proto_unmarshal",
        _ => return None,
    };

    let arguments = argument_nodes(arguments_node);
    let input_node = arguments.first().copied()?;
    let input_text = normalize_text(source.get(input_node.byte_range())?);
    let target_text = arguments
        .get(1)
        .and_then(|node| source.get(node.byte_range()))
        .map(normalize_text);

    Some(ParseInputCall {
        line: node.start_position().row + 1,
        parser_family: parser_family.to_string(),
        input_binding: simple_reference_text(&input_text),
        input_text,
        target_text,
        in_loop,
    })
}

fn parse_gin_call(
    node: Node<'_>,
    source: &str,
    imports: &[ImportSpec],
    in_loop: bool,
) -> Option<GinCallSummary> {
    let function_node = node.child_by_field_name("function")?;
    let arguments_node = node.child_by_field_name("arguments")?;
    let (receiver, name) = extract_call_target(function_node, source)?;
    let argument_texts = collect_argument_texts(arguments_node, source);
    let assigned_binding = assigned_binding_for_call(node, source);

    let operation = match name.as_str() {
        "GetRawData" => Some("get_raw_data"),
        "ShouldBind" => Some("should_bind"),
        "Bind" => Some("bind"),
        "ShouldBindQuery" => Some("should_bind_query"),
        "BindQuery" => Some("bind_query"),
        "ShouldBindJSON" => Some("should_bind_json"),
        "BindJSON" => Some("bind_json"),
        "ShouldBindXML" => Some("should_bind_xml"),
        "BindXML" => Some("bind_xml"),
        "ShouldBindYAML" => Some("should_bind_yaml"),
        "BindYAML" => Some("bind_yaml"),
        "ShouldBindTOML" => Some("should_bind_toml"),
        "BindTOML" => Some("bind_toml"),
        "ShouldBindBodyWith" => Some("should_bind_body_with"),
        "Copy" => Some("copy"),
        "IndentedJSON" => Some("indented_json"),
        "JSON" => Some("json"),
        "PureJSON" => Some("pure_json"),
        "Data" => Some("data"),
        _ => None,
    };

    if let Some(operation) = operation {
        return Some(GinCallSummary {
            line: node.start_position().row + 1,
            operation: operation.to_string(),
            argument_texts,
            assigned_binding,
            in_loop,
        });
    }

    let import_path = receiver
        .as_deref()
        .and_then(|receiver_name| resolve_import_path(receiver_name, imports));
    if matches!(import_path, Some("io") | Some("io/ioutil"))
        && name == "ReadAll"
        && argument_texts
            .first()
            .is_some_and(|argument| argument.contains(".Request.Body"))
    {
        return Some(GinCallSummary {
            line: node.start_position().row + 1,
            operation: "read_request_body".to_string(),
            argument_texts,
            assigned_binding,
            in_loop,
        });
    }

    None
}

fn extract_call_chain(call_node: Node<'_>, source: &str) -> Option<(String, Vec<GormChainStep>)> {
    let function_node = call_node.child_by_field_name("function")?;
    if function_node.kind() != "selector_expression" {
        return None;
    }

    let mut function_cursor = function_node.walk();
    let children = function_node
        .named_children(&mut function_cursor)
        .collect::<Vec<_>>();
    let receiver_node = *children.first()?;
    let field_node = *children.get(1)?;
    let method_name = source.get(field_node.byte_range())?.trim().to_string();
    let arguments_node = call_node.child_by_field_name("arguments")?;
    let step = GormChainStep {
        line: call_node.start_position().row + 1,
        method_name,
        argument_texts: collect_argument_texts(arguments_node, source),
        first_string_arg: first_string_literal(arguments_node, source),
    };

    if receiver_node.kind() == "call_expression" {
        let (root_text, mut steps) = extract_call_chain(receiver_node, source)?;
        steps.push(step);
        return Some((root_text, steps));
    }

    let root_text = normalize_text(source.get(receiver_node.byte_range())?);
    Some((root_text, vec![step]))
}

fn is_chained_subcall(node: Node<'_>) -> bool {
    node.parent().is_some_and(|parent| {
        parent.kind() == "selector_expression"
            && parent.parent().is_some_and(|grandparent| grandparent.kind() == "call_expression")
    })
}

fn is_gorm_chain_candidate(steps: &[GormChainStep]) -> bool {
    let Some(terminal_method) = steps.last().map(|step| step.method_name.as_str()) else {
        return false;
    };

    GORM_TERMINAL_METHODS.contains(&terminal_method)
        && steps
            .iter()
            .all(|step| GORM_QUERY_METHODS.contains(&step.method_name.as_str()))
}

fn collect_argument_texts(arguments_node: Node<'_>, source: &str) -> Vec<String> {
    argument_nodes(arguments_node)
        .into_iter()
        .filter_map(|argument| source.get(argument.byte_range()).map(normalize_text))
        .collect()
}

fn argument_nodes(arguments_node: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = arguments_node.walk();
    let arguments = arguments_node.named_children(&mut cursor).collect::<Vec<_>>();
    if arguments.is_empty() {
        collect_expression_nodes(arguments_node)
    } else {
        arguments
    }
}

fn assigned_binding_for_call(call_node: Node<'_>, source: &str) -> Option<String> {
    let owner = match call_node.parent() {
        Some(parent) if parent.kind() == "expression_list" => parent.parent(),
        other => other,
    }?;

    match owner.kind() {
        "assignment_statement" | "short_var_declaration" | "var_spec" => {
            let text = source.get(owner.byte_range())?;
            let (left, _) = split_assignment(text)?;
            let binding = left
                .trim()
                .trim_start_matches("var ")
                .split(',')
                .next()?
                .split_whitespace()
                .next()?
                .trim();
            is_identifier_name(binding).then(|| binding.to_string())
        }
        _ => None,
    }
}

fn resolve_import_path<'a>(receiver: &str, imports: &'a [ImportSpec]) -> Option<&'a str> {
    imports
        .iter()
        .find(|import| import.alias == receiver || import.path == receiver)
        .map(|import| import.path.as_str())
}

fn simple_reference_text(text: &str) -> Option<String> {
    let trimmed = text
        .trim()
        .trim_start_matches('&')
        .trim_start_matches('*')
        .trim();

    (!trimmed.is_empty()
        && trimmed
            .split('.')
            .all(|segment| is_identifier_name(segment.trim())))
        .then(|| trimmed.to_string())
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}