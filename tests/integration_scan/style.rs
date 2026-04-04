
use deslop::{ScanOptions, scan_repository};

use super::FixtureWorkspace;

#[test]
fn test_inconsistent_package_names() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("pkg/alpha.go",
        go_fixture!("package_conflict_a.txt"),
    );
    workspace.write_file("pkg/beta_test.go",
        go_fixture!("package_conflict_b.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "inconsistent_package_name")
    );

    }

#[test]
fn test_package_test_suffix_is_allowed() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("pkg/alpha.go",
        go_fixture!("package_conflict_a.txt"),
    );
    workspace.write_file("pkg/beta_test.go",
        go_fixture!("package_conflict_test_variant.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "inconsistent_package_name")
    );

    }

#[test]
fn test_misgrouped_imports() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("imports.go",
        go_fixture!("import_misgrouped.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "misgrouped_imports")
    );

    }

#[test]
fn test_grouped_imports() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("imports.go", go_fixture!("import_grouped.txt"));

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "misgrouped_imports")
    );

    }

#[test]
fn test_style_rules_respect_repository_config() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml",
        "disabled_rules = [\"misgrouped_imports\"]\n[severity_overrides]\ninconsistent_package_name = \"error\"\n",
    );
    workspace.write_file("pkg/alpha.go",
        go_fixture!("package_conflict_a.txt"),
    );
    workspace.write_file("pkg/beta_test.go",
        go_fixture!("package_conflict_b.txt"),
    );
    workspace.write_file("imports.go",
        go_fixture!("import_misgrouped.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "misgrouped_imports")
    );
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "inconsistent_package_name"
            && matches!(finding.severity, deslop::Severity::Error)
    }));

    }
