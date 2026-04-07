use std::path::Path;

use super::parser;
use crate::analysis::AnalysisConfig;
use crate::analysis::test_support::{find_parsed_file, parse_fixture_bundle};
use crate::heuristics::evaluate_file;
use crate::index::build_repository_index;

fn parse_bundle(relative_path: &str) -> Vec<crate::analysis::ParsedFile> {
    parse_fixture_bundle(relative_path, parser::parse_file)
}

#[test]
fn imported_package_reexports_do_not_trigger_hallucinated_import_calls() {
    let files = parse_bundle("python/backend/imported_package_reexports.txt");
    let current = find_parsed_file(&files, "/repo/tests/test_widgets.py");
    let index = build_repository_index(Path::new("/repo"), &files);
    let findings = evaluate_file(current, &index, &AnalysisConfig::default());

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
    let files = parse_bundle("python/backend/parenthesized_from_import.txt");
    let current = find_parsed_file(&files, "/repo/examples/bench.py");
    let index = build_repository_index(Path::new("/repo"), &files);
    let findings = evaluate_file(current, &index, &AnalysisConfig::default());

    assert!(
        !findings.iter().any(|finding| {
            (finding.rule_id == "hallucinated_import_call"
                || finding.rule_id == "hallucinated_local_call")
                && finding.message.contains("BookmarkNode")
        }),
        "parenthesized from-import should resolve imported names: {findings:?}"
    );
}
