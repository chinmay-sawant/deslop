use super::support::{assert_rules_absent, assert_rules_present, scan_files};

fn source_for(path: &str) -> &'static str {
    match path {
        "src/analysis/rust/evaluate.rs" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/analysis/rust/evaluate.rs"
        )),
        "src/analysis/rust/parser/functions.rs" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/analysis/rust/parser/functions.rs"
        )),
        "src/heuristics/rust/api_design.rs" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/heuristics/rust/api_design.rs"
        )),
        "src/benchmark/mod.rs" => {
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/benchmark/mod.rs"))
        }
        "src/cli/mod.rs" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/mod.rs")),
        "src/scan/mod.rs" => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scan/mod.rs")),
        "src/heuristics/rust/mod.rs" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/heuristics/rust/mod.rs"
        )),
        _ => panic!("unexpected fixture path: {path}"),
    }
}

#[test]
fn path_attributes_outside_mod_rs_are_not_flagged() {
    for path in [
        "src/analysis/rust/evaluate.rs",
        "src/analysis/rust/parser/functions.rs",
        "src/heuristics/rust/api_design.rs",
    ] {
        let report = scan_files(&[(path, source_for(path))]);
        assert_rules_absent(&report, &["rust_redundant_path_attribute"]);
    }
}

#[test]
fn thin_facade_modules_do_not_trip_the_oversized_module_rule() {
    for path in ["src/benchmark/mod.rs", "src/cli/mod.rs", "src/scan/mod.rs"] {
        let report = scan_files(&[(path, source_for(path))]);
        assert_rules_absent(&report, &["rust_oversized_module_file"]);
    }
}

#[test]
fn large_rust_modules_still_trip_the_oversized_module_rule() {
    let report = scan_files(&[(
        "src/heuristics/rust/mod.rs",
        source_for("src/heuristics/rust/mod.rs"),
    )]);
    assert_rules_present(&report, &["rust_oversized_module_file"]);
}
