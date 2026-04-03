//! Integration tests for `go_semantic_experimental` / `DESLOP_ENABLE_GO_SEMANTIC` gating.
//!
//! These tests must run sequentially (they mutate process env vars).
//! Use `cargo test --test semantic_flag -- --test-threads=1` to run in isolation.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use deslop::{ScanOptions, scan_repository};

fn temp_dir(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("deslop-semantic-{name}-{nonce}"));
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

/// Verify basic config propagation works without panics.
#[test]
fn semantic_flag_propagates_through_config() {
    let root = temp_dir("propagation");
    write_fixture(
        &root,
        ".deslop.toml",
        "go_semantic_experimental = true\n",
    );
    write_fixture(&root, "main.go", "package main\n\nfunc main() {}\n");

    let report = scan_repository(&ScanOptions {
        root: root.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(report.files_analyzed >= 1);
    fs::remove_dir_all(root).expect("temp dir cleanup should succeed");
}

/// Verify that when `go_semantic_experimental = true` in config,
/// nested loop allocation rules fire.
#[test]
fn semantic_gated_rules_enabled_via_config() {
    let root = temp_dir("semantic-alloc");
    write_fixture(&root, ".deslop.toml", "go_semantic_experimental = true\n");
    write_fixture(&root, "main.go", GO_NESTED_LOOP_ALLOC);

    let report = scan_repository(&ScanOptions {
        root: root.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    let has_semantic_rule = report
        .findings
        .iter()
        .any(|f| f.rule_id == "likely_n_squared_allocation");

    assert!(
        has_semantic_rule,
        "semantic-gated rule should fire when config enables it"
    );

    fs::remove_dir_all(root).expect("temp dir cleanup should succeed");
}

/// Verify that when go_semantic_experimental is explicitly false in config
/// and env var is not set, the semantic rule does NOT fire.
#[test]
fn semantic_gated_rules_disabled_via_config() {
    let root = temp_dir("no-semantic-alloc");
    write_fixture(
        &root,
        ".deslop.toml",
        "go_semantic_experimental = false\n",
    );
    write_fixture(&root, "main.go", GO_NESTED_LOOP_ALLOC);

    // The test is robust regardless of env var state because explicit false in
    // config takes precedence. But we also clear the env var to be thorough.
    let had_env = std::env::var("DESLOP_ENABLE_GO_SEMANTIC").ok();
    unsafe { std::env::remove_var("DESLOP_ENABLE_GO_SEMANTIC"); }

    let report = scan_repository(&ScanOptions {
        root: root.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    // Restore env var if it was previously set.
    if let Some(val) = had_env {
        unsafe { std::env::set_var("DESLOP_ENABLE_GO_SEMANTIC", val); }
    }

    let has_semantic_rule = report
        .findings
        .iter()
        .any(|f| f.rule_id == "likely_n_squared_allocation");

    assert!(
        !has_semantic_rule,
        "semantic-gated rule should NOT fire without the flag"
    );

    fs::remove_dir_all(root).expect("temp dir cleanup should succeed");
}
