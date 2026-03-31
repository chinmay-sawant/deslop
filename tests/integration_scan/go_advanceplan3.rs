use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::{create_temp_workspace, write_fixture};

fn has_rule(report: &deslop::ScanReport, rule_id: &str) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.rule_id == rule_id)
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
        "builder_or_buffer_recreated_per_iteration",
        "make_slice_inside_hot_loop_same_shape",
        "make_map_inside_hot_loop_same_shape",
        "repeated_slice_clone_in_loop",
        "byte_string_conversion_in_loop",
        "slice_membership_in_loop_map_candidate",
        "url_parse_in_loop_on_invariant_base",
        "time_parse_layout_in_loop",
        "strings_split_same_input_multiple_times",
        "bytes_split_same_input_multiple_times",
        "strconv_repeat_on_same_binding",
        "xml_unmarshal_same_payload_multiple_times",
        "yaml_unmarshal_same_payload_multiple_times",
        "proto_unmarshal_same_payload_multiple_times",
        "json_encoder_recreated_per_item",
        "json_decoder_recreated_per_item",
        "gzip_reader_writer_recreated_per_item",
        "csv_writer_flush_per_row",
        "read_then_decode_duplicate_materialization",
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
        "builder_or_buffer_recreated_per_iteration",
        "make_slice_inside_hot_loop_same_shape",
        "make_map_inside_hot_loop_same_shape",
        "repeated_slice_clone_in_loop",
        "byte_string_conversion_in_loop",
        "slice_membership_in_loop_map_candidate",
        "url_parse_in_loop_on_invariant_base",
        "time_parse_layout_in_loop",
        "strings_split_same_input_multiple_times",
        "bytes_split_same_input_multiple_times",
        "strconv_repeat_on_same_binding",
        "xml_unmarshal_same_payload_multiple_times",
        "yaml_unmarshal_same_payload_multiple_times",
        "proto_unmarshal_same_payload_multiple_times",
        "json_encoder_recreated_per_item",
        "json_decoder_recreated_per_item",
        "gzip_reader_writer_recreated_per_item",
        "csv_writer_flush_per_row",
        "read_then_decode_duplicate_materialization",
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
        "db_ping_per_request",
        "connection_pool_reconfigured_per_request",
        "prepare_inside_loop",
        "prepare_on_every_request_same_sql",
        "tx_begin_per_item_loop",
        "exec_inside_loop_without_batch",
        "queryrow_inside_loop_existence_check",
        "count_inside_loop",
        "gorm_session_allocated_per_item",
        "preload_inside_loop",
        "raw_scan_inside_loop",
        "association_find_inside_loop",
        "first_or_create_in_loop",
        "save_in_loop_full_model",
        "update_single_row_in_loop_without_batch",
        "delete_single_row_in_loop_without_batch",
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
        "db_ping_per_request",
        "connection_pool_reconfigured_per_request",
        "prepare_inside_loop",
        "prepare_on_every_request_same_sql",
        "tx_begin_per_item_loop",
        "exec_inside_loop_without_batch",
        "queryrow_inside_loop_existence_check",
        "count_inside_loop",
        "gorm_session_allocated_per_item",
        "preload_inside_loop",
        "raw_scan_inside_loop",
        "association_find_inside_loop",
        "first_or_create_in_loop",
        "save_in_loop_full_model",
        "update_single_row_in_loop_without_batch",
        "delete_single_row_in_loop_without_batch",
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
        "bindjson_into_map_any_hot_endpoint",
        "bindquery_into_map_any_hot_endpoint",
        "parsemultipartform_large_default_memory",
        "formfile_open_readall_whole_upload",
        "multiple_shouldbind_calls_same_handler",
        "shouldbindbodywith_when_single_bind_is_enough",
        "indentedjson_in_hot_path",
        "repeated_c_json_inside_stream_loop",
        "json_marshaled_manually_then_c_data",
        "servefile_via_readfile_then_c_data",
        "dumprequest_or_dumpresponse_in_hot_path",
        "file_or_template_read_per_request",
        "gin_context_copy_for_each_item_fanout",
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
        "bindjson_into_map_any_hot_endpoint",
        "bindquery_into_map_any_hot_endpoint",
        "parsemultipartform_large_default_memory",
        "formfile_open_readall_whole_upload",
        "multiple_shouldbind_calls_same_handler",
        "shouldbindbodywith_when_single_bind_is_enough",
        "indentedjson_in_hot_path",
        "repeated_c_json_inside_stream_loop",
        "json_marshaled_manually_then_c_data",
        "servefile_via_readfile_then_c_data",
        "dumprequest_or_dumpresponse_in_hot_path",
        "file_or_template_read_per_request",
        "gin_context_copy_for_each_item_fanout",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
