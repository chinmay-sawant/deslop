use super::{FixtureWorkspace, assert_rules_absent, assert_rules_present};

#[test]
fn test_concat_loops() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("concat.go", go_fixture!("string_concat_loop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
}

#[test]
fn test_numeric_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("concat.go", go_fixture!("string_concat_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
}

#[test]
fn test_json_loops() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("json.go", go_fixture!("json_marshal_loop.txt"));

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "repeated_json_marshaling"
            && finding.function_name.as_deref() == Some("EncodeAll")
    }));
}

#[test]
fn test_json_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("json.go", go_fixture!("json_marshal_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "repeated_json_marshaling")
    );
}

#[test]
fn test_hot_path() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("hot_path.go", go_fixture!("hot_path_slop.txt"));

    let report = workspace.scan();

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
}

#[test]
fn test_hot_path_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("hot_path.go", go_fixture!("hot_path_clean.txt"));

    let report = workspace.scan();

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
}

#[test]
fn test_dataset_load() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("full_dataset.go", go_fixture!("full_dataset_load_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
}

#[test]
fn test_streaming_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "full_dataset.go",
        go_fixture!("full_dataset_load_clean.txt"),
    );

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
}

#[test]
fn test_semantic_n_squared_rules_are_disabled_by_default() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("alloc.go", go_fixture!("n_squared_alloc_slop.txt"));
    workspace.write_file("concat.go", go_fixture!("n_squared_concat_slop.txt"));

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.rule_id.as_str(),
            "likely_n_squared_allocation" | "likely_n_squared_string_concat"
        )
    }));
}

#[test]
fn test_semantic_n_squared_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = true\n");
    workspace.write_file("alloc.go", go_fixture!("n_squared_alloc_slop.txt"));
    workspace.write_file("concat.go", go_fixture!("n_squared_concat_slop.txt"));

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "likely_n_squared_allocation"
            && finding.function_name.as_deref() == Some("Expand")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "likely_n_squared_string_concat"
            && finding.function_name.as_deref() == Some("Render")
    }));
}

#[test]
fn test_semantic_n_squared_clean_fixtures() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = true\n");
    workspace.write_file("alloc.go", go_fixture!("n_squared_alloc_clean.txt"));
    workspace.write_file("concat.go", go_fixture!("n_squared_concat_clean.txt"));

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.rule_id.as_str(),
            "likely_n_squared_allocation" | "likely_n_squared_string_concat"
        )
    }));
}

#[test]
fn test_semantic_nested_query_escalation() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = true\n");
    workspace.write_file("query.go", go_fixture!("n_squared_query_slop.txt"));

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "n_plus_one_query"
            && matches!(finding.severity, deslop::Severity::Error)
            && finding.function_name.as_deref() == Some("Load")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "go_perf_layer_database_access_query_inside_loop_without_batching"
            && finding.function_name.as_deref() == Some("Load")
    }));
}

#[test]
fn test_semantic_nested_query_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = true\n");
    workspace.write_file("query.go", go_fixture!("n_squared_query_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "n_plus_one_query")
    );
}

const EXTRA_GO_PERFORMANCE_RULES: &[&str] = &[
    "bytes_compare_equal_zero",
    "bytes_compare_not_equal_zero",
    "bytes_index_not_minus_one_contains",
    "bytes_index_any_not_minus_one_contains_any",
    "bytes_count_gt_zero_contains",
    "strings_compare_equal_zero",
    "strings_compare_not_equal_zero",
    "strings_index_any_not_minus_one_contains_any",
    "strings_count_gt_zero_contains",
    "strings_splitn_two_index_zero_cut",
    "strings_splitn_two_index_one_cut",
    "bytes_splitn_two_index_zero_cut",
    "bytes_splitn_two_index_one_cut",
    "strings_split_two_index_zero_cut",
    "strings_split_two_index_one_cut",
    "bytes_split_two_index_zero_cut",
    "bytes_split_two_index_one_cut",
    "strings_splitaftern_two_index_zero_cut",
    "strings_splitaftern_two_index_one_cut",
    "bytes_splitaftern_two_index_zero_cut",
    "bytes_splitaftern_two_index_one_cut",
    "strings_splitafter_two_index_zero_cut",
    "strings_splitafter_two_index_one_cut",
    "bytes_splitafter_two_index_zero_cut",
    "bytes_splitafter_two_index_one_cut",
    "strings_tolower_equalfold",
    "strings_toupper_equalfold",
    "bytes_tolower_equalfold",
    "bytes_toupper_equalfold",
    "writer_write_byte_slice_of_string",
    "builder_write_string_single_byte_literal",
    "buffer_write_string_single_byte_literal",
    "builder_write_rune_ascii_literal",
    "buffer_write_rune_ascii_literal",
    "bytes_newreader_on_string_conversion",
    "strings_newreader_on_byte_slice_conversion",
    "bytes_newbufferstring_on_string_conversion",
    "strings_replace_neg_one_replaceall",
    "bytes_replace_neg_one_replaceall",
    "strings_trimleft_space_trimspace",
    "strings_trimright_space_trimspace",
    "bytes_trimleft_space_trimspace",
    "bytes_trimright_space_trimspace",
    "bytes_buffer_string_len",
    "strings_builder_string_len",
    "bytes_buffer_truncate_zero_reset",
    "filepath_split_base_only",
    "filepath_split_dir_only",
    "path_split_base_only",
    "path_split_dir_only",
    "strings_hasprefix_manual_slice_after_len",
    "strings_hassuffix_manual_slice_after_len",
    "bytes_hasprefix_manual_slice_after_len",
    "bytes_hassuffix_manual_slice_after_len",
    "fmt_sprintf_bool_to_string",
    "fmt_sprintf_float_to_string",
    "fmt_sprintf_binary_to_string",
    "fmt_sprintf_octal_to_string",
    "fmt_sprintf_hex_to_string",
    "fmt_sprintf_quote_to_string",
    "fmt_sprintf_single_string_passthrough",
    "fmt_fprint_to_strings_builder",
    "fmt_fprintln_to_strings_builder",
    "fmt_fprintf_single_string_to_strings_builder",
    "fmt_fprint_to_bytes_buffer",
    "fmt_fprintln_to_bytes_buffer",
    "fmt_fprintf_single_string_to_bytes_buffer",
    "strconv_formatint_int64_cast_itoa",
    "time_tick_per_call",
    "time_newtimer_inside_loop",
    "time_newticker_inside_loop",
    "rand_seed_per_call",
    "rand_newsource_per_call",
    "rand_newsource_with_time_now_per_call",
    "rand_new_per_call",
    "time_since_candidate_via_now_sub",
    "time_until_candidate_via_deadline_sub_now",
    "duration_nanoseconds_zero_check",
    "runtime_numcpu_inside_loop",
    "runtime_gomaxprocs_per_request",
    "time_loadlocation_per_call",
    "time_fixedzone_per_call",
    "sync_once_do_inside_loop",
    "context_withtimeout_inside_loop",
    "json_valid_then_unmarshal",
    "json_marshalindent_in_loop",
    "json_indent_in_loop",
    "base64_encode_to_string_in_loop",
    "base64_decode_string_in_loop",
    "hex_encode_to_string_in_loop",
    "hex_decode_string_in_loop",
    "sha1_sum_in_loop",
    "sha256_sum_in_loop",
    "sha512_sum_in_loop",
    "md5_sum_in_loop",
    "crc32_checksum_in_loop",
    "crc64_checksum_in_loop",
    "adler32_checksum_in_loop",
    "hmac_new_in_loop",
    "strings_newreplacer_per_call",
];

#[test]
fn test_extra_go_performance_pack_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "extra_positive.go",
        go_fixture!("performance_extra_positive.txt"),
    );

    let report = workspace.scan();
    assert_rules_present(&report, EXTRA_GO_PERFORMANCE_RULES);
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id
            == "go_perf_layer_async_concurrency_context_timeout_allocated_per_inner_call"
            && finding.function_name.as_deref() == Some("ContextTimeoutLoop")
    }));
}

#[test]
fn test_extra_go_performance_pack_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("extra_clean.go", go_fixture!("performance_extra_clean.txt"));

    let report = workspace.scan();
    assert_rules_absent(&report, EXTRA_GO_PERFORMANCE_RULES);
    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id
            == "go_perf_layer_async_concurrency_context_timeout_allocated_per_inner_call"
    }));
}
