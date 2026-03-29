use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_inconsistent_package_names() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/alpha.go",
        go_fixture!("package_conflict_a.txt"),
    );
    write_fixture(
        &temp_dir,
        "pkg/beta_test.go",
        go_fixture!("package_conflict_b.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "inconsistent_package_name")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_package_test_suffix_is_allowed() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/alpha.go",
        go_fixture!("package_conflict_a.txt"),
    );
    write_fixture(
        &temp_dir,
        "pkg/beta_test.go",
        go_fixture!("package_conflict_test_variant.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "inconsistent_package_name")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_misgrouped_imports() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "imports.go",
        go_fixture!("import_misgrouped.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "misgrouped_imports")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_grouped_imports() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "imports.go", go_fixture!("import_grouped.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "misgrouped_imports")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_style_rules_respect_repository_config() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        ".deslop.toml",
        "disabled_rules = [\"misgrouped_imports\"]\n[severity_overrides]\ninconsistent_package_name = \"error\"\n",
    );
    write_fixture(
        &temp_dir,
        "pkg/alpha.go",
        go_fixture!("package_conflict_a.txt"),
    );
    write_fixture(
        &temp_dir,
        "pkg/beta_test.go",
        go_fixture!("package_conflict_b.txt"),
    );
    write_fixture(
        &temp_dir,
        "imports.go",
        go_fixture!("import_misgrouped.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
