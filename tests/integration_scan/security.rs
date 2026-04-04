use super::FixtureWorkspace;

#[test]
fn flags_weak_crypto_usage() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("crypto.go", go_fixture!("weak_crypto.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "weak_crypto")
    );
}

#[test]
fn test_secrets() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("secrets.go", go_fixture!("hardcoded_secret_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hardcoded_secret")
    );
}

#[test]
fn test_env_secrets() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("secrets.go", go_fixture!("hardcoded_secret_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hardcoded_secret")
    );
}

#[test]
fn test_sql_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("query.go", go_fixture!("sql_string_concat_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "sql_string_concat")
    );
}

#[test]
fn test_sql_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("query.go", go_fixture!("sql_string_concat_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "sql_string_concat")
    );
}
