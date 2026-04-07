#[path = "findings/mod.rs"]
pub(crate) mod findings;

#[cfg(test)]
pub(crate) use self::findings::{alias_lookup, call_matches_import, import_matches_item};
use self::findings::{
    doc_marker_findings, non_test_call_findings, non_test_macro_findings, rust_call_findings,
    rust_import_findings, unsafe_findings,
};

use crate::analysis::{Language, ParsedFile};
use crate::heuristics::rust::{
    api_design_file_findings, api_design_function_findings, async_file_findings,
    async_function_findings, boundary_file_findings, boundary_function_findings, domain_findings,
    module_surface_file_findings, performance_file_findings, performance_function_findings,
    runtime_file_findings, runtime_function_findings, runtime_ownership_function_findings,
    security_footguns_file_findings, security_footguns_function_findings,
    unsafe_soundness_findings,
};
use crate::heuristics::engine::{extend_file_rules, extend_function_rules};
use crate::index::RepositoryIndex;
use crate::model::Finding;

pub(crate) const BINDING_LOCATION: &str = file!();

const NON_TEST_MACRO_RULES: [(&str, &str, &str); 5] = [
    (
        "todo!",
        "todo_macro_leftover",
        "leaves todo! in non-test Rust code",
    ),
    (
        "unimplemented!",
        "unimplemented_macro_leftover",
        "leaves unimplemented! in non-test Rust code",
    ),
    (
        "dbg!",
        "dbg_macro_leftover",
        "leaves dbg! in non-test Rust code",
    ),
    (
        "panic!",
        "panic_macro_leftover",
        "leaves panic! in non-test Rust code",
    ),
    (
        "unreachable!",
        "unreachable_macro_leftover",
        "leaves unreachable! in non-test Rust code",
    ),
];

const NON_TEST_CALL_RULES: [(&str, &str, &str); 2] = [
    (
        "unwrap",
        "unwrap_in_non_test_code",
        "calls unwrap() in non-test Rust code",
    ),
    (
        "expect",
        "expect_in_non_test_code",
        "calls expect() in non-test Rust code",
    ),
];

type RustFileRule = fn(&ParsedFile) -> Vec<Finding>;
type RustFunctionRule = fn(&ParsedFile, &crate::analysis::ParsedFunction) -> Vec<Finding>;

const FILE_RULES: [RustFileRule; 4] = [
    domain_findings,
    api_design_file_findings,
    performance_file_findings,
    async_file_findings,
];

const FUNCTION_RULES: [RustFunctionRule; 6] = [
    unsafe_soundness_findings,
    api_design_function_findings,
    doc_marker_findings,
    performance_function_findings,
    async_function_findings,
    runtime_function_findings,
];

pub(super) fn evaluate_rust_findings(file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    extend_file_rules(&mut findings, file, &FILE_RULES);
    findings.extend(runtime_file_findings(file, index));
    findings.extend(boundary_file_findings(file));
    findings.extend(module_surface_file_findings(file));
    findings.extend(security_footguns_file_findings(file, index));

    for function in &file.functions {
        for (macro_name, rule_id, message_suffix) in NON_TEST_MACRO_RULES {
            findings.extend(non_test_macro_findings(
                file,
                function,
                macro_name,
                rule_id,
                message_suffix,
            ));
        }

        for (call_name, rule_id, message_suffix) in NON_TEST_CALL_RULES {
            findings.extend(non_test_call_findings(
                file,
                function,
                call_name,
                rule_id,
                message_suffix,
            ));
        }

        findings.extend(unsafe_findings(file, function));

        extend_function_rules(&mut findings, file, function, &FUNCTION_RULES);
        findings.extend(boundary_function_findings(file, function));
        findings.extend(runtime_ownership_function_findings(file, function));
        findings.extend(security_footguns_function_findings(file, function));
        let Some(package_name) = &file.package_name else {
            continue;
        };
        let Some(current_package) =
            index.package_for_file(Language::Rust, &file.path, package_name)
        else {
            continue;
        };
        findings.extend(rust_import_findings(
            file,
            function,
            index,
            &file.imports,
            current_package,
        ));
        findings.extend(rust_call_findings(file, function, index, &file.imports));
    }

    findings
}
