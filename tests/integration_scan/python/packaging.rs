use super::super::FixtureWorkspace;

fn assert_rules_present(report: &deslop::ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == *rule_id),
            "expected rule {rule_id} to fire"
        );
    }
}

fn assert_rules_absent(report: &deslop::ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == *rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }
}

const PACKAGING_RULES: &[&str] = &[
    "python_public_api_any_contract",
    "pyproject_missing_requires_python",
    "pyproject_script_entrypoint_unresolved",
    "cross_package_internal_import",
];

#[test]
fn test_python_packaging_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "pyproject.toml",
            python_fixture!("integration/packaging/pyproject_positive.txt"),
        ),
        (
            "pkg/cli.py",
            python_fixture!("integration/packaging/cli_positive.txt"),
        ),
        (
            "pkg/internal/admin.py",
            python_fixture!("integration/packaging/internal_admin.txt"),
        ),
        (
            "service/consumer.py",
            python_fixture!("integration/packaging/consumer_positive.txt"),
        ),
    ]);

    let report = workspace.scan();

    assert_rules_present(&report, PACKAGING_RULES);
}

#[test]
fn test_python_packaging_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "pyproject.toml",
            python_fixture!("integration/packaging/pyproject_clean.txt"),
        ),
        (
            "pkg/cli.py",
            python_fixture!("integration/packaging/cli_clean.txt"),
        ),
    ]);

    let report = workspace.scan();

    assert_rules_absent(&report, PACKAGING_RULES);
}

#[test]
fn test_python_packaging_skips_serializer_to_dict_any_contract() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[(
        "pkg/types.py",
        python_fixture!("integration/packaging/to_dict_serializer_clean.txt"),
    )]);

    let report = workspace.scan();

    assert_rules_absent(&report, &["python_public_api_any_contract"]);
}
