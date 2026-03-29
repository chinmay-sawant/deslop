use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_concat_loops() {
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

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_numeric_ok() {
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

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_json_loops() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "json.go", go_fixture!("json_marshal_loop.txt"));

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
fn test_json_ok() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "json.go", go_fixture!("json_marshal_clean.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "repeated_json_marshaling")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_hot_path() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "hot_path.go", go_fixture!("hot_path_slop.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "allocation_churn_in_loop")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "fmt_hot_path")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "reflection_hot_path")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_hot_path_ok() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "hot_path.go", go_fixture!("hot_path_clean.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "allocation_churn_in_loop")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "fmt_hot_path")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "reflection_hot_path")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_dataset_load() {
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

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_streaming_ok() {
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

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_semantic_n_squared_rules_are_opt_in() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "alloc.go",
        go_fixture!("n_squared_alloc_slop.txt"),
    );
    write_fixture(
        &temp_dir,
        "concat.go",
        go_fixture!("n_squared_concat_slop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report.findings.iter().any(|finding| {
            matches!(
                finding.rule_id.as_str(),
                "likely_n_squared_allocation" | "likely_n_squared_string_concat"
            )
        })
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_semantic_n_squared_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        ".deslop.toml",
        "go_semantic_experimental = true\n",
    );
    write_fixture(
        &temp_dir,
        "alloc.go",
        go_fixture!("n_squared_alloc_slop.txt"),
    );
    write_fixture(
        &temp_dir,
        "concat.go",
        go_fixture!("n_squared_concat_slop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "likely_n_squared_allocation"
            && finding.function_name.as_deref() == Some("Expand")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "likely_n_squared_string_concat"
            && finding.function_name.as_deref() == Some("Render")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_semantic_n_squared_clean_fixtures() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        ".deslop.toml",
        "go_semantic_experimental = true\n",
    );
    write_fixture(
        &temp_dir,
        "alloc.go",
        go_fixture!("n_squared_alloc_clean.txt"),
    );
    write_fixture(
        &temp_dir,
        "concat.go",
        go_fixture!("n_squared_concat_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report.findings.iter().any(|finding| {
            matches!(
                finding.rule_id.as_str(),
                "likely_n_squared_allocation" | "likely_n_squared_string_concat"
            )
        })
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_semantic_nested_query_escalation() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        ".deslop.toml",
        "go_semantic_experimental = true\n",
    );
    write_fixture(
        &temp_dir,
        "query.go",
        go_fixture!("n_squared_query_slop.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "n_plus_one_query"
            && matches!(finding.severity, deslop::Severity::Error)
            && finding.function_name.as_deref() == Some("Load")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_semantic_nested_query_clean() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        ".deslop.toml",
        "go_semantic_experimental = true\n",
    );
    write_fixture(
        &temp_dir,
        "query.go",
        go_fixture!("n_squared_query_clean.txt"),
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
            .any(|finding| finding.rule_id == "n_plus_one_query")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
