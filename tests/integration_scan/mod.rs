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

// `support` lives one directory above; #[path] is required here because
// the module file is outside the integration_scan/ subtree.
#[path = "../support/mod.rs"]
mod support;

mod benchmarking;
mod concurrency;
mod consistency;
mod context;
mod core;
mod data_access;
mod go_framework_patterns;
mod go_library_misuse;
mod go_resource_hygiene;
mod hallucination;
mod naming;
mod performance;
mod python;
mod rust;
mod rust_advanced;
mod rust_api_design_ext;
mod rust_module_surface;
mod security;
mod style;
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
