#[path = "support/mod.rs"]
mod support;

#[path = "cli_support/mod.rs"]
mod cli_support;

use cli_support::{parse_json_output, run_cli};

const EXPORT_HANDLE_FMT_POSITIVE: &str =
    "go/rules_fixtures/export_context_handle_fmt/export_context_handle_fmt_positive.txt";
const EXPORT_CLEAN_HANDLE_NEGATIVE: &str =
    "go/rules_fixtures/export_context_clean_handle/export_context_clean_handle_negative.txt";

// ── Rules subcommand tests ──

#[test]
fn cli_rules_lists_rules() {
    let output = run_cli(&["rules"]);

    assert!(output.status.success(), "rules should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dropped_error"), "should list known rules");
}

#[test]
fn cli_rules_json_produces_valid_json() {
    let output = run_cli(&["rules", "--json"]);

    assert!(output.status.success(), "rules --json should exit 0");
    let parsed = parse_json_output(&output);
    assert!(parsed.is_array(), "JSON rules output should be an array");
}

#[test]
fn cli_rules_language_filter() {
    let output = run_cli(&["rules", "--json", "--language", "rust"]);

    assert!(output.status.success());
    let parsed = parse_json_output(&output);
    let rules = parsed.as_array().unwrap();
    assert!(
        rules
            .iter()
            .all(|r| r["language"] == "rust" || r["language"] == "common"),
        "language filter should only return rust and common rules"
    );
}

#[test]
fn cli_scan_exports_context_and_chunks_by_default() {
    let workspace = support::FixtureWorkspace::new();
    workspace.write_fixture_file(EXPORT_HANDLE_FMT_POSITIVE, "main.go");

    let context_dir = workspace.root().join("context");
    let chunks_dir = workspace.root().join("chunks");
    let output = run_cli(&[
        "scan",
        workspace.root().to_str().unwrap(),
        "--no-fail",
        "--context-output-dir",
        context_dir.to_str().unwrap(),
        "--chunks-output-dir",
        chunks_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "scan should succeed: {:?}", output);
    assert!(context_dir.join("1.txt").is_file());
    let chunk_entries: Vec<_> = std::fs::read_dir(&chunks_dir)
        .expect("chunks dir should exist")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("Chunk_"))
        .collect();
    assert!(
        !chunk_entries.is_empty(),
        "expected at least one chunk file"
    );
}

#[test]
fn cli_scan_no_context_and_no_chunks_skip_export() {
    let workspace = support::FixtureWorkspace::new();
    workspace.write_fixture_file(EXPORT_CLEAN_HANDLE_NEGATIVE, "main.go");

    let context_dir = workspace.root().join("context");
    let chunks_dir = workspace.root().join("chunks");
    let output = run_cli(&[
        "scan",
        workspace.root().to_str().unwrap(),
        "--no-fail",
        "--no-context",
        "--no-chunks",
        "--context-output-dir",
        context_dir.to_str().unwrap(),
        "--chunks-output-dir",
        chunks_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "scan should succeed: {:?}", output);
    assert!(!context_dir.exists());
    assert!(!chunks_dir.exists());
}
