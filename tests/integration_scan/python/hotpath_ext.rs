use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::super::create_temp_workspace;
use super::write_files;

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

const HOTPATH_EXT_RULES: &[&str] = &[
    "yaml_load_same_payload_multiple_times",
    "xml_parse_same_payload_multiple_times",
    "repeated_datetime_strptime_same_format",
    "repeated_hashlib_new_same_algorithm",
    "repeated_list_index_lookup",
    "append_then_sort_each_iteration",
    "string_join_without_generator",
    "repeated_dict_get_same_key_no_cache",
    "sort_then_first_or_membership_only",
    "repeated_string_format_invariant_template",
    "json_encoder_recreated_per_item",
    "gzip_open_per_chunk",
    "pickle_dumps_in_loop_same_structure",
    "repeated_isinstance_chain_same_object",
    "concatenation_in_comprehension_body",
];

#[test]
fn test_python_hotpath_ext_positive() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/hotpath_ext_code.py",
            python_fixture!("integration/hotpath_ext/ext_positive.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_present(&report, HOTPATH_EXT_RULES);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_hotpath_ext_clean() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/hotpath_ext_code.py",
            python_fixture!("integration/hotpath_ext/ext_clean.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_absent(&report, HOTPATH_EXT_RULES);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
