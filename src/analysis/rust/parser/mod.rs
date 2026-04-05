mod imports;
mod items;
#[cfg(test)]
mod tests;

use std::path::Path;

use tree_sitter::Parser;

use crate::analysis::{
    AnalysisResult, Error, Language, LanguageFileData, ParsedFile, RustFileData,
};

use self::imports::collect_imports;
use self::items::{
    collect_attribute_summaries, collect_enum_summaries, collect_module_declarations,
    collect_pkg_strings, collect_static_summaries, collect_struct_summaries, collect_symbols,
    collect_trait_impls,
};

mod functions;

use self::functions::{collect_functions, is_test_file, module_name_for_path};
pub(crate) use functions::{is_inside_function, leading_attributes, string_literal_value};

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
    let attributes = collect_attribute_summaries(root, source);
    let module_declarations = collect_module_declarations(root, source);

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
        functions,
        imports,
        symbols,
        module_scope_calls: Vec::new(),
        top_level_bindings: Vec::new(),
        lang: LanguageFileData::Rust(RustFileData {
            rust_statics,
            rust_enums,
            structs,
            attributes,
            module_declarations,
        }),
    })
}
