use std::fs;

use deslop::{scan_repository, ScanOptions};

use super::{create_temp_workspace, write_fixture};

#[test]
fn flags_generic_names_and_weak_typing() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "sloppy.go",
        go_fixture!("generic_weak.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "generic_name"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "weak_typing"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_comment_style_and_overlong_names() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "comments.go",
        go_fixture!("comment_slop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "comment_style_title_case"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "comment_style_tutorial"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "overlong_name"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_concise_comments() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "comments.go",
        go_fixture!("comment_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "comment_style_title_case"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "comment_style_tutorial"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_legitimate_handler_names() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "handler.go",
        go_fixture!("legitimate_handler.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "generic_name"
            && finding.function_name.as_deref() == Some("HandleRequest")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_legitimate_adapter_names() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "adapter.go",
        go_fixture!("legitimate_adapter.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "generic_name"
            && finding.function_name.as_deref() == Some("ConvertValue")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
