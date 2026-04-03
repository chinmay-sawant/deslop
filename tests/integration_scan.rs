macro_rules! go_fixture {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/go/",
            $path
        ))
    };
}

macro_rules! rust_fixture {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/rust/",
            $path
        ))
    };
}

macro_rules! python_fixture {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/python/",
            $path
        ))
    };
}

#[path = "integration_scan/benchmarking.rs"]
mod benchmarking;
#[path = "integration_scan/concurrency.rs"]
mod concurrency;
#[path = "integration_scan/consistency.rs"]
mod consistency;
#[path = "integration_scan/context.rs"]
mod context;
#[path = "integration_scan/core.rs"]
mod core;
#[path = "integration_scan/data_access.rs"]
mod data_access;
#[path = "integration_scan/go_framework_patterns.rs"]
mod go_framework_patterns;
#[path = "integration_scan/go_library_misuse.rs"]
mod go_library_misuse;
#[path = "integration_scan/go_resource_hygiene.rs"]
mod go_resource_hygiene;
#[path = "integration_scan/hallucination.rs"]
mod hallucination;
#[path = "integration_scan/naming.rs"]
mod naming;
#[path = "integration_scan/performance.rs"]
mod performance;
#[path = "integration_scan/python/mod.rs"]
mod python;
#[path = "integration_scan/rust.rs"]
mod rust;
#[path = "integration_scan/rust_advanced.rs"]
mod rust_advanced;
#[path = "integration_scan/rust_api_design_ext.rs"]
mod rust_api_design_ext;
#[path = "integration_scan/security.rs"]
mod security;
#[path = "integration_scan/style.rs"]
mod style;
#[path = "support/mod.rs"]
mod support;
#[path = "integration_scan/test_quality.rs"]
mod test_quality;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use support::{assert_rules_absent, assert_rules_present, scan_files};

#[test]
fn test_error_slop() {
    let report = scan_files(&[("error_handling.go", go_fixture!("error_handling_slop.txt"))]);

    assert_rules_present(
        &report,
        &["dropped_error", "panic_on_error", "error_wrapping_misuse"],
    );
}

#[test]
fn test_error_ok() {
    let report = scan_files(&[("error_handling.go", go_fixture!("error_handling_clean.txt"))]);

    assert_rules_absent(
        &report,
        &["error_wrapping_misuse", "dropped_error", "panic_on_error"],
    );
}

fn create_temp_workspace() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("deslop-test-{nonce}"));
    fs::create_dir_all(&dir).expect("temp dir creation should succeed");
    dir
}

fn write_fixture(root: &Path, relative_path: &str, contents: &str) {
    support::write_fixture(root, relative_path, contents);
}
