use std::path::{Path, PathBuf};

use super::{AnalysisResult, ParsedFile};
use crate::{DEFAULT_MAX_BYTES, read_to_string_limited};

const FIXTURE_BUNDLE_HEADER: &str = "-- path: ";

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub(crate) fn load_fixture(relative_path: &str) -> String {
    let path = fixture_root().join(relative_path);
    let source = read_to_string_limited(&path, DEFAULT_MAX_BYTES);
    assert!(source.is_ok(), "failed to load fixture {}", path.display());
    match source {
        Ok(source) => source,
        Err(_) => unreachable!("asserted above"),
    }
}

pub(crate) fn parse_fixture_bundle(
    relative_path: &str,
    parse: fn(&Path, &str) -> AnalysisResult<ParsedFile>,
) -> Vec<ParsedFile> {
    fixture_bundle_entries(&load_fixture(relative_path))
        .into_iter()
        .map(|(repo_path, source)| parse_fixture_source(&repo_path, &source, parse))
        .collect()
}

pub(crate) fn find_parsed_file<'a>(files: &'a [ParsedFile], repo_path: &str) -> &'a ParsedFile {
    let parsed = files.iter().find(|file| file.path == Path::new(repo_path));
    assert!(parsed.is_some(), "missing parsed fixture file: {repo_path}");
    match parsed {
        Some(file) => file,
        None => unreachable!("asserted above"),
    }
}

fn parse_fixture_source(
    repo_path: &str,
    source: &str,
    parse: fn(&Path, &str) -> AnalysisResult<ParsedFile>,
) -> ParsedFile {
    let parsed = parse(Path::new(repo_path), source);
    assert!(parsed.is_ok(), "fixture source should parse: {repo_path}");
    match parsed {
        Ok(file) => file,
        Err(_) => unreachable!("asserted above"),
    }
}

fn fixture_bundle_entries(contents: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut current_path = None;
    let mut current_source = String::new();

    for line in contents.lines() {
        if let Some(path) = line.strip_prefix(FIXTURE_BUNDLE_HEADER) {
            if let Some(previous_path) = current_path.replace(path.to_string()) {
                entries.push((previous_path, std::mem::take(&mut current_source)));
            }
            continue;
        }

        assert!(
            current_path.is_some(),
            "fixture bundle must start with '{}'",
            FIXTURE_BUNDLE_HEADER
        );

        if !current_source.is_empty() {
            current_source.push('\n');
        }
        current_source.push_str(line);
    }

    if let Some(path) = current_path {
        entries.push((path, current_source));
    }

    assert!(
        !entries.is_empty(),
        "fixture bundle should contain at least one file"
    );

    entries
}
