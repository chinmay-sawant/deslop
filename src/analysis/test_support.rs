use std::path::Path;

use super::ParsedFile;

pub(crate) fn find_parsed_file<'a>(files: &'a [ParsedFile], repo_path: &str) -> &'a ParsedFile {
    let parsed = files.iter().find(|file| file.path == Path::new(repo_path));
    assert!(parsed.is_some(), "missing parsed fixture file: {repo_path}");
    match parsed {
        Some(file) => file,
        None => unreachable!("asserted above"),
    }
}
