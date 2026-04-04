use std::fs;
use std::path::{Path, PathBuf};

use deslop::{ScanOptions, ScanReport, scan_repository, scan_repository_with_go_semantic};
use tempfile::TempDir;

pub(crate) struct FixtureWorkspace {
    root: TempDir,
}

impl FixtureWorkspace {
    pub(crate) fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("deslop-test-")
            .tempdir()
            .expect("temp dir creation should succeed");

        Self { root }
    }

    #[allow(dead_code)]
    pub(crate) fn root(&self) -> &Path {
        self.root.path()
    }

    pub(crate) fn write_file(&self, relative_path: &str, contents: &str) {
        write_fixture(self.root.path(), relative_path, contents);
    }

    pub(crate) fn write_files(&self, files: &[(&str, &str)]) {
        write_files(self.root.path(), files);
    }

    pub(crate) fn scan(&self) -> ScanReport {
        self.scan_with_options(true)
    }

    pub(crate) fn scan_report(&self) -> ScanReport {
        self.scan()
    }

    pub(crate) fn scan_with_options(&self, respect_ignore: bool) -> ScanReport {
        scan_root_with_options(self.root.path().to_path_buf(), respect_ignore)
    }

    #[allow(dead_code)]
    pub(crate) fn scan_with_go_semantic(&self, go_semantic: bool) -> ScanReport {
        scan_root_with_go_semantic(self.root.path().to_path_buf(), go_semantic)
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

#[allow(dead_code)]
pub(crate) fn scan_files(files: &[(&str, &str)]) -> ScanReport {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(files);
    workspace.scan_report()
}

#[allow(dead_code)]
pub(crate) fn scan_root(root: PathBuf) -> ScanReport {
    scan_root_with_options(root, true)
}

#[allow(dead_code)]
pub(crate) fn scan_root_with_options(root: PathBuf, respect_ignore: bool) -> ScanReport {
    scan_repository(&ScanOptions {
        root,
        respect_ignore,
    })
    .expect("scan should succeed")
}

#[allow(dead_code)]
pub(crate) fn scan_root_with_go_semantic(root: PathBuf, go_semantic: bool) -> ScanReport {
    scan_repository_with_go_semantic(
        &ScanOptions {
            root,
            respect_ignore: true,
        },
        go_semantic,
    )
    .expect("scan should succeed")
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub(crate) fn assert_rules_present(report: &ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(report_has_rule(report, rule_id), "missing rule: {rule_id}");
    }
}

#[allow(dead_code)]
pub(crate) fn assert_rules_absent(report: &ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            !report_has_rule(report, rule_id),
            "unexpected rule: {rule_id}"
        );
    }
}
