//! Integration tests for `go_semantic_experimental` and the explicit CLI toggle.

#[path = "support/mod.rs"]
mod support;

use support::FixtureWorkspace;

#[test]
fn semantic_flag_defaults_to_true_without_config() {
    let workspace = FixtureWorkspace::new();
    let fixture = support::load_fixture("go/n_squared_alloc_slop.txt");
    workspace.write_file("main.go", &fixture);

    let report = workspace.scan_with_go_semantic(false);

    assert!(report.files_analyzed >= 1);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.rule_id == "likely_n_squared_allocation")
    );
}

#[test]
fn semantic_gated_rules_enabled_via_toggle() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = false\n");
    let fixture = support::load_fixture("go/n_squared_alloc_slop.txt");
    workspace.write_file("main.go", &fixture);

    let report = workspace.scan_with_go_semantic(true);

    let has_semantic_rule = report
        .findings
        .iter()
        .any(|f| f.rule_id == "likely_n_squared_allocation");

    assert!(
        has_semantic_rule,
        "semantic-gated rule should fire when the CLI toggle enables it"
    );
}

#[test]
fn semantic_gated_rules_disabled_without_toggle() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = false\n");
    let fixture = support::load_fixture("go/n_squared_alloc_slop.txt");
    workspace.write_file("main.go", &fixture);

    let report = workspace.scan_with_go_semantic(false);

    let has_semantic_rule = report
        .findings
        .iter()
        .any(|f| f.rule_id == "likely_n_squared_allocation");

    assert!(
        !has_semantic_rule,
        "semantic-gated rule should NOT fire without the flag"
    );
}
