use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use goslop::{benchmark_repository, scan_repository, BenchmarkOptions, ScanOptions};

#[test]
fn scans_go_files_and_extracts_fingerprints() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "main.go", include_str!("./fixtures/simple.go"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(report.functions_found, 2);
    assert!(report.parse_failures.is_empty());
    assert_eq!(report.files[0].package_name.as_deref(), Some("sample"));
    assert!(report.findings.is_empty());

    let names = report.files[0]
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["Add", "Run"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn respects_gitignore() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, ".gitignore", "ignored.go\n");
    write_fixture(&temp_dir, "main.go", include_str!("./fixtures/simple.go"));
    write_fixture(&temp_dir, "ignored.go", include_str!("./fixtures/simple.go"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn skips_generated_files_and_keeps_syntax_error_flag() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "generated.go", include_str!("./fixtures/generated.go"));
    write_fixture(&temp_dir, "broken.go", include_str!("./fixtures/malformed.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(report.files[0].path.file_name().and_then(|name| name.to_str()), Some("broken.go"));
    assert!(report.files[0].syntax_error);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_generic_names_and_weak_typing() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "sloppy.go", include_str!("./fixtures/generic_weak.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| finding.rule_id == "generic_name"));
    assert!(report.findings.iter().any(|finding| finding.rule_id == "weak_typing"));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_legitimate_handler_names() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "handler.go", include_str!("./fixtures/legitimate_handler.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "generic_name" && finding.function_name.as_deref() == Some("HandleRequest")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_legitimate_adapter_names() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "adapter.go", include_str!("./fixtures/legitimate_adapter.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "generic_name" && finding.function_name.as_deref() == Some("ConvertValue")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_hallucinated_import_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "main.go", include_str!("./fixtures/hallucinated_import.txt"));
    write_fixture(&temp_dir, "utils/utils.go", include_str!("./fixtures/utils_package.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report
        .findings
        .iter()
        .any(|finding| finding.rule_id == "hallucinated_import_call"));

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

#[test]
fn benchmarks_a_real_scan_path() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "main.go", include_str!("./fixtures/simple.go"));

    let report = benchmark_repository(&BenchmarkOptions {
        root: temp_dir.clone(),
        repeats: 2,
        warmups: 1,
        respect_ignore: true,
    })
    .expect("benchmark should succeed");

    assert_eq!(report.repeats, 2);
    assert_eq!(report.warmups, 1);
    assert_eq!(report.runs.len(), 2);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

fn create_temp_workspace() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("goslop-test-{nonce}"));
    fs::create_dir_all(&dir).expect("temp dir creation should succeed");
    dir
}

fn write_fixture(root: &Path, relative_path: &str, contents: &str) {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir creation should succeed");
    }
    fs::write(path, contents).expect("fixture write should succeed");
}