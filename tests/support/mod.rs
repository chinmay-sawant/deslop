#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use deslop::{ScanOptions, ScanReport, scan_repository};

pub(crate) struct FixtureWorkspace {
    root: PathBuf,
}

impl FixtureWorkspace {
    pub(crate) fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("deslop-test-{nonce}"));
        fs::create_dir_all(&root).expect("temp dir creation should succeed");

        Self { root }
    }

    #[allow(dead_code)]
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    #[allow(dead_code)]
    pub(crate) fn write_file(&self, relative_path: &str, contents: &str) {
        write_fixture(&self.root, relative_path, contents);
    }

    pub(crate) fn write_files(&self, files: &[(&str, &str)]) {
        write_files(&self.root, files);
    }

    pub(crate) fn scan(&self) -> ScanReport {
        scan_repository(&ScanOptions {
            root: self.root.clone(),
            respect_ignore: true,
        })
        .expect("scan should succeed")
    }
}

impl Drop for FixtureWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub(crate) fn write_fixture(root: &Path, relative_path: &str, contents: &str) {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent dir creation should succeed");
    }
    fs::write(path, contents).expect("fixture write should succeed");
}

pub(crate) fn write_files(root: &Path, files: &[(&str, &str)]) {
    for (relative_path, contents) in files {
        write_fixture(root, relative_path, contents);
    }
}

pub(crate) fn scan_files(files: &[(&str, &str)]) -> ScanReport {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(files);
    workspace.scan()
}

pub(crate) fn report_has_rule(report: &ScanReport, rule_id: &str) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.rule_id == rule_id)
}

#[allow(dead_code)]
pub(crate) fn find_rule<'a>(report: &'a ScanReport, rule_id: &str) -> Option<&'a deslop::Finding> {
    report
        .findings
        .iter()
        .find(|finding| finding.rule_id == rule_id)
}

pub(crate) fn assert_rules_present(report: &ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(report_has_rule(report, rule_id), "missing rule: {rule_id}");
    }
}

pub(crate) fn assert_rules_absent(report: &ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            !report_has_rule(report, rule_id),
            "unexpected rule: {rule_id}"
        );
    }
}
