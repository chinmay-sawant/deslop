use deslop::{ScanOptions, scan_repository};

use super::FixtureWorkspace;

#[test]
fn test_hallucination() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", go_fixture!("hallucinated_import.txt"));
    workspace.write_file("utils/utils.go", go_fixture!("utils_package.txt"));

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

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
    workspace.write_file(
        "main.go",
        r#"package sample

import render "github.com/acme/project/pkg/render"

func Run(address string) string {
    return render.Sanitize(address)
}
"#,
    );
    workspace.write_file(
        "pkg/render/render.go",
        r#"package render

func Normalize(address string) string {
    return address
}
"#,
    );
    workspace.write_file(
        "internal/render/render.go",
        r#"package render

func Sanitize(address string) string {
    return address
}
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

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
        r#"package pdf

import font "example.com/font"

var (
    IsCustomFont = font.IsCustomFont
)

func collectAllStandardFontsInTemplate() {
    IsCustomFont("Helvetica")
}
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

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
        r#"package sample

import render "github.com/acme/project/pkg/render"

func Run(address string) string {
    return render.Normalize(address)
}
"#,
    );
    workspace.write_file(
        "pkg/render/render.go",
        r#"package render

func Sanitize(address string) string {
    return address
}
"#,
    );
    workspace.write_file(
        "pkg/render/lib.rs",
        r#"pub fn Normalize(address: &str) -> String {
    address.to_string()
}
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("Run")
            && finding.message.contains("render.Normalize")
    }));
}
