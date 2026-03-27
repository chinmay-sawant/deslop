use std::fs::{File, OpenOptions, symlink_metadata};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use crate::{Error, Result};

pub const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024;

pub fn read_to_string_limited(path: &Path, max_bytes: u64) -> Result<String> {
    reject_symlink(path)?;
    let file = open_readonly(path)?;
    let size = file
        .metadata()
        .map_err(|error| Error::io(path, error))?
        .len();

    if size > max_bytes {
        return Err(Error::InputTooLarge {
            path: path.to_path_buf(),
            size,
            max_bytes,
        });
    }

    let mut reader = BufReader::new(file);
    let mut source = String::new();
    let bytes_read = u64::try_from(
        reader
        .by_ref()
        .take(max_bytes + 1)
        .read_to_string(&mut source)
        .map_err(|error| Error::io(path, error))?,
    )
    .map_err(|_| Error::byte_count_overflow(path, source.len()))?;

    if bytes_read > max_bytes {
        return Err(Error::InputTooLarge {
            path: path.to_path_buf(),
            size: bytes_read,
            max_bytes,
        });
    }

    Ok(source)
}

pub(crate) fn canonicalize_within_root(root: &Path, candidate: &Path) -> Result<PathBuf> {
    let canonical_root = root.canonicalize().map_err(|error| Error::io(root, error))?;
    reject_symlink(candidate)?;
    let canonical_path = candidate
        .canonicalize()
        .map_err(|error| Error::io(candidate, error))?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err(Error::path_outside_root(canonical_path, canonical_root));
    }

    Ok(canonical_path)
}

fn reject_symlink(path: &Path) -> Result<()> {
    let metadata = symlink_metadata(path).map_err(|error| Error::io(path, error))?;
    if metadata.file_type().is_symlink() {
        return Err(Error::symlink_rejected(path));
    }

    Ok(())
}

#[cfg(unix)]
fn open_readonly(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| Error::io(path, error))
}

#[cfg(not(unix))]
fn open_readonly(path: &Path) -> Result<File> {
    File::open(path).map_err(|error| Error::io(path, error))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use proptest::prelude::*;

    use super::read_to_string_limited;
    use crate::Error;

    fn temp_file(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("deslop-io-{name}-{nonce}.txt"))
    }

    #[test]
    fn reads_small_file() {
        let path = temp_file("small");
        fs::write(&path, "hello world").expect("temp file write should succeed");

        let contents = read_to_string_limited(&path, 64).expect("small file should read");
        assert_eq!(contents, "hello world");

        fs::remove_file(path).expect("temp file cleanup should succeed");
    }

    #[test]
    fn rejects_oversized_file() {
        let path = temp_file("large");
        fs::write(&path, "0123456789abcdef").expect("temp file write should succeed");

        let error = read_to_string_limited(&path, 4).expect_err("oversized file should fail");
        assert!(matches!(error, Error::InputTooLarge { .. }));

        fs::remove_file(path).expect("temp file cleanup should succeed");
    }

    #[test]
    fn accepts_input_at_exact_limit() {
        let path = temp_file("exact-limit");
        fs::write(&path, "1234").expect("temp file write should succeed");

        let contents = read_to_string_limited(&path, 4).expect("exact limit should succeed");
        assert_eq!(contents, "1234");

        fs::remove_file(path).expect("temp file cleanup should succeed");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_input() {
        use std::os::unix::fs::symlink;

        let target = temp_file("symlink-target");
        let link = temp_file("symlink-link");
        fs::write(&target, "hello").expect("target file write should succeed");
        symlink(&target, &link).expect("symlink should be created");

        let error = read_to_string_limited(&link, 32).expect_err("symlinked file should fail");
        assert!(matches!(error, Error::SymlinkRejected { .. }));

        fs::remove_file(link).expect("symlink cleanup should succeed");
        fs::remove_file(target).expect("target cleanup should succeed");
    }

    proptest! {
        #[test]
        fn bounded_reads_track_input_size(len in 0usize..256, limit in 0usize..256) {
            let path = temp_file("prop-limit");
            let contents = "x".repeat(len);
            fs::write(&path, &contents).expect("temp file write should succeed");

            let result = read_to_string_limited(
                &path,
                u64::try_from(limit).expect("proptest limit should fit into u64"),
            );
            prop_assert_eq!(result.is_ok(), len <= limit);

            if let Ok(read_back) = result {
                prop_assert_eq!(read_back.len(), len);
            }

            fs::remove_file(path).expect("temp file cleanup should succeed");
        }
    }
}