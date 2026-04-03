use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn cargo_bin() -> String {
    let output = Command::new("cargo")
        .args(["build", "--quiet"])
        .output()
        .expect("cargo build should succeed");
    assert!(output.status.success(), "cargo build failed");
    // Return the path to the binary
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| "target".to_string());
    format!("{}/debug/deslop", target_dir)
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("deslop-cli-{name}-{nonce}"));
    fs::create_dir_all(&path).expect("temp dir creation should succeed");
    path
}

fn write_fixture(root: &Path, relative_path: &str, contents: &str) {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir creation should succeed");
    }
    fs::write(path, contents).expect("fixture write should succeed");
}

// ── Scan subcommand tests ──

#[test]
fn cli_scan_exits_zero_for_clean_code() {
    let bin = cargo_bin();
    let root = temp_dir("clean");
    write_fixture(&root, "main.go", "package main\n\nfunc main() {\n}\n");

    let output = Command::new(&bin)
        .args(["scan", root.to_str().unwrap()])
        .output()
        .expect("scan should execute");

    assert!(
        output.status.success(),
        "clean scan should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(root).expect("temp dir cleanup should succeed");
}

#[test]
fn cli_scan_exits_nonzero_for_findings() {
    let bin = cargo_bin();
    let root = temp_dir("findings");
    // Overlong name + fmt.Sprintf in loop → guaranteed findings.
    write_fixture(
        &root,
        "bad.go",
        "package main\n\nimport \"fmt\"\n\nfunc HandleCalculateAndProcessUserDataFromExternalSource() {\n\tfor i := 0; i < 100; i++ {\n\t\tfmt.Sprintf(\"item %d\", i)\n\t}\n}\n",
    );

    let output = Command::new(&bin)
        .args(["scan", root.to_str().unwrap()])
        .output()
        .expect("scan should execute");

    assert!(!output.status.success(), "findings scan should exit non-zero");

    fs::remove_dir_all(root).expect("temp dir cleanup should succeed");
}

#[test]
fn cli_scan_no_fail_exits_zero_even_with_findings() {
    let bin = cargo_bin();
    let root = temp_dir("nofail");
    write_fixture(
        &root,
        "bad.go",
        "package main\n\nimport \"fmt\"\n\nfunc HandleCalculateAndProcessUserDataFromExternalSource() {\n\tfor i := 0; i < 100; i++ {\n\t\tfmt.Sprintf(\"item %d\", i)\n\t}\n}\n",
    );

    let output = Command::new(&bin)
        .args(["scan", root.to_str().unwrap(), "--no-fail"])
        .output()
        .expect("scan should execute");

    assert!(
        output.status.success(),
        "--no-fail should exit 0 even with findings: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(root).expect("temp dir cleanup should succeed");
}

#[test]
fn cli_scan_json_produces_valid_json() {
    let bin = cargo_bin();
    let root = temp_dir("json");
    write_fixture(&root, "main.go", "package main\n\nfunc main() {\n}\n");

    let output = Command::new(&bin)
        .args(["scan", root.to_str().unwrap(), "--json", "--no-fail"])
        .output()
        .expect("scan should execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("JSON output should be valid JSON");
    assert!(parsed.get("root").is_some(), "JSON should contain 'root' field");
    assert!(parsed.get("findings").is_some(), "JSON should contain 'findings' field");
    assert!(
        parsed.get("timings").is_some(),
        "JSON should contain 'timings' field"
    );

    fs::remove_dir_all(root).expect("temp dir cleanup should succeed");
}

#[test]
fn cli_scan_ignore_flag_suppresses_specific_rules() {
    let bin = cargo_bin();
    let root = temp_dir("ignore");
    write_fixture(
        &root,
        "bad.go",
        "package main\n\nimport \"fmt\"\n\nfunc HandleCalculateAndProcessUserDataFromExternalSource() {\n\tfor i := 0; i < 100; i++ {\n\t\tfmt.Sprintf(\"item %d\", i)\n\t}\n}\n",
    );

    let output = Command::new(&bin)
        .args([
            "scan",
            root.to_str().unwrap(),
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

    fs::remove_dir_all(root).expect("temp dir cleanup should succeed");
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
