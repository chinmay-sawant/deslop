use super::{assert_rules_absent, assert_rules_present, report_has_rule, scan_python_files};

#[test]
fn test_python_performance_rule_family_positive() {
    let report = scan_python_files(&[(
        "pkg/loop_shapes.py",
        python_fixture!("performance/loop_shapes_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "string_concat_in_loop",
            "list_materialization_first_element",
            "deque_candidate_queue",
            "temporary_collection_in_loop",
            "recursive_traversal_risk",
            "list_membership_in_loop",
            "repeated_len_in_loop",
        ],
    );
}

#[test]
fn test_python_performance_rule_family_negative() {
    let report = scan_python_files(&[(
        "pkg/loop_shapes.py",
        python_fixture!("performance/loop_shapes_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "string_concat_in_loop",
            "list_materialization_first_element",
            "deque_candidate_queue",
            "temporary_collection_in_loop",
            "recursive_traversal_risk",
            "list_membership_in_loop",
            "repeated_len_in_loop",
        ],
    );
}

#[test]
fn test_python_recursive_traversal_skips_non_self_method_name_collisions() {
    let report = scan_python_files(&[(
        "pkg/argparse_helpers.py",
        python_fixture!("performance/recursive_call_method_name_collision_negative.txt"),
    )]);

    assert!(
        !report_has_rule(&report, "recursive_traversal_risk"),
        "did not expect recursive_traversal_risk for parser.parse_args()"
    );
}

#[test]
fn test_python_async_boundary_rules_positive() {
    let report = scan_python_files(&[(
        "pkg/async_boundaries.py",
        python_fixture!("performance/async_boundaries_positive.txt"),
    )]);

    assert_rules_present(&report, &["blocking_sync_io_in_async", "full_dataset_load"]);
}

#[test]
fn test_python_async_boundary_rules_negative() {
    let report = scan_python_files(&[(
        "pkg/async_boundaries.py",
        python_fixture!("performance/async_boundaries_negative.txt"),
    )]);

    assert_rules_absent(&report, &["blocking_sync_io_in_async", "full_dataset_load"]);
}
