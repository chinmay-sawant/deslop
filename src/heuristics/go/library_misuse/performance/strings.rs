use super::*;

pub(super) fn string_operation_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(strings_contains_vs_index(file, function, lines));
    findings.extend(string_to_byte_single_char(file, function, lines));
    findings.extend(string_concat_for_path(file, function, lines));
    findings.extend(sprintf_int_to_string(file, function, lines));
    findings.extend(sprintf_simple_string(file, function, lines));
    findings.extend(strings_replace_single_char(file, function, lines));
    findings.extend(repeated_string_trim(file, function, lines));
    findings.extend(len_string_empty_check(file, function, lines));
    findings.extend(string_format_error_wrap(file, function, lines));
    findings.extend(hasprefix_then_trimprefix(file, function, lines));
    findings.extend(hassuffix_then_trimsuffix(file, function, lines));
    findings.extend(builder_write_string_plus(file, function, lines));
    findings
}

// A1
fn strings_contains_vs_index(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "strings") {
        for bl in lines {
            let pat1 = format!("{alias}.Index(");
            if bl.text.contains(&pat1)
                && (bl.text.contains("!= -1")
                    || bl.text.contains(">= 0")
                    || bl.text.contains("> -1"))
            {
                findings.push(Finding {
                    rule_id: "strings_contains_vs_index".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses strings.Index instead of strings.Contains",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Index(...) != -1 at line {}", alias, bl.line),
                        "strings.Contains communicates intent better".into(),
                    ],
                });
            }
        }
    }
    findings
}

// A2
fn string_to_byte_single_char(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if (bl.text.contains("string(") && bl.text.contains(") == \"") && bl.text.len() < 200)
            || (bl.text.contains("[]byte(") && bl.text.contains(")[0]"))
        {
            let is_single = bl.text.contains("== \"") && {
                let after = bl.text.split("== \"").nth(1).unwrap_or("");
                after.starts_with(|c: char| c != '"') && after.chars().nth(1) == Some('"')
            };
            if is_single || bl.text.contains("[0]") {
                findings.push(Finding {
                    rule_id: "string_to_byte_for_single_char_check".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} converts string/byte for single character comparison",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("single-char comparison via conversion at line {}", bl.line),
                        "direct byte indexing avoids allocation".into(),
                    ],
                });
            }
        }
    }
    findings
}

// A3
fn string_concat_for_path(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let has_filepath = has_import_path(file, "path/filepath");
    for bl in lines {
        if !has_filepath
            && (bl.text.contains("+ \"/\" +")
                || bl.text.contains("+ `\\` +")
                || bl.text.contains("+ \"\\\\\" +"))
        {
            findings.push(Finding {
                rule_id: "string_concatenation_for_path_join".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} builds file paths with string concatenation",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("manual path assembly at line {}", bl.line),
                    "filepath.Join handles separators correctly".into(),
                ],
            });
        }
    }
    findings
}

// A4
fn sprintf_int_to_string(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "fmt") {
        for bl in lines {
            let pat = format!("{alias}.Sprintf(\"%d\"");
            if bl.text.contains(&pat) {
                findings.push(Finding {
                    rule_id: "sprintf_for_simple_int_to_string".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses fmt.Sprintf for integer-to-string conversion",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Sprintf(\"%d\", ...) at line {}", alias, bl.line),
                        "strconv.Itoa is ~6× faster with fewer allocations".into(),
                    ],
                });
            }
        }
    }
    findings
}

// A5
fn sprintf_simple_string(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "fmt") {
        for bl in lines {
            let pat = format!("{alias}.Sprintf(\"");
            if bl.text.contains(&pat) {
                let after = bl.text.split(&pat).nth(1).unwrap_or("");
                if let Some(fmt_str) = after.split('"').next() {
                    let has_only_s = fmt_str.contains("%s")
                        && !fmt_str.contains("%d")
                        && !fmt_str.contains("%v")
                        && !fmt_str.contains("%f")
                        && !fmt_str.contains("%+")
                        && !fmt_str.contains("%#")
                        && !fmt_str.contains("%w");
                    let count = fmt_str.matches("%s").count();
                    if has_only_s && count >= 2 {
                        findings.push(Finding {
                            rule_id: "sprintf_for_simple_string_format".into(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: bl.line,
                            end_line: bl.line,
                            message: format!(
                                "function {} uses fmt.Sprintf with only %s verbs",
                                function.fingerprint.name
                            ),
                            evidence: vec![
                                format!(
                                    "{}.Sprintf with %s-only format at line {}",
                                    alias, bl.line
                                ),
                                "string concatenation or strings.Join avoids reflection".into(),
                            ],
                        });
                    }
                }
            }
        }
    }
    findings
}

// A6
fn strings_replace_single_char(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "strings") {
        for bl in lines {
            let pat = format!("{alias}.ReplaceAll(");
            if bl.text.contains(&pat) {
                let after = bl.text.split(&pat).nth(1).unwrap_or("");
                let args: Vec<&str> = after.splitn(6, '"').collect();
                // args: ["s, ", "x", ", ", "y", ")"]
                if args.len() >= 4 && args[1].len() == 1 && args[3].len() <= 1 {
                    findings.push(Finding {
                        rule_id: "strings_replace_all_for_single_char".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} uses strings.ReplaceAll for single character replacement",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "{}.ReplaceAll with single-char args at line {}",
                                alias, bl.line
                            ),
                            "strings.Map is ~2× faster for single-char replacement".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

// A7
fn repeated_string_trim(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "strings") {
        for bl in lines {
            let trim = format!("{alias}.Trim");
            let lower = format!("{alias}.ToLower(");
            let upper = format!("{alias}.ToUpper(");
            let count = [&trim, &lower, &upper]
                .iter()
                .filter(|p| bl.text.contains(p.as_str()))
                .count();
            if count >= 2
                && (bl.text.contains(&format!("{alias}.TrimSpace("))
                    || bl.text.contains(&format!("{alias}.TrimPrefix("))
                    || bl.text.contains(&format!("{alias}.TrimSuffix(")))
            {
                findings.push(Finding {
                    rule_id: "repeated_string_trim_normalize".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} chains multiple string normalization operations",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("chained trim/case operations at line {}", bl.line),
                        "a single-pass normalizer avoids multiple string scans".into(),
                    ],
                });
            }
        }
    }
    findings
}

// A8
fn len_string_empty_check(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if !bl.text.contains("len(") {
            continue;
        }
        let is_empty_check = bl.text.contains(") == 0")
            || bl.text.contains(")==0")
            || bl.text.contains(") != 0")
            || bl.text.contains(")!=0");
        if !is_empty_check {
            continue;
        }
        let inner = bl
            .text
            .split("len(")
            .nth(1)
            .and_then(|suffix| suffix.split(')').next())
            .map(str::trim);
        let Some(inner) = inner else {
            continue;
        };
        if is_identifier_name(inner) && identifier_is_likely_string(function, lines, inner, bl.line)
        {
            findings.push(Finding {
                rule_id: "len_string_for_empty_check".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses len(s) for an empty-string check",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("len(string) empty check at line {}", bl.line),
                    "prefer s == \"\" for empty checks and len(s) > 0 for non-empty checks".into(),
                ],
            });
        }
    }
    findings
}

fn identifier_is_likely_string(
    function: &ParsedFunction,
    lines: &[BodyLine],
    name: &str,
    current_line: usize,
) -> bool {
    signature_declares_string_identifier(&function.signature_text, name)
        || function
            .local_strings
            .iter()
            .any(|literal| literal.name == name && literal.line < current_line)
        || local_binding_declares_string(lines, name, current_line)
}

fn signature_declares_string_identifier(signature_text: &str, name: &str) -> bool {
    let normalized = signature_text.replace('\n', " ");
    normalized.contains(&format!("{name} string"))
        || normalized.contains(&format!("{name} ...string"))
}

fn local_binding_declares_string(lines: &[BodyLine], name: &str, current_line: usize) -> bool {
    for line in lines {
        if line.line >= current_line {
            break;
        }

        let trimmed = line.text.trim();
        if trimmed.starts_with(&format!("var {name} string"))
            || trimmed.starts_with(&format!("var {name} ="))
        {
            return true;
        }

        let Some((left, right)) = split_assignment(trimmed) else {
            continue;
        };

        if !binding_mentions_identifier(left, name) {
            continue;
        }

        if rhs_is_likely_string(right) {
            return true;
        }
    }

    false
}

fn binding_mentions_identifier(left: &str, name: &str) -> bool {
    left.trim()
        .trim_start_matches("var ")
        .replace(',', " ")
        .split_whitespace()
        .any(|token| token == name)
}

fn rhs_is_likely_string(right: &str) -> bool {
    let trimmed = right.trim();
    if trimmed.starts_with("[]byte(")
        || trimmed.starts_with("[]rune(")
        || trimmed.starts_with("make([]")
        || trimmed.contains("strings.Split(")
        || trimmed.contains("strings.Fields(")
        || trimmed.contains("FindAll")
        || trimmed.contains("FindSubmatch")
    {
        return false;
    }

    trimmed.starts_with('"')
        || trimmed.starts_with('`')
        || trimmed.contains("string(")
        || trimmed.contains(".String()")
        || trimmed.contains(".Error()")
        || trimmed.contains("C.GoString(")
        || trimmed.contains("fmt.Sprintf(")
        || trimmed.contains("strings.Trim")
        || trimmed.contains("strings.To")
        || trimmed.contains("strings.Join(")
        || trimmed.contains("strings.Replace")
        || trimmed.contains("strings.Clone(")
        || trimmed.contains("strings.Repeat(")
        || trimmed.contains("strings.Map(")
}

// A9
fn string_format_error_wrap(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "fmt") {
        for bl in lines {
            let pat = format!("{alias}.Errorf(");
            if bl.text.contains(&pat) && bl.text.contains("%s") && bl.text.contains(".Error()") {
                findings.push(Finding {
                    rule_id: "string_format_for_error_wrap".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses %s with err.Error() instead of %w",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Errorf with %s and .Error() at line {}", alias, bl.line),
                        "%w wraps without stringifying and preserves error chain".into(),
                    ],
                });
            }
        }
    }
    findings
}

// A10
fn hasprefix_then_trimprefix(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "strings") {
        let has_pat = format!("{alias}.HasPrefix(");
        let trim_pat = format!("{alias}.TrimPrefix(");
        for (i, bl) in lines.iter().enumerate() {
            if bl.text.contains(&has_pat) {
                for next in lines.iter().skip(i + 1).take(5) {
                    if next.text.contains(&trim_pat) {
                        findings.push(Finding {
                            rule_id: "strings_hasprefix_then_trimprefix".into(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: bl.line,
                            end_line: next.line,
                            message: format!(
                                "function {} checks HasPrefix then TrimPrefix (use CutPrefix)",
                                function.fingerprint.name
                            ),
                            evidence: vec![
                                format!(
                                    "HasPrefix at line {}, TrimPrefix at line {}",
                                    bl.line, next.line
                                ),
                                "strings.CutPrefix (Go 1.20+) does both in one scan".into(),
                            ],
                        });
                        break;
                    }
                }
            }
        }
    }
    findings
}

// A11
fn hassuffix_then_trimsuffix(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "strings") {
        let has_pat = format!("{alias}.HasSuffix(");
        let trim_pat = format!("{alias}.TrimSuffix(");
        for (i, bl) in lines.iter().enumerate() {
            if bl.text.contains(&has_pat) {
                for next in lines.iter().skip(i + 1).take(5) {
                    if next.text.contains(&trim_pat) {
                        findings.push(Finding {
                            rule_id: "strings_hassuffix_then_trimsuffix".into(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: bl.line,
                            end_line: next.line,
                            message: format!(
                                "function {} checks HasSuffix then TrimSuffix (use CutSuffix)",
                                function.fingerprint.name
                            ),
                            evidence: vec![
                                format!(
                                    "HasSuffix at line {}, TrimSuffix at line {}",
                                    bl.line, next.line
                                ),
                                "strings.CutSuffix (Go 1.20+) does both in one scan".into(),
                            ],
                        });
                        break;
                    }
                }
            }
        }
    }
    findings
}

// A12
fn builder_write_string_plus(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains(".WriteString(") && bl.text.contains(" + ") && !bl.text.contains("fmt.")
        {
            findings.push(Finding {
                rule_id: "string_builder_write_string_vs_plus".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} concatenates strings before WriteString",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("WriteString(a + b) at line {}", bl.line),
                    "separate WriteString calls avoid intermediate allocation".into(),
                ],
            });
        }
    }
    findings
}

// ── Section B — Slice And Map Operations ──

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::analysis::parse_source_file;
    use crate::heuristics::go::framework_patterns::body_lines;

    use super::len_string_empty_check;

    macro_rules! go_fixture {
        ($name:literal) => {
            include_str!(concat!("../../../../../tests/fixtures/go/", $name, ".txt"))
        };
    }

    #[test]
    fn len_string_empty_check_flags_string_parameters() {
        let source = go_fixture!("library_misuse_len_string_parameter_positive");

        let file =
            parse_source_file(Path::new("sample.go"), source).expect("go source should parse");
        let function = file.functions[0].clone();
        let lines = body_lines(&function);
        let findings = len_string_empty_check(&file, &function, &lines);

        assert_eq!(
            findings.len(),
            1,
            "string parameters should still be flagged"
        );
        assert_eq!(findings[0].rule_id, "len_string_for_empty_check");
    }

    #[test]
    fn len_string_empty_check_skips_slice_and_bytes_checks() {
        let source = go_fixture!("library_misuse_len_string_collections_negative");

        let file =
            parse_source_file(Path::new("sample.go"), source).expect("go source should parse");
        let function = file.functions[0].clone();
        let lines = body_lines(&function);
        let findings = len_string_empty_check(&file, &function, &lines);

        assert!(
            findings.is_empty(),
            "collection length checks should not be reported as string checks"
        );
    }

    #[test]
    fn len_string_empty_check_skips_local_collection_bindings() {
        let source = go_fixture!("library_misuse_len_string_local_collections_negative");

        let file =
            parse_source_file(Path::new("sample.go"), source).expect("go source should parse");
        let function = file.functions[0].clone();
        let lines = body_lines(&function);
        let findings = len_string_empty_check(&file, &function, &lines);

        assert!(
            findings.is_empty(),
            "split results and rune slices should not be treated as strings"
        );
    }
}
