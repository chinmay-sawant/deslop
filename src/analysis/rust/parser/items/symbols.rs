use tree_sitter::Node;

use crate::analysis::{DeclaredSymbol, ImportSpec, ParsedFunction};
use crate::model::SymbolKind;

pub(super) fn collect_symbols(
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

    for import in imports {
        if import.is_public && import.alias != "*" {
            symbols.push(DeclaredSymbol {
                name: import.alias.clone(),
                kind: SymbolKind::Function,
                receiver_type: None,
                receiver_is_pointer: None,
                line: 1,
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
