use std::path::{Component, Path, PathBuf};

use ignore::WalkBuilder;

use crate::io::canonicalize_within_root;
use crate::{Error, Result};

pub fn discover_source_files(
    root: &Path,
    respect_ignore: bool,
    supported_extensions: &[&str],
) -> Result<Vec<PathBuf>> {
    let canonical_root = root.canonicalize().map_err(|error| Error::io(root, error))?;
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

        match canonicalize_within_root(&canonical_root, path) {
            Ok(canonical_path) => files.push(canonical_path),
            Err(Error::SymlinkRejected { .. } | Error::PathOutsideRoot { .. }) => continue,
            Err(error) => return Err(error),
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn is_excluded_path(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(name) if name == "vendor"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::discover_source_files;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("deslop-walker-{name}-{nonce}"));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    #[test]
    fn discovers_real_rust_files() {
        let root = temp_dir("files");
        let src = root.join("src");
        fs::create_dir_all(&src).expect("src dir should be created");
        fs::write(src.join("lib.rs"), "fn demo() {}\n").expect("fixture should be written");

        let files = discover_source_files(&root, true, &["rs"]).expect("walk should succeed");
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("src/lib.rs"));

        fs::remove_dir_all(root).expect("temp dir should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn skips_symlinked_files_that_escape_root() {
        use std::os::unix::fs::symlink;

        let root = temp_dir("symlink-root");
        let outside = temp_dir("symlink-outside");
        let src = root.join("src");
        fs::create_dir_all(&src).expect("src dir should be created");
        fs::write(src.join("lib.rs"), "fn local() {}\n").expect("fixture should be written");

        let outside_file = outside.join("escape.rs");
        fs::write(&outside_file, "fn escape() {}\n").expect("outside file should be written");
        symlink(&outside_file, src.join("escape.rs")).expect("symlink should be created");

        let files = discover_source_files(&root, true, &["rs"]).expect("walk should succeed");
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("src/lib.rs"));

        fs::remove_dir_all(root).expect("root temp dir should be removed");
        fs::remove_dir_all(outside).expect("outside temp dir should be removed");
    }
}
