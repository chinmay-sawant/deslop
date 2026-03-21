use std::fs;

use deslop::{scan_repository, ScanOptions};

use super::{create_temp_workspace, write_fixture};

#[test]
fn flags_hallucinated_import_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "main.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/hallucinated_import.txt")),
    );
    write_fixture(
        &temp_dir,
        "utils/utils.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/generic/utils_package.txt")),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "hallucinated_import_call"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_hallucinated_import_calls_using_import_directory_not_package_name_only() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "main.go",
        r#"package sample

import render "github.com/acme/project/pkg/render"

func Run(address string) string {
    return render.Sanitize(address)
}
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/render/render.go",
        r#"package render

func Normalize(address string) string {
    return address
}
"#,
    );
    write_fixture(
        &temp_dir,
        "internal/render/render.go",
        r#"package render

func Sanitize(address string) string {
    return address
}
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_import_call"
            && finding.function_name.as_deref() == Some("Run")
            && finding.message.contains("render.Sanitize")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_package_level_function_alias_vars_as_hallucinated() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
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
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "hallucinated_local_call"
            && finding.function_name.as_deref() == Some("collectAllStandardFontsInTemplate")
            && finding.start_line == 9
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
