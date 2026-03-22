use std::fs;

use deslop::{scan_repository, ScanOptions};

use super::{create_temp_workspace, write_fixture};

#[test]
fn flags_string_concat_in_loops() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "concat.go",
        go_fixture!("string_concat_loop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "string_concat_in_loop"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_numeric_plus_equals_in_loops() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "concat.go",
        go_fixture!("string_concat_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "string_concat_in_loop"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_repeated_json_marshaling_inside_loops() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "json.go",
        go_fixture!("json_marshal_loop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "repeated_json_marshaling"
            && finding.function_name.as_deref() == Some("EncodeAll")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_single_json_marshaling_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "json.go",
        go_fixture!("json_marshal_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "repeated_json_marshaling"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_hot_path_allocation_fmt_and_reflection_patterns() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "hot_path.go",
        go_fixture!("hot_path_slop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "allocation_churn_in_loop"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "fmt_hot_path"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "reflection_hot_path"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_hot_path_rules_for_one_off_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "hot_path.go",
        go_fixture!("hot_path_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "allocation_churn_in_loop"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "fmt_hot_path"));
    assert!(!report.findings.iter().any(|finding| finding.rule_id == "reflection_hot_path"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_full_dataset_load_patterns() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "full_dataset.go",
        go_fixture!("full_dataset_load_slop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "full_dataset_load"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_streaming_reads_as_full_dataset_loads() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "full_dataset.go",
        go_fixture!("full_dataset_load_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| finding.rule_id == "full_dataset_load"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
