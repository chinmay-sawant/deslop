use std::path::{Path, PathBuf};

use toml::Value;

use crate::io::{DEFAULT_MAX_BYTES, read_to_string_limited};

pub(super) fn read_root_manifest(root: &Path) -> Option<(PathBuf, Value)> {
    let manifest_path = root.join("Cargo.toml");
    let source = read_to_string_limited(&manifest_path, DEFAULT_MAX_BYTES).ok()?;
    let parsed = source.parse::<Value>().ok()?;

    Some((manifest_path, parsed))
}
