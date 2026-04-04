use std::process::Command;

#[path = "support/mod.rs"]
mod support;

use support::FixtureWorkspace;

fn cargo_bin() -> String {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_deslop") {
        return path;
    }

    let mut path = std::env::current_exe().expect("test binary path should be available");
    path.pop();
    path.pop();
    path.push("deslop");
    path.to_string_lossy().into_owned()
}

// ── Scan subcommand tests ──

#[test]
fn cli_scan_exits_zero_for_clean_code() {
    let bin = cargo_bin();
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", "package main\n\nfunc main() {\n}\n");

    let output = Command::new(&bin)
        .args(["scan", workspace.root().to_str().unwrap()])
        .output()
        .expect("scan should execute");

    assert!(
        output.status.success(),
        "clean scan should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_scan_exits_nonzero_for_findings() {
    let bin = cargo_bin();
    let workspace = FixtureWorkspace::new();
    // Overlong name + fmt.Sprintf in loop → guaranteed findings.
    workspace.write_file(
        "bad.go",
        "package main\n\nimport \"fmt\"\n\nfunc HandleCalculateAndProcessUserDataFromExternalSource() {\n\tfor i := 0; i < 100; i++ {\n\t\tfmt.Sprintf(\"item %d\", i)\n\t}\n}\n",
    );

    let output = Command::new(&bin)
        .args(["scan", workspace.root().to_str().unwrap()])
        .output()
        .expect("scan should execute");

    assert!(
        !output.status.success(),
        "findings scan should exit non-zero"
    );
}

#[test]
fn cli_scan_no_fail_exits_zero_even_with_findings() {
    let bin = cargo_bin();
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "bad.go",
        "package main\n\nimport \"fmt\"\n\nfunc HandleCalculateAndProcessUserDataFromExternalSource() {\n\tfor i := 0; i < 100; i++ {\n\t\tfmt.Sprintf(\"item %d\", i)\n\t}\n}\n",
    );

    let output = Command::new(&bin)
        .args(["scan", workspace.root().to_str().unwrap(), "--no-fail"])
        .output()
        .expect("scan should execute");

    assert!(
        output.status.success(),
        "--no-fail should exit 0 even with findings: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_scan_json_produces_valid_json() {
    let bin = cargo_bin();
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", "package main\n\nfunc main() {\n}\n");

    let output = Command::new(&bin)
        .args([
            "scan",
            workspace.root().to_str().unwrap(),
            "--json",
            "--no-fail",
        ])
        .output()
        .expect("scan should execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("JSON output should be valid JSON");
    assert!(
        parsed.get("root").is_some(),
        "JSON should contain 'root' field"
    );
    assert!(
        parsed.get("findings").is_some(),
        "JSON should contain 'findings' field"
    );
    assert!(
        parsed.get("timings").is_some(),
        "JSON should contain 'timings' field"
    );
}

#[test]
fn cli_scan_ignore_flag_suppresses_specific_rules() {
    let bin = cargo_bin();
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "bad.go",
        "package main\n\nimport \"fmt\"\n\nfunc HandleCalculateAndProcessUserDataFromExternalSource() {\n\tfor i := 0; i < 100; i++ {\n\t\tfmt.Sprintf(\"item %d\", i)\n\t}\n}\n",
    );

    let output = Command::new(&bin)
        .args([
            "scan",
            workspace.root().to_str().unwrap(),
            "--json",
            "--no-fail",
            "--ignore",
            "overlong_name",
        ])
        .output()
        .expect("scan should execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("JSON output should be valid JSON");
    let findings = parsed["findings"].as_array().unwrap();
    assert!(
        !findings.iter().any(|f| f["rule_id"] == "overlong_name"),
        "--ignore should suppress the specified rule"
    );
}

// ── Rules subcommand tests ──

#[test]
fn cli_rules_lists_rules() {
    let bin = cargo_bin();

    let output = Command::new(&bin)
        .args(["rules"])
        .output()
        .expect("rules should execute");

    assert!(output.status.success(), "rules should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dropped_error"), "should list known rules");
}

#[test]
fn cli_rules_json_produces_valid_json() {
    let bin = cargo_bin();

    let output = Command::new(&bin)
        .args(["rules", "--json"])
        .output()
        .expect("rules should execute");

    assert!(output.status.success(), "rules --json should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("JSON output should be valid JSON");
    assert!(parsed.is_array(), "JSON rules output should be an array");
}

#[test]
fn cli_rules_language_filter() {
    let bin = cargo_bin();

    let output = Command::new(&bin)
        .args(["rules", "--json", "--language", "rust"])
        .output()
        .expect("rules should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let rules = parsed.as_array().unwrap();
    assert!(
        rules
            .iter()
            .all(|r| r["language"] == "rust" || r["language"] == "common"),
        "language filter should only return rust and common rules"
    );
}
