use super::FixtureWorkspace;

#[test]
fn test_hallucination() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", go_fixture!("hallucinated_import.txt"));
    workspace.write_file("utils/utils.go", go_fixture!("utils_package.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call")
    );
}

#[test]
fn test_hallucination_dir() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", go_fixture!("hallucination/dir_main.txt"));
    workspace.write_file(
        "pkg/render/render.go",
        go_fixture!("hallucination/dir_pkg_render.txt"),
    );
    workspace.write_file(
        "internal/render/render.go",
        go_fixture!("hallucination/dir_internal_render.txt"),
    );

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("Run")
            && finding.message.contains("render.Sanitize")
    }));
}

#[test]
fn test_alias_hallucination() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "pdf/generator.go",
        go_fixture!("hallucination/alias_positive.txt"),
    );

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_local_call"
            && finding.function_name.as_deref() == Some("collectAllStandardFontsInTemplate")
            && finding.start_line == 9
    }));
}

#[test]
fn test_rust_go_separation() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "main.go",
        go_fixture!("hallucination/rust_go_separation_main.txt"),
    );
    workspace.write_file(
        "pkg/render/render.go",
        go_fixture!("hallucination/rust_go_separation_render.txt"),
    );
    workspace.write_file(
        "pkg/render/lib.rs",
        rust_fixture!("integration/rust_go_separation_lib.txt"),
    );

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("Run")
            && finding.message.contains("render.Normalize")
    }));
}
