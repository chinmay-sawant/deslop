#[path = "general/calls.rs"]
mod calls;
#[path = "general/imports.rs"]
mod imports;
#[path = "general/literals.rs"]
mod literals;
#[path = "general/symbols.rs"]
mod symbols;

pub(super) use calls::{collect_calls, extract_call_target};
pub(super) use imports::collect_imports;
pub(super) use literals::{
    build_test_summary, collect_local_strings, collect_pkg_strings, collect_struct_tags,
    first_string_literal,
};
pub(super) use symbols::{
    collect_expression_nodes, collect_go_structs, collect_identifiers, collect_interface_summaries,
    collect_package_vars, collect_symbols, count_descendants, extract_receiver, find_package_name,
    find_var_name_node, is_identifier_name, split_assignment,
};
