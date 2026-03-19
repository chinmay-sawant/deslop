use std::path::{Component, Path, PathBuf};

use anyhow::Result;
use ignore::WalkBuilder;

pub fn discover_go_files(root: &Path, respect_ignore: bool) -> Result<Vec<PathBuf>> {
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
        let entry = entry?;
        let path = entry.path();

        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("go") {
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
