use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

pub(crate) mod plan1;
pub(crate) mod plan2;
pub(crate) mod plan3;
pub(crate) mod plan4;

pub(crate) fn file_findings(file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(plan1::file_findings(file));
    findings.extend(plan2::file_findings(file));
    findings.extend(plan4::file_findings(file, index));
    findings
}

pub(crate) fn function_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(plan1::function_findings(file, function));
    findings.extend(plan3::function_findings(file, function));
    findings.extend(plan4::function_findings(file, function));
    findings
}

pub(crate) fn is_test_like(file: &ParsedFile, function: Option<&ParsedFunction>) -> bool {
    function.is_some_and(|function| function.is_test_function) || file.is_test_file
}

pub(crate) fn is_main_like_file(file: &ParsedFile) -> bool {
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    path.ends_with("/main.rs") || path.ends_with("main.rs") || path.contains("/bin/")
}

pub(crate) fn contains_any(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| text.contains(marker))
}

pub(crate) fn first_line_with_any(body: &str, base_line: usize, markers: &[&str]) -> Option<usize> {
    body.lines()
        .enumerate()
        .find(|(_, line)| contains_any(line, markers))
        .map(|(offset, _)| base_line + offset)
}

pub(crate) fn has_secret_like_text(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    [
        "password",
        "secret",
        "token",
        "api_key",
        "apikey",
        "access_token",
        "private_key",
        "cert",
        "certificate",
        "auth",
        "key",
    ]
    .iter()
    .any(|token| normalized.contains(token))
}

pub(crate) fn has_numeric_narrowing_cast(line: &str) -> bool {
    [
        " as u8",
        " as u16",
        " as u32",
        " as u64",
        " as usize",
        " as i8",
        " as i16",
        " as i32",
        " as i64",
        " as isize",
    ]
    .iter()
    .any(|marker| line.contains(marker))
}

pub(crate) fn has_red_flag_attribute(text: &str, needle: &str) -> bool {
    text.contains(needle)
}

pub(crate) fn file_attributes(file: &ParsedFile) -> &[crate::analysis::RustAttributeSummary] {
    file.rust_attributes()
}

pub(crate) fn file_has_attribute(file: &ParsedFile, needle: &str) -> bool {
    file_attributes(file)
        .iter()
        .any(|attribute| attribute.text.contains(needle))
}

pub(crate) fn file_top_level_attribute_lines(file: &ParsedFile, needle: &str) -> Vec<usize> {
    file_attributes(file)
        .iter()
        .filter(|attribute| attribute.text.contains(needle))
        .map(|attribute| attribute.line)
        .collect()
}

pub(crate) fn severity_for(file: &ParsedFile, warning_if_public: bool, base: Severity) -> Severity {
    if warning_if_public && !file.is_test_file {
        Severity::Warning
    } else {
        base
    }
}
