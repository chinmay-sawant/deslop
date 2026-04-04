use std::fs;
use std::path::{Path, PathBuf};

use deslop::syntax_error_for_source;

const CORPUS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/corpus/regressions");

#[test]
fn corpus_regressions_cover_malformed_generated_and_edge_cases() {
    for entry in collect_sources(Path::new(CORPUS_ROOT)) {
        let source = fs::read_to_string(&entry)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", entry.display()));
        let relative = entry
            .strip_prefix(CORPUS_ROOT)
            .unwrap_or_else(|error| panic!("failed to strip corpus prefix: {error}"));
        let language_extension = relative
            .components()
            .next()
            .and_then(|component| component.as_os_str().to_str())
            .and_then(|language| match language {
                "go" => Some("go"),
                "python" => Some("py"),
                "rust" => Some("rs"),
                _ => None,
            })
            .unwrap_or_else(|| panic!("unexpected corpus language in {}", relative.display()));
        let synthetic_path = PathBuf::from(format!("corpus_fixture.{language_extension}"));
        let syntax_error = syntax_error_for_source(&synthetic_path, &source)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", entry.display()));

        let category = relative
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>");

        match category {
            "malformed" => {
                assert!(
                    syntax_error,
                    "{} should keep syntax_error=true",
                    relative.display()
                );
            }
            "generated" | "edge_cases" => {
                assert!(
                    !syntax_error,
                    "{} should parse as a valid corpus regression",
                    relative.display()
                );
            }
            other => panic!(
                "unexpected corpus category {other} in {}",
                relative.display()
            ),
        }
    }
}

fn collect_sources(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_sources_recursive(root, &mut files);
    files.sort();
    files
}

fn collect_sources_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_sources_recursive(&path, files);
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension == "txt")
        {
            files.push(path);
        }
    }
}
