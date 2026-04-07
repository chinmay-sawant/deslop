use super::FixtureWorkspace;

#[test]
fn test_python_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[("service.py", python_fixture!("rule_pack_positive.txt"))]);

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_sync_io_in_async")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "exception_swallowed")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "eval_exec_usage")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "print_debugging_leftover")
    );
}

#[test]
fn test_python_rule_suppressions() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[("service.py", python_fixture!("rule_pack_negative.txt"))]);

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_sync_io_in_async")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "exception_swallowed")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "eval_exec_usage")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "print_debugging_leftover")
    );
}

#[test]
fn test_python_test_rule_suppressions() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[(
        "tests/test_service.py",
        python_fixture!("rule_pack_test_only.txt"),
    )]);

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.rule_id.as_str(),
            "string_concat_in_loop"
                | "blocking_sync_io_in_async"
                | "full_dataset_load"
                | "exception_swallowed"
                | "eval_exec_usage"
                | "print_debugging_leftover"
        )
    }));
}

#[test]
fn test_python_phase4_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[("pkg/__init__.py", python_fixture!("phase4_positive.txt"))]);

    let report = workspace.scan();

    for rule_id in [
        "blocking_sync_io_in_async",
        "none_comparison",
        "side_effect_comprehension",
        "redundant_return_none",
        "hardcoded_path_string",
        "variadic_public_api",
        "temporary_collection_in_loop",
        "recursive_traversal_risk",
        "list_membership_in_loop",
        "repeated_len_in_loop",
        "builtin_reduction_candidate",
        "broad_exception_handler",
        "missing_context_manager",
        "public_api_missing_type_hints",
        "mixed_sync_async_module",
        "textbook_docstring_small_helper",
        "mixed_naming_conventions",
        "god_function",
        "god_class",
        "monolithic_init_module",
        "too_many_instance_attributes",
        "eager_constructor_collaborators",
        "over_abstracted_wrapper",
        "list_materialization_first_element",
        "deque_candidate_queue",
        "mixed_concerns_function",
        "name_responsibility_mismatch",
        "unrelated_heavy_import",
        "obvious_commentary",
        "enthusiastic_commentary",
        "commented_out_code",
        "repeated_string_literal",
        "duplicate_error_handler_block",
        "duplicate_validation_pipeline",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected rule {rule_id} to fire"
        );
    }
}

#[test]
fn test_python_phase4_suppressions() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[("pkg/module.py", python_fixture!("phase4_negative.txt"))]);

    let report = workspace.scan();

    for rule_id in [
        "blocking_sync_io_in_async",
        "none_comparison",
        "side_effect_comprehension",
        "redundant_return_none",
        "hardcoded_path_string",
        "variadic_public_api",
        "temporary_collection_in_loop",
        "recursive_traversal_risk",
        "list_membership_in_loop",
        "repeated_len_in_loop",
        "builtin_reduction_candidate",
        "broad_exception_handler",
        "missing_context_manager",
        "public_api_missing_type_hints",
        "mixed_sync_async_module",
        "textbook_docstring_small_helper",
        "mixed_naming_conventions",
        "god_function",
        "god_class",
        "monolithic_init_module",
        "too_many_instance_attributes",
        "eager_constructor_collaborators",
        "over_abstracted_wrapper",
        "list_materialization_first_element",
        "deque_candidate_queue",
        "mixed_concerns_function",
        "name_responsibility_mismatch",
        "unrelated_heavy_import",
        "obvious_commentary",
        "enthusiastic_commentary",
        "commented_out_code",
        "repeated_string_literal",
        "duplicate_error_handler_block",
        "duplicate_validation_pipeline",
    ] {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }
}
