use deslop::{ScanReport, Severity};

pub(super) use super::super::support::{
    FixtureWorkspace, assert_rules_absent, assert_rules_present, find_rule, report_has_rule,
    scan_files,
};

pub(super) fn scan_python_files(files: &[(&str, &str)]) -> ScanReport {
    scan_files(files)
}

pub(super) fn scan_generated_files(setup: impl FnOnce(&FixtureWorkspace)) -> ScanReport {
    let workspace = FixtureWorkspace::new();
    setup(&workspace);
    workspace.scan()
}

mod duplication;
mod maintainability;
mod performance;
mod structure;
