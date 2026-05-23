use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::io::{DEFAULT_MAX_BYTES, read_to_string_limited};
use crate::model::Finding;
use crate::rules::RuleLanguage;

pub(crate) struct FunctionContext {
    pub start_line: usize,
    pub end_line: usize,
    pub lines: Vec<String>,
}

pub(crate) fn resolve_function_context(
    finding: &Finding,
    parsed_files: &[ParsedFile],
    cached_lines: &mut HashMap<PathBuf, Vec<String>>,
) -> FunctionContext {
    if !finding.path.exists() {
        return FunctionContext {
            start_line: finding.start_line,
            end_line: finding.start_line,
            lines: Vec::new(),
        };
    }

    if let Some(parsed_file) = parsed_files.iter().find(|file| file.path == finding.path)
        && let Some(function) = find_enclosing_function(
            parsed_file,
            finding.start_line,
            finding.function_name.as_deref(),
        )
    {
        let start_line = function.fingerprint.start_line;
        let end_line = function.fingerprint.end_line;
        let lines = load_source_lines(&finding.path, cached_lines);
        let function_lines = slice_lines(&lines, start_line, end_line);
        if !function_lines.is_empty() {
            return FunctionContext {
                start_line,
                end_line,
                lines: function_lines,
            };
        }
    }

    let lines = load_source_lines(&finding.path, cached_lines);
    if let Some((start_line, end_line, function_lines)) =
        extract_enclosing_function(&lines, &finding.path, finding.start_line)
    {
        return FunctionContext {
            start_line,
            end_line,
            lines: function_lines,
        };
    }

    FunctionContext {
        start_line: finding.start_line,
        end_line: finding.start_line,
        lines: Vec::new(),
    }
}

fn find_enclosing_function<'a>(
    file: &'a ParsedFile,
    line_no: usize,
    function_name: Option<&str>,
) -> Option<&'a ParsedFunction> {
    file.functions
        .iter()
        .find(|function| {
            function.fingerprint.start_line <= line_no && line_no <= function.fingerprint.end_line
        })
        .or_else(|| {
            function_name.and_then(|name| {
                file.functions
                    .iter()
                    .find(|function| function.fingerprint.name == name)
            })
        })
}

fn load_source_lines(path: &Path, cache: &mut HashMap<PathBuf, Vec<String>>) -> Vec<String> {
    if let Some(lines) = cache.get(path) {
        return lines.clone();
    }

    let lines: Vec<String> = read_to_string_limited(path, DEFAULT_MAX_BYTES)
        .map(|content| content.lines().map(str::to_string).collect())
        .unwrap_or_default();
    cache.insert(path.to_path_buf(), lines.clone());
    lines
}

fn slice_lines(lines: &[String], start_line: usize, end_line: usize) -> Vec<String> {
    if start_line == 0 || end_line == 0 || start_line > lines.len() {
        return Vec::new();
    }
    lines[start_line.saturating_sub(1)..end_line.min(lines.len())].to_vec()
}

fn extract_enclosing_function(
    lines: &[String],
    file_path: &Path,
    line_no: usize,
) -> Option<(usize, usize, Vec<String>)> {
    if lines.is_empty() || line_no == 0 || line_no > lines.len() {
        return None;
    }

    let target_idx = line_no - 1;
    let suffix = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let span = if suffix == "py" {
        extract_python_function(lines, target_idx)
    } else {
        extract_brace_function(lines, target_idx, &suffix)
    }?;

    let (start_idx, end_idx) = span;
    Some((
        start_idx + 1,
        end_idx + 1,
        lines[start_idx..=end_idx].to_vec(),
    ))
}

fn extract_python_function(lines: &[String], target_idx: usize) -> Option<(usize, usize)> {
    let mut signature_idx = None;
    for idx in (0..=target_idx).rev() {
        let trimmed = lines[idx].trim_start();
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            signature_idx = Some(idx);
            break;
        }
    }
    let signature_idx = signature_idx?;

    let base_indent = leading_spaces(&lines[signature_idx]);
    let mut end_idx = lines.len().saturating_sub(1);
    for (line_idx, line) in lines.iter().enumerate().skip(signature_idx + 1) {
        let stripped = line.trim();
        if stripped.is_empty() {
            continue;
        }
        let indent = leading_spaces(line);
        if indent <= base_indent {
            end_idx = line_idx.saturating_sub(1);
            break;
        }
    }

    let mut decorator_start = signature_idx;
    while decorator_start > 0 && lines[decorator_start - 1].trim_start().starts_with('@') {
        decorator_start -= 1;
    }

    if decorator_start <= target_idx && target_idx <= end_idx {
        Some((decorator_start, end_idx))
    } else {
        None
    }
}

fn extract_brace_function(
    lines: &[String],
    target_idx: usize,
    suffix: &str,
) -> Option<(usize, usize)> {
    let mut signature_idx = None;
    for idx in (0..=target_idx).rev() {
        if is_function_signature(&lines[idx], suffix) {
            signature_idx = Some(idx);
            break;
        }
    }
    let signature_idx = signature_idx?;

    let mut open_idx = None;
    for (idx, line) in lines.iter().enumerate().skip(signature_idx) {
        if line.contains('{') {
            open_idx = Some(idx);
            break;
        }
    }
    let open_idx = open_idx?;

    let mut depth = 0;
    let mut end_idx = open_idx;
    for (idx, line) in lines.iter().enumerate().skip(open_idx) {
        let (delta, _) = brace_scan_line(line, false);
        if (idx == open_idx && delta > 0) || idx > open_idx {
            depth += delta;
        }
        if depth == 0 && idx >= open_idx {
            end_idx = idx;
            break;
        }
    }

    let mut start_idx = signature_idx;
    while start_idx > 0 {
        let trimmed = lines[start_idx - 1].trim_start();
        let is_leading_comment = (suffix == "rs"
            && (trimmed.starts_with("#[") || trimmed.starts_with("///")))
            || (suffix == "go" && trimmed.starts_with("//"));
        if is_leading_comment {
            start_idx -= 1;
        } else {
            break;
        }
    }

    if start_idx <= target_idx && target_idx <= end_idx {
        Some((start_idx, end_idx))
    } else {
        None
    }
}

fn is_function_signature(line: &str, suffix: &str) -> bool {
    let trimmed = line.trim_start();
    match suffix {
        "rs" => trimmed.contains(" fn ") || trimmed.starts_with("fn "),
        "go" => trimmed.starts_with("func "),
        _ => trimmed.contains('(') && !trimmed.contains(';'),
    }
}

fn leading_spaces(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn brace_scan_line(line: &str, mut in_block_comment: bool) -> (i32, bool) {
    let mut delta = 0;
    let mut i = 0;
    let chars: Vec<char> = line.chars().collect();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_backtick = false;
    let mut escape = false;

    while i < chars.len() {
        let ch = chars[i];
        let nxt = chars.get(i + 1).copied().unwrap_or('\0');

        if in_block_comment {
            if ch == '*' && nxt == '/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if escape {
            escape = false;
            i += 1;
            continue;
        }

        if in_single {
            if ch == '\\' {
                escape = true;
            } else if ch == '\'' {
                in_single = false;
            }
            i += 1;
            continue;
        }

        if in_double {
            if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_double = false;
            }
            i += 1;
            continue;
        }

        if in_backtick {
            if ch == '`' {
                in_backtick = false;
            }
            i += 1;
            continue;
        }

        if ch == '/' && nxt == '*' {
            in_block_comment = true;
            i += 2;
            continue;
        }
        if ch == '/' && nxt == '/' {
            break;
        }
        if ch == '\'' {
            in_single = true;
            i += 1;
            continue;
        }
        if ch == '"' {
            in_double = true;
            i += 1;
            continue;
        }
        if ch == '`' {
            in_backtick = true;
            i += 1;
            continue;
        }
        if ch == '{' {
            delta += 1;
        } else if ch == '}' {
            delta -= 1;
        }
        i += 1;
    }

    (delta, in_block_comment)
}

pub(crate) fn infer_language(path: &Path) -> Option<RuleLanguage> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("go") => Some(RuleLanguage::Go),
        Some("py") => Some(RuleLanguage::Python),
        Some("rs") => Some(RuleLanguage::Rust),
        _ => None,
    }
}
