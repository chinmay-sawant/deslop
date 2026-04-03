//! Integration tests for `go_semantic_experimental` and the explicit CLI toggle.

use deslop::{ScanOptions, scan_repository_with_go_semantic};

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
fn semantic_flag_propagates_through_config() {
    let workspace = FixtureWorkspace::new();
    write_fixture(
        &workspace,
        ".deslop.toml",
        "go_semantic_experimental = true\n",
    );
    write_fixture(&workspace, "main.go", "package main\n\nfunc main() {}\n");

    let report = scan_repository_with_go_semantic(
        &ScanOptions {
            root: workspace.root().to_path_buf(),
            respect_ignore: true,
        },
        false,
    )
    .expect("scan should succeed");

    assert!(report.files_analyzed >= 1);
}

#[test]
fn semantic_gated_rules_enabled_via_toggle() {
    let workspace = FixtureWorkspace::new();
    write_fixture(
        &workspace,
        ".deslop.toml",
        "go_semantic_experimental = false\n",
    );
    write_fixture(&workspace, "main.go", GO_NESTED_LOOP_ALLOC);

    let report = scan_repository_with_go_semantic(
        &ScanOptions {
            root: workspace.root().to_path_buf(),
            respect_ignore: true,
        },
        true,
    )
    .expect("scan should succeed");

    let has_semantic_rule = report
        .findings
        .iter()
        .any(|f| f.rule_id == "likely_n_squared_allocation");

    assert!(
        has_semantic_rule,
        "semantic-gated rule should fire when config enables it"
    );
}

#[test]
fn semantic_gated_rules_disabled_without_toggle() {
    let workspace = FixtureWorkspace::new();
    write_fixture(
        &workspace,
        ".deslop.toml",
        "go_semantic_experimental = false\n",
    );
    write_fixture(&workspace, "main.go", GO_NESTED_LOOP_ALLOC);

    let report = scan_repository_with_go_semantic(
        &ScanOptions {
            root: workspace.root().to_path_buf(),
            respect_ignore: true,
        },
        false,
    )
    .expect("scan should succeed");

    let has_semantic_rule = report
        .findings
        .iter()
        .any(|f| f.rule_id == "likely_n_squared_allocation");

    assert!(
        !has_semantic_rule,
        "semantic-gated rule should NOT fire without the flag"
    );
}

fn write_fixture(workspace: &FixtureWorkspace, relative_path: &str, contents: &str) {
    workspace.write_file(relative_path, contents);
}
