//! Integration tests for `go_semantic_experimental` and the explicit CLI toggle.

#[path = "support/mod.rs"]
mod support;

use support::FixtureWorkspace;

const GO_NESTED_LOOP_ALLOC: &str = r#"package main

import "fmt"

func buildIndex(items []string) map[string][]string {
	result := make(map[string][]string)
	for _, item := range items {
		for _, other := range items {
			temp := make([]byte, 0, 128)
			temp = append(temp, item...)
			_ = temp
			if item != other {
				result[item] = append(result[item], other)
			}
		}
	}
	fmt.Println(result)
	return result
}
"#;

#[test]
fn semantic_flag_defaults_to_true_without_config() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", GO_NESTED_LOOP_ALLOC);

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
    workspace.write_file("main.go", GO_NESTED_LOOP_ALLOC);

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
    workspace.write_file("main.go", GO_NESTED_LOOP_ALLOC);

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
