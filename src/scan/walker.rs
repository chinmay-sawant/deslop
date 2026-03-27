use std::path::{Component, Path, PathBuf};

use ignore::WalkBuilder;

use crate::{Error, Result};

pub fn discover_source_files(
    root: &Path,
    respect_ignore: bool,
    supported_extensions: &[&str],
) -> Result<Vec<PathBuf>> {
    let mut builder = WalkBuilder::new(root);
    builder.require_git(false);

    if !respect_ignore {
        builder
            .hidden(false)
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false);
    }

    let mut files = Vec::new();

    for entry in builder.build() {
        let entry = entry.map_err(|error| Error::walk(root, error))?;
        let path = entry.path();

        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }

        if !supported_extensions.contains(
            &path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default(),
        ) {
            continue;
        }

        if is_excluded_path(path) {
            continue;
        }

        files.push(path.to_path_buf());
    }

    files.sort();
    Ok(files)
}

fn is_excluded_path(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(name) if name == "vendor"))
}
