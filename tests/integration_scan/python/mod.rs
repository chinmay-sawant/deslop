use std::path::Path;

pub(super) mod baseline;
pub(super) mod phase5_rules;

pub(super) fn write_files(root: &Path, files: &[(&str, &str)]) {
    for (relative_path, contents) in files {
        super::write_fixture(root, relative_path, contents);
    }
}
