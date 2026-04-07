use super::{FixtureWorkspace, assert_rule_severity, assert_rules_absent, assert_rules_present};

#[test]
fn test_rust_fingerprints() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/main.rs", rust_fixture!("simple.txt"));

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(report.functions_found, 2);
    assert!(report.parse_failures.is_empty());
    assert_eq!(report.files[0].package_name.as_deref(), Some("main"));

    let names = report.files[0]
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["sum_pair", "render_summary"]);
}

#[test]
fn test_rust_syntax() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("broken.txt"));

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert!(report.files[0].syntax_error);
    assert!(report.parse_failures.is_empty());
}

#[test]
fn test_mixed_repo() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", go_fixture!("simple.go"));
    workspace.write_file("src/main.rs", rust_fixture!("simple.txt"));

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 2);
    assert!(report.parse_failures.is_empty());

    let analyzed_paths = report
        .files
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(workspace.root())
                .expect("report path should stay under the temp dir")
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(analyzed_paths, vec!["main.go", "src/main.rs"]);
}

#[test]
fn test_rust_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("rule_pack_positive.txt"));

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "todo_macro_leftover",
            "unimplemented_macro_leftover",
            "dbg_macro_leftover",
            "panic_macro_leftover",
            "unreachable_macro_leftover",
            "todo_doc_comment_leftover",
            "fixme_doc_comment_leftover",
            "hack_doc_comment_leftover",
            "unwrap_in_non_test_code",
            "expect_in_non_test_code",
            "unsafe_without_safety_comment",
        ],
    );
}

#[test]
fn test_rust_suppressions() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("src/lib.rs", rust_fixture!("rule_pack_negative.txt"));

    let report = workspace.scan();

    assert_rules_absent(
        &report,
        &[
            "todo_macro_leftover",
            "unimplemented_macro_leftover",
            "dbg_macro_leftover",
            "panic_macro_leftover",
            "unreachable_macro_leftover",
            "todo_doc_comment_leftover",
            "fixme_doc_comment_leftover",
            "hack_doc_comment_leftover",
            "unwrap_in_non_test_code",
            "expect_in_non_test_code",
            "unsafe_without_safety_comment",
        ],
    );
}

#[test]
fn test_rust_rule_ignore_directives() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("integration/rule_ignore_directives.txt"),
    );

    let report = workspace.scan();

    assert_rules_absent(
        &report,
        &["unwrap_in_non_test_code", "panic_macro_leftover"],
    );
    assert_rules_present(&report, &["expect_in_non_test_code"]);
}

#[test]
fn test_rust_repository_config_controls_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml",
        "rust_async_experimental = false\n[severity_overrides]\nexpect_in_non_test_code = \"error\"\n",
    );
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("integration/repository_config_controls_rules.txt"),
    );

    let report = workspace.scan();

    assert_rules_absent(&report, &["rust_async_std_mutex_await"]);
    assert_rule_severity(&report, "expect_in_non_test_code", deslop::Severity::Error);
}

#[test]
fn test_rust_hallucination() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("hallucinated_import_positive_main.txt"),
    );
    workspace.write_file(
        "src/config/render.rs",
        rust_fixture!("hallucinated_import_positive_render.txt"),
    );

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("run_render")
            && finding.message.contains("render::missing_formatter")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("run_missing_module")
            && finding.message.contains("helpers::load")
    }));
}

#[test]
fn test_rust_hierarchy() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/config/mod.rs",
        rust_fixture!("hallucinated_import_negative_mod.txt"),
    );
    workspace.write_file(
        "src/config/render.rs",
        rust_fixture!("hallucinated_import_negative_render.txt"),
    );
    workspace.write_file(
        "src/config/sub/helpers.rs",
        rust_fixture!("hallucinated_import_negative_helpers.txt"),
    );

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && matches!(
                finding.function_name.as_deref(),
                Some("run_render") | Some("run_helper")
            )
    }));
}

#[test]
fn test_direct_hallucination() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("direct_call_hallucination_positive.txt"),
    );
    workspace.write_file(
        "src/config/render.rs",
        rust_fixture!("direct_call_hallucination_render.txt"),
    );

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("run_direct_import")
            && finding.message.contains("render_missing")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_local_call"
            && finding.function_name.as_deref() == Some("run_same_module")
            && finding.message.contains("missing_local")
    }));
}

#[test]
fn test_rust_direct_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "src/lib.rs",
        rust_fixture!("direct_call_hallucination_negative.txt"),
    );
    workspace.write_file(
        "src/config/render.rs",
        rust_fixture!("direct_call_hallucination_render.txt"),
    );

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.function_name.as_deref(),
            Some("run_direct_import")
                | Some("run_same_module")
                | Some("run_local_closure")
                | Some("run_constructor")
        ) && matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        )
    }));
}
