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

#[path = "../support/mod.rs"]
mod support;

#[path = "benchmarking.rs"]
mod benchmarking;
#[path = "concurrency.rs"]
mod concurrency;
#[path = "consistency.rs"]
mod consistency;
#[path = "context.rs"]
mod context;
#[path = "core.rs"]
mod core;
#[path = "data_access.rs"]
mod data_access;
#[path = "go_framework_patterns.rs"]
mod go_framework_patterns;
#[path = "go_library_misuse.rs"]
mod go_library_misuse;
#[path = "go_resource_hygiene.rs"]
mod go_resource_hygiene;
#[path = "hallucination.rs"]
mod hallucination;
#[path = "naming.rs"]
mod naming;
#[path = "performance.rs"]
mod performance;
#[path = "python/mod.rs"]
mod python;
#[path = "rust.rs"]
mod rust;
#[path = "rust_advanced.rs"]
mod rust_advanced;
#[path = "rust_api_design_ext.rs"]
mod rust_api_design_ext;
#[path = "security.rs"]
mod security;
#[path = "style.rs"]
mod style;
#[path = "test_quality.rs"]
mod test_quality;

use support::{FixtureWorkspace, assert_rules_absent, assert_rules_present, scan_files};

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
