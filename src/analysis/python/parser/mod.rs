mod comments;
mod general;
mod phase4;
mod performance;
#[cfg(test)]
mod tests;

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use tree_sitter::Parser;

use crate::analysis::{Language, ParsedFile};

use self::comments::collect_comment_summaries;
use self::general::{
    collect_class_summaries, collect_functions, collect_imports, collect_pkg_strings,
    collect_symbols, is_test_file, module_name_for_path,
};

pub(super) fn parse_file(path: &Path, source: &str) -> Result<ParsedFile> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .map_err(|error| anyhow!(error.to_string()))
        .context("failed to configure Python parser")?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("tree-sitter returned no parse tree"))?;

    let root = tree.root_node();
    let is_test_file = is_test_file(path);
    let imports = collect_imports(root, source);
    let package_string_literals = collect_pkg_strings(root, source);
    let comments = collect_comment_summaries(source);
    let functions = collect_functions(root, source, is_test_file);
    let symbols = collect_symbols(root, source, &functions);
    let class_summaries = collect_class_summaries(root, source);

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
    })
}
