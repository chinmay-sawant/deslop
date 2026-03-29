use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::super::create_temp_workspace;
use super::write_files;

#[test]
fn test_python_fingerprints() {
    let temp_dir = create_temp_workspace();
    write_files(&temp_dir, &[("app.py", python_fixture!("simple.txt"))]);

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(report.functions_found, 2);
    assert!(report.parse_failures.is_empty());
    assert_eq!(report.files[0].package_name.as_deref(), Some("app"));

    let names = report.files[0]
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["build_summary", "render"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_syntax() {
    let temp_dir = create_temp_workspace();
    write_files(&temp_dir, &[("broken.py", python_fixture!("broken.txt"))]);

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert!(report.files[0].syntax_error);
    assert!(report.parse_failures.is_empty());

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_mixed_repo() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            ("app.py", python_fixture!("simple.txt")),
            ("main.go", go_fixture!("simple.go")),
            ("src/main.rs", rust_fixture!("simple.txt")),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 3);
    assert_eq!(report.files_analyzed, 3);
    assert!(report.parse_failures.is_empty());

    let analyzed_paths = report
        .files
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(&temp_dir)
                .expect("report path should stay under the temp dir")
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(analyzed_paths, vec!["app.py", "main.go", "src/main.rs"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_rust_mixed_repo() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            ("pkg/render/service.py", python_fixture!("simple.txt")),
            ("pkg/render/lib.rs", rust_fixture!("simple.txt")),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 2);
    assert!(report.parse_failures.is_empty());

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_same_directory_mixed_repo() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            ("pkg/render/__init__.py", python_fixture!("simple.txt")),
            ("pkg/render/main.go", go_fixture!("simple.go")),
            ("pkg/render/lib.rs", rust_fixture!("simple.txt")),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 3);
    assert_eq!(report.files_analyzed, 3);
    assert!(report.parse_failures.is_empty());

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_rules() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[("service.py", python_fixture!("rule_pack_positive.txt"))],
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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_rule_suppressions() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[("service.py", python_fixture!("rule_pack_negative.txt"))],
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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_test_rule_suppressions() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "tests/test_service.py",
            python_fixture!("rule_pack_test_only.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase4_rules() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[("pkg/__init__.py", python_fixture!("phase4_positive.txt"))],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase4_suppressions() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[("pkg/module.py", python_fixture!("phase4_negative.txt"))],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase4_repo_rules() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/root.py",
                python_fixture!("integration/baseline/phase4_repo_root.txt"),
            ),
            (
                "pkg/base.py",
                python_fixture!("integration/baseline/phase4_repo_base.txt"),
            ),
            (
                "pkg/mid.py",
                python_fixture!("integration/baseline/phase4_repo_mid.txt"),
            ),
            (
                "pkg/helpers.py",
                python_fixture!("integration/baseline/phase4_repo_helpers.txt"),
            ),
            (
                "pkg/models.py",
                python_fixture!("integration/baseline/phase4_repo_models.txt"),
            ),
            (
                "pkg/services.py",
                python_fixture!("integration/baseline/phase4_repo_services.txt"),
            ),
            (
                "pkg/adapters.py",
                python_fixture!("integration/baseline/phase4_repo_adapters.txt"),
            ),
            (
                "pkg/leaf.py",
                python_fixture!("integration/baseline/phase4_repo_leaf.txt"),
            ),
            (
                "tests/test_helpers.py",
                python_fixture!("integration/baseline/phase4_repo_test_helpers.txt"),
            ),
        ],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "deep_inheritance_hierarchy",
        "tight_module_coupling",
        "duplicate_test_utility_logic",
        "cross_file_repeated_literal",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected repo rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_hallucination_rule() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[
            (
                "pkg/target.py",
                python_fixture!("integration/baseline/hallucination_target.txt"),
            ),
            (
                "pkg/caller.py",
                python_fixture!("integration/baseline/hallucination_caller.txt"),
            ),
        ],
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
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("imaginary_function")),
        "expected hallucinated_import_call to fire for imaginary_function"
    );

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("MissingQualifiedClass")),
        "expected hallucinated_import_call to fire for MissingQualifiedClass"
    );

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("MissingImportedClass")),
        "expected hallucinated_import_call to fire for MissingImportedClass"
    );

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_local_call"
                && finding.message.contains("FakeLocalFunction")),
        "expected hallucinated_local_call to fire for FakeLocalFunction"
    );

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("existing_function")),
        "did not expect finding for existing_function"
    );

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("RealImportedClass")),
        "did not expect finding for RealImportedClass"
    );

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_local_call"
                && finding.message.contains("RealLocalFunction")),
        "did not expect finding for RealLocalFunction"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding.message.contains("Path")),
        "did not expect finding for imported stdlib class Path"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding.message.contains("ThirdPartyClient")),
        "did not expect finding for unresolved third-party import alias"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding.message.contains("RuntimeError")),
        "did not expect finding for builtin exception RuntimeError"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding.message.contains("SessionBundle")),
        "did not expect finding for local dataclass SessionBundle"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding
            .message
            .contains("SnapBackTranscriptionClient")),
        "did not expect finding for local class SnapBackTranscriptionClient"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
