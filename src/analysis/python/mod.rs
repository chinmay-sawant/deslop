mod parser;

use std::path::Path;

use crate::analysis::{Language, LanguageBackend, ParsedFile};
use crate::heuristics::{evaluate_python_file, evaluate_python_repo};
use crate::index::RepositoryIndex;
use crate::model::Finding;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PythonAnalyzer;

impl LanguageBackend for PythonAnalyzer {
    fn language(&self) -> Language {
        Language::Python
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["py"]
    }

    fn supports_path(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()) == Some("py")
    }

    fn parse_file(&self, path: &Path, source: &str) -> crate::Result<ParsedFile> {
        parser::parse_file(path, source).map_err(crate::Error::from)
    }

    fn evaluate_file(&self, file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
        evaluate_python_file(file, index)
    }

    fn evaluate_repo(&self, files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
        evaluate_python_repo(files, index)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::parser;
    use crate::heuristics::evaluate_python_file;
    use crate::index::build_repository_index;

    fn parse_file(path: &str, source: &str) -> crate::analysis::ParsedFile {
        let parsed = parser::parse_file(Path::new(path), source);
        assert!(parsed.is_ok(), "python source should parse");
        match parsed {
            Ok(file) => file,
            Err(_) => unreachable!("asserted above"),
        }
    }

    #[test]
    fn imported_package_reexports_do_not_trigger_hallucinated_import_calls() {
        let package = parse_file(
            "/repo/pkg/widgets/__init__.py",
            r#"
from .types import WidgetTemplate, LayoutConfig, Heading
from .generator import render_widget
"#,
        );
        let types = parse_file(
            "/repo/pkg/widgets/types.py",
            r#"
class WidgetTemplate:
    pass

class LayoutConfig:
    pass

class Heading:
    pass
"#,
        );
        let generator = parse_file(
            "/repo/pkg/widgets/generator.py",
            r#"
def render_widget(template):
    return {"ok": True}
"#,
        );
        let current = parse_file(
            "/repo/tests/test_widgets.py",
            r#"
from widgets import render_widget, WidgetTemplate, LayoutConfig, Heading

def test_basic_render():
    template = WidgetTemplate(
        config=LayoutConfig(mode="grid"),
        heading=Heading(text="Test Widget"),
    )
    return render_widget(template)
"#,
        );

        let index = build_repository_index(
            Path::new("/repo"),
            &[current.clone(), package, types, generator],
        );
        let findings = evaluate_python_file(&current, &index);

        assert!(
            !findings.iter().any(|finding| {
                finding.rule_id == "hallucinated_import_call"
                    && (finding.message.contains("WidgetTemplate")
                        || finding.message.contains("LayoutConfig")
                        || finding.message.contains("Heading"))
            }),
            "package re-exports should resolve as imported symbols: {findings:?}"
        );
    }

    #[test]
    fn parenthesized_from_import_with_inline_comment_does_not_fall_back_to_local_call() {
        let package = parse_file(
            "/repo/pkg/widgets/__init__.py",
            r#"
from .types import BookmarkNode
"#,
        );
        let types = parse_file(
            "/repo/pkg/widgets/types.py",
            r#"
class BookmarkNode:
    pass
"#,
        );
        let current = parse_file(
            "/repo/examples/bench.py",
            r#"
from widgets import (  # benchmark imports
    BookmarkNode,
)

def bookmark_node() -> BookmarkNode:
    return BookmarkNode()
"#,
        );

        let index = build_repository_index(Path::new("/repo"), &[current.clone(), package, types]);
        let findings = evaluate_python_file(&current, &index);

        assert!(
            !findings.iter().any(|finding| {
                (finding.rule_id == "hallucinated_import_call"
                    || finding.rule_id == "hallucinated_local_call")
                    && finding.message.contains("BookmarkNode")
            }),
            "parenthesized from-import should resolve imported names: {findings:?}"
        );
    }
}
