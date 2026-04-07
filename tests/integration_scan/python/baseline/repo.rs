use super::FixtureWorkspace;

#[test]
fn test_python_phase4_repo_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
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
    ]);

    let report = workspace.scan();

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
}

#[test]
fn test_python_hallucination_rule() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "pkg/target.py",
            python_fixture!("hallucination/import_resolution_target.txt"),
        ),
        (
            "pkg/caller.py",
            python_fixture!("hallucination/import_resolution_positive.txt"),
        ),
    ]);

    let report = workspace.scan();

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
}

#[test]
fn test_python_tight_module_coupling_skips_package_entrypoints() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "pkg/__init__.py",
            python_fixture!("integration/baseline/package_entrypoint_init_clean.txt"),
        ),
        (
            "pkg/helpers.py",
            python_fixture!("integration/baseline/package_entrypoint_helpers.txt"),
        ),
        (
            "pkg/models.py",
            python_fixture!("integration/baseline/package_entrypoint_models.txt"),
        ),
        (
            "pkg/services.py",
            python_fixture!("integration/baseline/package_entrypoint_services.txt"),
        ),
        (
            "pkg/adapters.py",
            python_fixture!("integration/baseline/package_entrypoint_adapters.txt"),
        ),
        (
            "pkg/leaf.py",
            python_fixture!("integration/baseline/package_entrypoint_leaf.txt"),
        ),
    ]);

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "tight_module_coupling"),
        "did not expect tight_module_coupling for package entrypoint re-exports"
    );
}
