use super::FixtureWorkspace;

#[test]
fn test_naming_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("sloppy.go", go_fixture!("generic_weak.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "generic_name")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "weak_typing")
    );
}

#[test]
fn test_doc_overlong() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("comments.go", go_fixture!("comment_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "comment_style_title_case")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "comment_style_tutorial")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "overlong_name")
    );
}

#[test]
fn test_doc_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("comments.go", go_fixture!("comment_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "comment_style_title_case")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "comment_style_tutorial")
    );
}

#[test]
fn test_handler_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("handler.go", go_fixture!("legitimate_handler.txt"));

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "generic_name"
            && finding.function_name.as_deref() == Some("HandleRequest")
    }));
}

#[test]
fn test_adapter_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("adapter.go", go_fixture!("legitimate_adapter.txt"));

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "generic_name"
            && finding.function_name.as_deref() == Some("ConvertValue")
    }));
}
