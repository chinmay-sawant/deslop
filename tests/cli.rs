#[path = "support/mod.rs"]
mod support;

#[path = "cli_support/mod.rs"]
mod cli_support;

use cli_support::{assert_json_fields, parse_json_output, run_cli};
use support::FixtureWorkspace;

// ── Scan subcommand tests ──

#[test]
fn cli_scan_exits_zero_for_clean_code() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", "package main\n\nfunc main() {\n}\n");

    let output = run_cli(&["scan", workspace.root().to_str().unwrap()]);

    assert!(
        output.status.success(),
        "clean scan should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_scan_exits_nonzero_for_findings() {
    let workspace = FixtureWorkspace::new();
    // Overlong name + fmt.Sprintf in loop → guaranteed findings.
    workspace.write_file(
        "bad.go",
        "package main\n\nimport \"fmt\"\n\nfunc HandleCalculateAndProcessUserDataFromExternalSource() {\n\tfor i := 0; i < 100; i++ {\n\t\tfmt.Sprintf(\"item %d\", i)\n\t}\n}\n",
    );

    let output = run_cli(&["scan", workspace.root().to_str().unwrap()]);

    assert!(
        !output.status.success(),
        "findings scan should exit non-zero"
    );
}

#[test]
fn cli_scan_no_fail_exits_zero_even_with_findings() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "bad.go",
        "package main\n\nimport \"fmt\"\n\nfunc HandleCalculateAndProcessUserDataFromExternalSource() {\n\tfor i := 0; i < 100; i++ {\n\t\tfmt.Sprintf(\"item %d\", i)\n\t}\n}\n",
    );

    let output = run_cli(&["scan", workspace.root().to_str().unwrap(), "--no-fail"]);

    assert!(
        output.status.success(),
        "--no-fail should exit 0 even with findings: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_scan_json_produces_valid_json() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", "package main\n\nfunc main() {\n}\n");

    let output = run_cli(&[
        "scan",
        workspace.root().to_str().unwrap(),
        "--json",
        "--no-fail",
    ]);

    let parsed = parse_json_output(&output);
    assert_json_fields(&parsed, &["root", "findings", "timings"]);
}

#[test]
fn cli_scan_ignore_flag_suppresses_specific_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "bad.go",
        "package main\n\nimport \"fmt\"\n\nfunc HandleCalculateAndProcessUserDataFromExternalSource() {\n\tfor i := 0; i < 100; i++ {\n\t\tfmt.Sprintf(\"item %d\", i)\n\t}\n}\n",
    );

    let output = run_cli(&[
        "scan",
        workspace.root().to_str().unwrap(),
        "--json",
        "--no-fail",
        "--ignore",
        "overlong_name",
    ]);

    let parsed = parse_json_output(&output);
    let findings = parsed["findings"].as_array().unwrap();
    assert!(
        !findings.iter().any(|f| f["rule_id"] == "overlong_name"),
        "--ignore should suppress the specified rule"
    );
}

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
