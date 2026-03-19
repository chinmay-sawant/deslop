use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use goslop::{BenchmarkOptions, ScanOptions, benchmark_repository, scan_repository};

const GOMINDMAPPER_ROOT: &str = "/home/chinmay/ChinmayPersonalProjects/mindmapper/gomindmapper";
const GOPDFSUIT_ROOT: &str = "/home/chinmay/ChinmayPersonalProjects/gopdfsuit";

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
#[ignore]
fn scans_real_projects_and_prints_reports() {
    let roots = [Path::new(GOMINDMAPPER_ROOT), Path::new(GOPDFSUIT_ROOT)];

    for root in roots {
        let report = scan_repository(&ScanOptions {
            root: root.to_path_buf(),
            respect_ignore: true,
        })
        .unwrap_or_else(|error| panic!("scan should succeed for {}: {error}", root.display()));

        assert!(
            report.files_discovered > 0,
            "{} should contain Go files",
            root.display()
        );
        assert!(
            report.files_analyzed > 0,
            "{} should contain analyzable Go files",
            root.display()
        );

        println!("scan report for {}", root.display());
        println!("  files_discovered: {}", report.files_discovered);
        println!("  files_analyzed: {}", report.files_analyzed);
        println!("  functions_found: {}", report.functions_found);
        println!("  findings: {}", report.findings.len());
        println!("  parse_failures: {}", report.parse_failures.len());
        println!(
            "  index_summary: packages={} symbols={} imports={}",
            report.index_summary.package_count,
            report.index_summary.symbol_count,
            report.index_summary.import_count
        );
        println!(
            "  timings_ms: discover={} parse={} index={} heuristics={} total={}",
            report.timings.discover_ms,
            report.timings.parse_ms,
            report.timings.index_ms,
            report.timings.heuristics_ms,
            report.timings.total_ms
        );
    }
}

#[test]
fn respects_gitignore() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, ".gitignore", "ignored.go\n");
    write_fixture(&temp_dir, "main.go", include_str!("./fixtures/simple.go"));
    write_fixture(
        &temp_dir,
        "ignored.go",
        include_str!("./fixtures/simple.go"),
    );

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
    write_fixture(
        &temp_dir,
        "generated.go",
        include_str!("./fixtures/generated.go"),
    );
    write_fixture(
        &temp_dir,
        "broken.go",
        include_str!("./fixtures/malformed.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(
        report.files[0]
            .path
            .file_name()
            .and_then(|name| name.to_str()),
        Some("broken.go")
    );
    assert!(report.files[0].syntax_error);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_generic_names_and_weak_typing() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "sloppy.go",
        include_str!("./fixtures/generic_weak.txt"),
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
            .any(|finding| finding.rule_id == "generic_name")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "weak_typing")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_error_handling_slop_patterns() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "error_handling.go",
        include_str!("./fixtures/error_handling_slop.txt"),
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
            .any(|finding| finding.rule_id == "dropped_error")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "panic_on_error")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "error_wrapping_misuse")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_wrapped_error_handling_as_misuse() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "error_handling.go",
        include_str!("./fixtures/error_handling_clean.txt"),
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
            .any(|finding| finding.rule_id == "error_wrapping_misuse")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "dropped_error")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "panic_on_error")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_comment_style_and_overlong_names() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "comments.go",
        include_str!("./fixtures/comment_slop.txt"),
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
            .any(|finding| finding.rule_id == "comment_style_title_case")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "comment_style_tutorial")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "overlong_name")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_concise_comments() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "comments.go",
        include_str!("./fixtures/comment_clean.txt"),
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
            .any(|finding| finding.rule_id == "comment_style_title_case")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "comment_style_tutorial")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_weak_crypto_usage() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "crypto.go",
        include_str!("./fixtures/weak_crypto.txt"),
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
            .any(|finding| finding.rule_id == "weak_crypto")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_missing_context_for_http_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "http.go",
        include_str!("./fixtures/missing_context.txt"),
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
            .any(|finding| finding.rule_id == "missing_context")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_context_aware_http_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "http.go",
        include_str!("./fixtures/context_aware_http.txt"),
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
            .any(|finding| finding.rule_id == "missing_context")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_sleep_polling_patterns() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "poll.go",
        include_str!("./fixtures/sleep_polling.txt"),
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
            .any(|finding| finding.rule_id == "sleep_polling")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_sleep_outside_loops() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "poll.go",
        include_str!("./fixtures/sleep_clean.txt"),
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
            .any(|finding| finding.rule_id == "sleep_polling")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_string_concat_in_loops() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "concat.go",
        include_str!("./fixtures/string_concat_loop.txt"),
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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_numeric_plus_equals_in_loops() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "concat.go",
        include_str!("./fixtures/string_concat_clean.txt"),
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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_goroutines_without_coordination() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "go_routine.go",
        include_str!("./fixtures/goroutine_slop.txt"),
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
            .any(|finding| finding.rule_id == "goroutine_without_coordination")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_goroutines_with_waitgroup_coordination() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "go_routine.go",
        include_str!("./fixtures/goroutine_clean.txt"),
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
            .any(|finding| finding.rule_id == "goroutine_without_coordination")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_missing_context_for_exec_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "exec.go",
        include_str!("./fixtures/missing_context_exec.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "missing_context" && finding.message.contains("context-aware work")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_context_aware_exec_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "exec.go",
        include_str!("./fixtures/context_aware_exec.txt"),
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
            .any(|finding| finding.rule_id == "missing_context")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_legitimate_handler_names() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "handler.go",
        include_str!("./fixtures/legitimate_handler.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "generic_name"
            && finding.function_name.as_deref() == Some("HandleRequest")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn does_not_flag_legitimate_adapter_names() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "adapter.go",
        include_str!("./fixtures/legitimate_adapter.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "generic_name"
            && finding.function_name.as_deref() == Some("ConvertValue")
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn flags_hallucinated_import_calls() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "main.go",
        include_str!("./fixtures/hallucinated_import.txt"),
    );
    write_fixture(
        &temp_dir,
        "utils/utils.go",
        include_str!("./fixtures/utils_package.txt"),
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
            .any(|finding| finding.rule_id == "hallucinated_import_call")
    );

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
