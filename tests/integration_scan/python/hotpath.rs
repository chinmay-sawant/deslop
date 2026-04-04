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

const HOTPATH_RULES: &[&str] = &[
    "regex_compile_in_hot_path",
    "json_loads_same_payload_multiple_times",
    "repeated_json_dumps_same_object",
    "sorted_only_for_first_element",
    "list_comprehension_only_for_length",
    "readlines_then_iterate",
    "read_then_splitlines",
    "in_check_on_list_literal",
    "string_startswith_endswith_chain",
    "enumerate_on_range_len",
    "csv_writer_flush_per_row",
    "write_without_buffering_in_loop",
    "repeated_open_same_file_in_function",
    "dict_items_or_keys_materialized_in_loop",
];

#[test]
fn test_python_hotpath_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[(
        "pkg/hotpath_code.py",
        python_fixture!("integration/hotpath/core_positive.txt"),
    )]);

    let report = workspace.scan();

    assert_rules_present(&report, HOTPATH_RULES);
}

#[test]
fn test_python_hotpath_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[(
        "pkg/hotpath_code.py",
        python_fixture!("integration/hotpath/core_clean.txt"),
    )]);

    let report = workspace.scan();

    assert_rules_absent(&report, HOTPATH_RULES);
}
