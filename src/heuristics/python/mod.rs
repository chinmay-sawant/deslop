mod maintainability;
mod performance;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::Finding;

use self::maintainability::{eval_exec_findings, print_debugging_findings};
use self::performance::{
    blocking_sync_io_findings, full_dataset_load_findings, string_concat_findings,
};

pub(crate) fn python_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(string_concat_findings(file, function));
    findings.extend(blocking_sync_io_findings(file, function));
    findings.extend(full_dataset_load_findings(file, function));
    findings.extend(eval_exec_findings(file, function));
    findings.extend(print_debugging_findings(file, function));
    findings
}
