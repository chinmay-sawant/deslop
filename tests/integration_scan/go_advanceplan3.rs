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
        "slice_append_without_prealloc_known_bound",
        "nested_append_without_outer_capacity",
        "map_growth_without_size_hint",
        "strings_builder_without_grow_known_bound",
        "bytes_buffer_without_grow_known_bound",
        "repeated_map_clone_in_loop",
        "append_then_trim_each_iteration",
        "stable_value_normalization_in_inner_loop",
        "bufio_writer_missing_in_bulk_export",
        "bufio_reader_missing_for_small_read_loop",
        "nested_linear_join_map_candidate",
        "append_then_sort_each_iteration",
        "sort_before_first_or_membership_only",
        "filter_then_count_then_iterate",
        "uuid_hash_formatting_only_for_logs",
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
        "slice_append_without_prealloc_known_bound",
        "nested_append_without_outer_capacity",
        "map_growth_without_size_hint",
        "strings_builder_without_grow_known_bound",
        "bytes_buffer_without_grow_known_bound",
        "repeated_map_clone_in_loop",
        "append_then_trim_each_iteration",
        "stable_value_normalization_in_inner_loop",
        "bufio_writer_missing_in_bulk_export",
        "bufio_reader_missing_for_small_read_loop",
        "nested_linear_join_map_candidate",
        "append_then_sort_each_iteration",
        "sort_before_first_or_membership_only",
        "filter_then_count_then_iterate",
        "uuid_hash_formatting_only_for_logs",
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
        "nested_transaction_in_request_path",
        "select_or_get_inside_loop_lookup",
        "row_by_row_upsert_loop",
        "repeated_same_query_template_same_function",
        "exists_via_count_star",
        "find_all_then_manual_paginate_in_go",
        "duplicate_find_then_preload_followup",
        "gorm_select_missing_projection_on_wide_model",
        "gorm_joins_plus_preload_plus_find_without_limit",
        "order_by_without_limit_orm_chain",
        "order_by_random_request_path",
        "distinct_wide_row_request_path",
        "lower_or_func_wrapped_indexed_column",
        "date_or_cast_wrapped_indexed_column",
        "leading_wildcard_builder_chain",
        "scan_into_map_string_any_hot_path",
        "automigrate_or_schema_probe_in_request_path",
        "save_for_single_column_change",
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
        "nested_transaction_in_request_path",
        "row_by_row_upsert_loop",
        "repeated_same_query_template_same_function",
        "exists_via_count_star",
        "find_all_then_manual_paginate_in_go",
        "duplicate_find_then_preload_followup",
        "gorm_joins_plus_preload_plus_find_without_limit",
        "order_by_without_limit_orm_chain",
        "order_by_random_request_path",
        "distinct_wide_row_request_path",
        "lower_or_func_wrapped_indexed_column",
        "date_or_cast_wrapped_indexed_column",
        "leading_wildcard_builder_chain",
        "automigrate_or_schema_probe_in_request_path",
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
        "template_parse_in_handler",
        "loadhtmlglob_or_loadhtmlfiles_in_request_path",
        "middleware_allocates_http_client_per_request",
        "middleware_allocates_db_or_gorm_handle_per_request",
        "middleware_allocates_regex_or_template_per_request",
        "env_or_config_lookup_per_request",
        "upstream_http_call_per_item_in_handler_loop",
        "duplicate_upstream_calls_same_url_same_handler",
        "errgroup_fanout_without_limit_in_handler",
        "large_csv_or_json_export_without_bufio",
        "gzip_or_zip_writer_created_per_chunk",
        "repeated_body_rewind_for_multiple_decoders",
        "middleware_rebinds_body_after_handler_bind",
        "no_streaming_for_large_export_handler",
        "large_h_payload_built_only_for_json_response",
        "repeated_large_map_literal_response_construction",
        "gin_logger_debug_body_logging_on_hot_routes",
        "upstream_json_decode_same_response_multiple_times",
        "no_batching_on_handler_driven_db_write_loop",
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
        "template_parse_in_handler",
        "loadhtmlglob_or_loadhtmlfiles_in_request_path",
        "middleware_allocates_http_client_per_request",
        "middleware_allocates_db_or_gorm_handle_per_request",
        "middleware_allocates_regex_or_template_per_request",
        "env_or_config_lookup_per_request",
        "upstream_http_call_per_item_in_handler_loop",
        "duplicate_upstream_calls_same_url_same_handler",
        "errgroup_fanout_without_limit_in_handler",
        "large_csv_or_json_export_without_bufio",
        "gzip_or_zip_writer_created_per_chunk",
        "repeated_body_rewind_for_multiple_decoders",
        "middleware_rebinds_body_after_handler_bind",
        "no_streaming_for_large_export_handler",
        "large_h_payload_built_only_for_json_response",
        "repeated_large_map_literal_response_construction",
        "gin_logger_debug_body_logging_on_hot_routes",
        "upstream_json_decode_same_response_multiple_times",
        "no_batching_on_handler_driven_db_write_loop",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_go_advanceplan3_request_path_framework_expansion_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "request_paths_positive.go",
        go_fixture!("advanceplan3_request_paths_positive.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "middleware_allocates_http_client_per_request",
        "middleware_allocates_db_or_gorm_handle_per_request",
        "middleware_allocates_regex_or_template_per_request",
        "env_or_config_lookup_per_request",
        "template_parse_in_handler",
        "file_or_template_read_per_request",
        "upstream_http_call_per_item_in_handler_loop",
        "duplicate_upstream_calls_same_url_same_handler",
        "errgroup_fanout_without_limit_in_handler",
        "large_csv_or_json_export_without_bufio",
        "gzip_or_zip_writer_created_per_chunk",
        "no_streaming_for_large_export_handler",
        "repeated_large_map_literal_response_construction",
        "upstream_json_decode_same_response_multiple_times",
        "no_batching_on_handler_driven_db_write_loop",
    ] {
        assert!(has_rule(&report, rule_id), "missing rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_go_advanceplan3_request_path_framework_expansion_clean() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "request_paths_clean.go",
        go_fixture!("advanceplan3_request_paths_clean.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "middleware_allocates_http_client_per_request",
        "middleware_allocates_db_or_gorm_handle_per_request",
        "middleware_allocates_regex_or_template_per_request",
        "env_or_config_lookup_per_request",
        "template_parse_in_handler",
        "file_or_template_read_per_request",
        "upstream_http_call_per_item_in_handler_loop",
        "duplicate_upstream_calls_same_url_same_handler",
        "errgroup_fanout_without_limit_in_handler",
        "large_csv_or_json_export_without_bufio",
        "gzip_or_zip_writer_created_per_chunk",
        "no_streaming_for_large_export_handler",
        "repeated_large_map_literal_response_construction",
        "upstream_json_decode_same_response_multiple_times",
        "no_batching_on_handler_driven_db_write_loop",
    ] {
        assert!(!has_rule(&report, rule_id), "unexpected rule: {rule_id}");
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
