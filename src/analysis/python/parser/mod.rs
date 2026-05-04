mod comments;
mod functions;
mod general;
mod hotpath;
mod hotpath_ext;
mod performance;
mod phase4;

use std::cell::RefCell;
use std::path::Path;

use tree_sitter::Parser;

use crate::analysis::{
    AnalysisResult, Error, Language, LanguageFileData, ParsedFile, PythonFileData,
};

use self::comments::collect_comment_summaries;
use self::functions::collect_functions;
use self::general::{
    collect_class_summaries, collect_imports, collect_module_scope_calls, collect_pkg_strings,
    collect_python_models, collect_symbols, collect_top_level_bindings, is_test_file,
    module_name_for_path,
};

thread_local! {
    static PYTHON_PARSER: RefCell<Option<Parser>> = const { RefCell::new(None) };
}

pub(super) fn parse_file(path: &Path, source: &str) -> AnalysisResult<ParsedFile> {
    let tree = with_python_parser(|parser| {
        parser
            .parse(source, None)
            .ok_or_else(|| Error::missing_parse_tree("Python"))
    })?;

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
        functions,
        imports,
        symbols,
        module_scope_calls,
        top_level_bindings,
        lang: LanguageFileData::Python(PythonFileData {
            class_summaries,
            python_models,
        }),
    })
}

fn with_python_parser<T>(
    action: impl FnOnce(&mut Parser) -> AnalysisResult<T>,
) -> AnalysisResult<T> {
    PYTHON_PARSER.with(|parser_cell| {
        let mut parser_slot = parser_cell.borrow_mut();
        if parser_slot.is_none() {
            let mut parser = Parser::new();
            parser
                .set_language(&tree_sitter_python::LANGUAGE.into())
                .map_err(|error| Error::parser_configuration("Python", error.to_string()))?;
            *parser_slot = Some(parser);
        }

        let Some(parser) = parser_slot.as_mut() else {
            return Err(Error::parser_configuration(
                "Python",
                "thread-local parser cache was not initialized",
            ));
        };

        action(parser)
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
