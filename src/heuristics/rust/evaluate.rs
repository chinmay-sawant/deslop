use crate::analysis::{Language, ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

#[cfg(test)]
pub(crate) use crate::analysis::rust::findings::{
    alias_lookup, call_matches_import, import_matches_item,
};

use crate::analysis::rust::findings::{
    doc_marker_findings, non_test_call_findings, non_test_macro_findings, rust_call_findings,
    rust_import_findings, unsafe_findings,
};

use super::{
    api_design_file_findings, api_design_function_findings, async_file_findings,
    async_function_findings, boundary_file_findings, boundary_function_findings, domain_findings,
    module_surface_file_findings, performance_file_findings, performance_function_findings,
    runtime_file_findings, runtime_function_findings, runtime_ownership_function_findings,
    security_footguns_file_findings, security_footguns_function_findings,
    unsafe_soundness_findings, bad_practices,
};

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

pub(crate) fn evaluate_rust_file_hygiene_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let mut findings = Vec::new();

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
    findings.extend(doc_marker_findings(file, function));
    findings
}

pub(crate) fn rust_api_design_file_findings(file: &ParsedFile) -> Vec<Finding> {
    api_design_file_findings(file)
}

pub(crate) fn rust_api_design_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    api_design_function_findings(file, function)
}

pub(crate) fn rust_async_file_findings(file: &ParsedFile) -> Vec<Finding> {
    async_file_findings(file)
}

pub(crate) fn rust_async_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    async_function_findings(file, function)
}

pub(crate) fn rust_boundary_file_findings(file: &ParsedFile) -> Vec<Finding> {
    boundary_file_findings(file)
}

pub(crate) fn rust_boundary_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    boundary_function_findings(file, function)
}

pub(crate) fn rust_domain_file_findings(file: &ParsedFile) -> Vec<Finding> {
    domain_findings(file)
}

pub(crate) fn rust_module_surface_file_findings(file: &ParsedFile) -> Vec<Finding> {
    module_surface_file_findings(file)
}

pub(crate) fn rust_performance_file_findings(file: &ParsedFile) -> Vec<Finding> {
    performance_file_findings(file)
}

pub(crate) fn rust_performance_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    performance_function_findings(file, function)
}

pub(crate) fn rust_runtime_file_findings(
    file: &ParsedFile,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    runtime_file_findings(file, index)
}

pub(crate) fn rust_runtime_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    runtime_function_findings(file, function)
}

pub(crate) fn rust_runtime_ownership_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    runtime_ownership_function_findings(file, function)
}

pub(crate) fn rust_security_file_findings(
    file: &ParsedFile,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    security_footguns_file_findings(file, index)
}

pub(crate) fn rust_security_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    security_footguns_function_findings(file, function)
}

pub(crate) fn rust_unsafe_soundness_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    unsafe_soundness_findings(file, function)
}

pub(crate) fn rust_bad_practices_file_findings(file: &ParsedFile) -> Vec<Finding> {
    bad_practices::bad_practices_file_findings(file)
}

pub(crate) fn rust_bad_practices_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    bad_practices::bad_practices_function_findings(file, function)
}

pub(crate) fn rust_bad_practices_indexed_file_findings(
    file: &ParsedFile,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    bad_practices::bad_practices_indexed_file_findings(file, index)
}

pub(crate) fn rust_import_resolution_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    let Some(package_name) = &file.package_name else {
        return Vec::new();
    };
    let Some(current_package) = index.package_for_file(Language::Rust, &file.path, package_name)
    else {
        return Vec::new();
    };

    rust_import_findings(file, function, index, &file.imports, current_package)
}

pub(crate) fn rust_local_call_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    rust_call_findings(file, function, index, &file.imports)
}
