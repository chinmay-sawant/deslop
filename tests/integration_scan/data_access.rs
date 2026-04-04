use super::FixtureWorkspace;

#[test]
fn test_db_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("db.go", go_fixture!("db_query_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "n_plus_one_query")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "wide_select_query")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "likely_unindexed_query")
    );
}

#[test]
fn test_db_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("db.go", go_fixture!("db_query_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "n_plus_one_query")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "wide_select_query")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "likely_unindexed_query")
    );
}
