use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

fn has_rule(report: &deslop::ScanReport, rule_id: &str) -> bool {
    report.findings.iter().any(|finding| finding.rule_id == rule_id)
}

#[test]
fn test_go_advanceplan3_core_hot_path_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "core_positive.go",
        go_fixture!("advanceplan3_core_positive.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "regexp_compile_in_hot_path",
        "template_parse_in_hot_path",
        "json_encoder_recreated_per_item",
        "gzip_reader_writer_recreated_per_item",
        "csv_writer_flush_per_row",
        "json_unmarshal_same_payload_multiple_times",
    ] {
        assert!(has_rule(&report, rule_id), "missing rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_go_advanceplan3_core_hot_path_clean() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "core_clean.go",
        go_fixture!("advanceplan3_core_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "regexp_compile_in_hot_path",
        "template_parse_in_hot_path",
        "json_encoder_recreated_per_item",
        "gzip_reader_writer_recreated_per_item",
        "csv_writer_flush_per_row",
        "json_unmarshal_same_payload_multiple_times",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_go_advanceplan3_data_access_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "data_positive.go",
        go_fixture!("advanceplan3_data_positive.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "sql_open_per_request",
        "gorm_open_per_request",
        "prepare_inside_loop",
        "gorm_debug_enabled_in_request_path",
        "create_single_in_loop_instead_of_batches",
        "gorm_find_without_limit_on_handler_path",
        "offset_pagination_on_large_table",
        "gorm_preload_clause_associations_on_wide_graph",
        "count_then_find_same_filter",
    ] {
        assert!(has_rule(&report, rule_id), "missing rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_go_advanceplan3_data_access_clean() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "data_clean.go",
        go_fixture!("advanceplan3_data_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "sql_open_per_request",
        "gorm_open_per_request",
        "prepare_inside_loop",
        "gorm_debug_enabled_in_request_path",
        "create_single_in_loop_instead_of_batches",
        "gorm_find_without_limit_on_handler_path",
        "offset_pagination_on_large_table",
        "gorm_preload_clause_associations_on_wide_graph",
        "count_then_find_same_filter",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_go_advanceplan3_gin_request_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "gin_positive.go",
        go_fixture!("advanceplan3_gin_positive.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "get_raw_data_then_should_bindjson_duplicate_body",
        "readall_body_then_bind_duplicate_deserialize",
        "multiple_shouldbind_calls_same_handler",
        "indentedjson_in_hot_path",
        "json_marshaled_manually_then_c_data",
    ] {
        assert!(has_rule(&report, rule_id), "missing rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_go_advanceplan3_gin_request_clean() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "gin_clean.go",
        go_fixture!("advanceplan3_gin_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "get_raw_data_then_should_bindjson_duplicate_body",
        "readall_body_then_bind_duplicate_deserialize",
        "multiple_shouldbind_calls_same_handler",
        "indentedjson_in_hot_path",
        "json_marshaled_manually_then_c_data",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}