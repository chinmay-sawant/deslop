use std::path::Path;

pub(super) mod advanceplan2;
pub(super) mod baseline;
pub(super) mod framework;
pub(super) mod hotpath;
pub(super) mod hotpath_ext;
pub(super) mod mlops;
pub(super) mod packaging;
pub(super) mod phase5_rules;

pub(super) fn write_files(root: &Path, files: &[(&str, &str)]) {
    crate::support::write_files(root, files);
}
