mod comments;
mod general;
mod performance;
mod phase4;
#[cfg(test)]
mod tests;

use std::path::Path;

use tree_sitter::Parser;

use crate::analysis::{AnalysisResult, Error, Language, ParsedFile};

use self::comments::collect_comment_summaries;
use self::general::{
    collect_class_summaries, collect_functions, collect_imports, collect_module_scope_calls,
    collect_pkg_strings, collect_python_models, collect_symbols, collect_top_level_bindings,
    is_test_file, module_name_for_path,
};

pub(super) fn parse_file(path: &Path, source: &str) -> AnalysisResult<ParsedFile> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .map_err(|error| Error::parser_configuration("Python", error.to_string()))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| Error::missing_parse_tree("Python"))?;

    let root = tree.root_node();
    let is_test_file = is_test_file(path);
    let mut imports = collect_imports(root, source);
    if is_package_export_module(path) {
        mark_public_reexports(&mut imports);
    }
    let package_string_literals = collect_pkg_strings(root, source);
    let comments = collect_comment_summaries(source);
    let functions = collect_functions(root, source, is_test_file);
    let symbols = collect_symbols(root, source, &functions, &imports);
    let class_summaries = collect_class_summaries(root, source);
    let module_scope_calls = collect_module_scope_calls(root, source);
    let top_level_bindings = collect_top_level_bindings(root, source);
    let python_models = collect_python_models(root, source);

    Ok(ParsedFile {
        language: Language::Python,
        path: path.to_path_buf(),
        package_name: module_name_for_path(path),
        is_test_file,
        syntax_error: root.has_error(),
        line_count: source.lines().count(),
        byte_size: source.len(),
        pkg_strings: package_string_literals,
        comments,
        struct_tags: Vec::new(),
        functions,
        imports,
        symbols,
        class_summaries,
        package_vars: Vec::new(),
        interfaces: Vec::new(),
        go_structs: Vec::new(),
        module_scope_calls,
        top_level_bindings,
        python_models,
        rust_statics: Vec::new(),
        rust_enums: Vec::new(),
        structs: Vec::new(),
    })
}

fn is_package_export_module(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("__init__.py")
}

fn mark_public_reexports(imports: &mut [crate::analysis::ImportSpec]) {
    for import in imports {
        if import.alias != "*" && !import.alias.starts_with('_') && import.path.starts_with('.') {
            import.is_public = true;
        }
    }
}
