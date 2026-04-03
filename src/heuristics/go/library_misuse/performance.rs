mod collections;
mod error_api;
mod io_encoding;
mod runtime_sync;
mod strings;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::super::framework_patterns::{
    BodyLine, has_import_path, import_aliases_for, is_identifier_name, is_request_path_function,
    join_lines, split_assignment,
};

use self::collections::slice_map_findings;
use self::error_api::error_interface_findings;
use self::io_encoding::io_encoding_findings;
use self::runtime_sync::runtime_sync_findings;
use self::strings::string_operation_findings;

pub(super) fn performance_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(string_operation_findings(file, function, lines));
    findings.extend(slice_map_findings(file, function, lines));
    findings.extend(runtime_sync_findings(file, function, lines));
    findings.extend(io_encoding_findings(file, function, lines));
    findings.extend(error_interface_findings(file, function, lines));
    findings
}
