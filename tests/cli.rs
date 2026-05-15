#[path = "support/mod.rs"]
mod support;

#[path = "cli_support/mod.rs"]
mod cli_support;

use cli_support::{parse_json_output, run_cli};

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
