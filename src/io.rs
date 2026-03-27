use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::{Error, Result};

pub const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024;

pub fn read_to_string_limited(path: &Path, max_bytes: u64) -> Result<String> {
    let file = File::open(path).map_err(|error| Error::io(path, error))?;
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
    let bytes_read = reader
        .by_ref()
        .take(max_bytes + 1)
        .read_to_string(&mut source)
        .map_err(|error| Error::io(path, error))? as u64;

    if bytes_read > max_bytes {
        return Err(Error::InputTooLarge {
            path: path.to_path_buf(),
            size: bytes_read,
            max_bytes,
        });
    }

    Ok(source)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

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
}