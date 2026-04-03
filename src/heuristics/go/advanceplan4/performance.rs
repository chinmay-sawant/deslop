use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::super::advanceplan3::{
    BodyLine, has_import_path, import_aliases_for, is_identifier_name, is_request_path_function,
    join_lines, split_assignment,
};

pub(super) fn performance_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(string_operation_findings(file, function, lines));
    findings.extend(slice_map_findings(file, function, lines));
    findings.extend(runtime_sync_findings(file, function, lines));
    findings.extend(io_encoding_findings(file, function, lines));
    findings.extend(error_interface_findings(file, function, lines));
    findings
}

// ── Section A — String And Byte Operations ──

fn string_operation_findings(
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
        if inner.is_some_and(is_identifier_name) {
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

fn slice_map_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(copy_append_idiom(file, function, lines));
    findings.extend(map_delete_loop(file, function, lines));
    findings.extend(sort_slice_vs_sort(file, function, lines));
    findings.extend(range_string_by_index(file, function, lines));
    findings.extend(map_lookup_double(file, function, lines));
    findings.extend(slice_grow_without_cap_hint(file, function, lines));
    findings.extend(interface_slice_alloc(file, function, lines));
    findings.extend(map_of_slices_prealloc(file, function, lines));
    findings.extend(clear_map_go121(file, function, lines));
    findings.extend(unnecessary_slice_copy(file, function, lines));
    findings.extend(three_index_slice_for_append(file, function, lines));
    findings.extend(range_copy_large_struct(file, function, lines));
    findings.extend(dense_int_set_map(file, function, lines));
    findings
}

// B1
fn copy_append_idiom(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("append([]") && bl.text.contains("(nil),") && bl.text.contains("...)") {
            findings.push(Finding {
                rule_id: "copy_append_idiom_waste".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} clones a slice via append(nil, src...)",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("append([]T(nil), src...) at line {}", bl.line),
                    "make(len(src)) + copy allocates the exact size once".into(),
                ],
            });
        }
    }
    findings
}

// B2
fn map_delete_loop(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        if bl.in_loop && bl.text.contains("delete(") {
            if let Some(prev) = lines.get(i.wrapping_sub(1)) {
                if prev.text.contains("for") && prev.text.contains("range") {
                    findings.push(Finding {
                        rule_id: "map_delete_in_loop_vs_new_map".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} deletes map entries in a loop",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("delete() in range loop at line {}", bl.line),
                            "creating a new map is O(1) vs O(n) iterative delete".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

// B3
fn sort_slice_vs_sort(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "sort") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.Sort("))
                && (bl.text.contains("StringSlice")
                    || bl.text.contains("IntSlice")
                    || bl.text.contains("Float64Slice"))
            {
                findings.push(Finding {
                    rule_id: "sort_slice_vs_sort_sort".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses sort.Sort with type adapter",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Sort with type adapter at line {}", alias, bl.line),
                        "slices.Sort (Go 1.21+) avoids interface dispatch overhead".into(),
                    ],
                });
            }
        }
    }
    findings
}

// B4
fn range_string_by_index(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("i := 0; i < len(") && bl.text.contains("); i++") {
            let next_lines: Vec<&BodyLine> = lines
                .iter()
                .filter(|l| l.line > bl.line && l.line <= bl.line + 3)
                .collect();
            for nl in &next_lines {
                if nl.text.contains("[i]") && !nl.text.contains("[]byte") {
                    findings.push(Finding {
                        rule_id: "range_over_string_by_index".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} iterates string by byte index",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("index-based string iteration at line {}", bl.line),
                            "for _, r := range s correctly handles multi-byte runes".into(),
                        ],
                    });
                    break;
                }
            }
        }
    }
    findings
}

// B5
fn map_lookup_double(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        if !(bl.text.contains(", ok :=") || bl.text.contains(", ok =")) {
            continue;
        }
        let Some(start) = bl.text.find('[') else {
            continue;
        };
        let Some(end) = bl.text[start..].find(']') else {
            continue;
        };
        let end = start + end;
        let key_expr = bl.text[start + 1..end].trim();
        let map_expr = bl.text[..start]
            .split_whitespace()
            .last()
            .unwrap_or("")
            .trim_start_matches("if");
        if map_expr.is_empty() || key_expr.is_empty() {
            continue;
        }
        for next in lines.iter().skip(i + 1).take(5) {
            if next.text.contains(&format!("{map_expr}[{key_expr}]")) {
                findings.push(Finding {
                    rule_id: "map_lookup_double_access".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: next.line,
                    message: format!(
                        "function {} performs double map lookup for same key",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!(
                            "first lookup at line {}, second at line {}",
                            bl.line, next.line
                        ),
                        "v, ok := m[k] does one lookup instead of two".into(),
                    ],
                });
                break;
            }
        }
    }
    findings
}

// B7
fn interface_slice_alloc(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains("[]interface{}") || bl.text.contains("[]any{") {
            if bl.text.contains("make(") || bl.text.contains(":=") || bl.text.contains("var ") {
                findings.push(Finding {
                    rule_id: "interface_slice_allocation".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses []interface{{}} or []any for homogeneous data",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("interface slice at line {}", bl.line),
                        "typed slices or generics avoid heap escape per element".into(),
                    ],
                });
            }
        }
    }
    findings
}

// B6
fn slice_grow_without_cap_hint(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if !bl.in_loop || !bl.text.contains("append(") {
            continue;
        }
        let Some(target) = bl
            .text
            .split("append(")
            .nth(1)
            .and_then(|suffix| suffix.split(',').next())
            .map(str::trim)
        else {
            continue;
        };
        if !is_identifier_name(target) {
            continue;
        }
        let has_zero_cap_init = lines.iter().any(|line| {
            line.line < bl.line
                && (line.text.starts_with(&format!("var {target} []"))
                    || line.text.starts_with(&format!("{target} := []"))
                    || (line.text.starts_with(&format!("{target} := make([]"))
                        && line.text.contains(", 0)")
                        && !line.text.contains(", 0,")))
        });
        if has_zero_cap_init {
            findings.push(Finding {
                rule_id: "slice_grow_without_cap_hint".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} appends to {} in a loop without a capacity hint",
                    function.fingerprint.name, target
                ),
                evidence: vec![
                    format!("append({target}, ...) inside loop at line {}", bl.line),
                    "make([]T, 0, len(source)) avoids repeated grow-and-copy cycles".into(),
                ],
            });
        }
    }
    findings
}

// B8
fn map_of_slices_prealloc(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.in_loop && bl.text.contains("] = append(") && bl.text.contains('[') {
            findings.push(Finding {
                rule_id: "map_of_slices_prealloc".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} appends to a map-of-slices entry without preallocating",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("map[key] = append(map[key], ...) at line {}", bl.line),
                    "pre-sizing inner slices avoids repeated header writes and backing-array churn"
                        .into(),
                ],
            });
        }
    }
    findings
}

// B9
fn clear_map_go121(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        if bl.in_loop && bl.text.contains("delete(") {
            if let Some(prev) = lines.get(i.wrapping_sub(1)) {
                if prev.text.contains("for") && prev.text.contains("range") {
                    findings.push(Finding {
                        rule_id: "clear_map_go121".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: prev.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} clears a map with per-key delete calls",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "range/delete clear pattern at lines {}-{}",
                                prev.line, bl.line
                            ),
                            "clear(m) (Go 1.21+) resets the map in a single runtime call".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

// B10
fn unnecessary_slice_copy(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, bl) in lines.iter().enumerate() {
        let Some((left, right)) = split_assignment(&bl.text) else {
            continue;
        };
        if !(right.contains("append([]") && right.contains("(nil),") && right.contains("...)"))
            && !right.contains("slices.Clone(")
        {
            continue;
        }
        let binding = left
            .trim()
            .trim_start_matches("var ")
            .split(',')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim();
        if !is_identifier_name(binding) {
            continue;
        }
        let mutated = lines.iter().skip(index + 1).take(8).any(|line| {
            line.text.contains(&format!("append({binding},"))
                || line.text.contains(&format!("{binding}["))
                || line.text.contains(&format!("{binding} ="))
        });
        if !mutated {
            findings.push(Finding {
                rule_id: "unnecessary_slice_copy_for_readonly".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} clones a slice that appears to be read-only",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "read-only slice clone assigned to {binding} at line {}",
                        bl.line
                    ),
                    "reusing the original slice avoids an extra allocation and full copy".into(),
                ],
            });
        }
    }
    findings
}

// B11
fn three_index_slice_for_append(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, bl) in lines.iter().enumerate() {
        let Some((left, right)) = split_assignment(&bl.text) else {
            continue;
        };
        if right.trim_start().starts_with("[]") || !right.contains('[') || !right.contains(']') {
            continue;
        }
        let slice_part = right
            .split('[')
            .nth(1)
            .and_then(|suffix| suffix.split(']').next())
            .unwrap_or("");
        if slice_part.matches(':').count() != 1 {
            continue;
        }
        let binding = left
            .trim()
            .trim_start_matches("var ")
            .split(',')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim();
        if !is_identifier_name(binding) {
            continue;
        }
        if let Some(next) = lines
            .iter()
            .skip(index + 1)
            .take(5)
            .find(|line| line.text.contains(&format!("append({binding},")))
        {
            findings.push(Finding {
                rule_id: "three_index_slice_for_append_safety".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: next.line,
                message: format!(
                    "function {} appends to a subslice without capping its capacity",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "subslice assignment at line {}, append at line {}",
                        bl.line, next.line
                    ),
                    "use original[a:b:b] so append cannot overwrite the parent slice".into(),
                ],
            });
        }
    }
    findings
}

// B12
fn range_copy_large_struct(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let signature_has_large_struct_slice = function.signature_text.contains("[]struct{")
        || file.go_structs.iter().any(|go_struct| {
            go_struct.fields.len() >= 4
                && function
                    .signature_text
                    .contains(&format!("[]{}", go_struct.name))
        });
    if !signature_has_large_struct_slice {
        return findings;
    }
    for bl in lines {
        if bl.text.starts_with("for _, ")
            && bl.text.contains(":= range")
            && !bl.text.contains("string")
            && !bl.text.contains("int")
            && !bl.text.contains("byte")
        {
            if let Some(slice_name) = bl
                .text
                .split("range ")
                .nth(1)
                .map(|s| s.trim().trim_end_matches(" {").trim())
            {
                if !slice_name.contains("map[") && !slice_name.contains("chan ") {
                    // heuristic: we flag when the variable name suggests a struct slice
                    let name_lower = slice_name.to_lowercase();
                    if name_lower.ends_with("items")
                        || name_lower.ends_with("records")
                        || name_lower.ends_with("entries")
                        || name_lower.ends_with("objects")
                        || name_lower.ends_with("models")
                        || name_lower.ends_with("rows")
                        || name_lower.ends_with("users")
                        || name_lower.ends_with("events")
                        || name_lower.ends_with("results")
                        || name_lower.ends_with("nodes")
                    {
                        findings.push(Finding {
                            rule_id: "range_copy_large_struct".into(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: bl.line, end_line: bl.line,
                            message: format!("function {} may copy large structs in range loop", function.fingerprint.name),
                            evidence: vec![
                                format!("for _, v := range {} at line {}", slice_name, bl.line),
                                "use index access for i := range s {{ v := &s[i] }} to avoid copies".into(),
                            ],
                        });
                    }
                }
            }
        }
    }
    findings
}

// B13
fn dense_int_set_map(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        let lower = bl.text.to_lowercase();
        if (bl.text.contains("map[int]bool") || bl.text.contains("map[int]struct{}"))
            && (lower.contains("seen") || lower.contains("set") || lower.contains("visited"))
        {
            findings.push(Finding {
                rule_id: "unnecessary_map_for_set_of_ints".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses a map as a dense integer set",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("map[int]set pattern at line {}", bl.line),
                    "for dense ranges, []bool or a bitset uses far less memory than a map".into(),
                ],
            });
        }
    }
    findings
}

// ── Section C — Runtime And Sync Primitives ──

fn runtime_sync_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(sync_mutex_for_counter(file, function, lines));
    findings.extend(sync_mutex_for_readonly_config(file, function, lines));
    findings.extend(sync_pool_ignored_small_allocs(file, function, lines));
    findings.extend(mutex_value_receiver(file, function, lines));
    findings.extend(time_now_tight_loop(file, function, lines));
    findings.extend(defer_in_tight_loop(file, function, lines));
    findings.extend(select_single_case(file, function, lines));
    findings.extend(goroutine_for_sync(file, function, lines));
    findings.extend(unbuffered_channel_known_count(file, function, lines));
    findings.extend(waitgroup_add_in_loop(file, function, lines));
    findings
}

// C1
fn sync_mutex_for_counter(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        if bl.text.contains(".Lock()") {
            let next_lines: Vec<&BodyLine> = lines.iter().skip(i + 1).take(3).collect();
            for nl in &next_lines {
                if (nl.text.contains("++")
                    || nl.text.contains("+= 1")
                    || nl.text.contains("-= 1")
                    || nl.text.contains("--"))
                    && !nl.text.contains("//")
                    && !nl.text.contains('.')
                {
                    let unlock = next_lines.iter().any(|l| l.text.contains(".Unlock()"));
                    if unlock {
                        findings.push(Finding {
                            rule_id: "sync_mutex_for_atomic_counter".into(),
                            severity: Severity::Warning,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: bl.line,
                            end_line: nl.line,
                            message: format!(
                                "function {} uses mutex for simple counter increment",
                                function.fingerprint.name
                            ),
                            evidence: vec![
                                format!("Lock/counter/Unlock pattern at line {}", bl.line),
                                "atomic.AddInt64 is lock-free and ~5× faster".into(),
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

// C2
fn sync_mutex_for_readonly_config(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, bl) in lines.iter().enumerate() {
        if !bl.text.contains(".RLock()") {
            continue;
        }
        let Some(unlock_line) = lines
            .iter()
            .skip(index + 1)
            .take(6)
            .find(|line| line.text.contains(".RUnlock()"))
        else {
            continue;
        };
        let has_config_read = lines
            .iter()
            .skip(index + 1)
            .take_while(|line| line.line <= unlock_line.line)
            .any(|line| {
                line.text.contains("config.")
                    || line.text.contains("cfg.")
                    || line.text.contains("settings.")
            });
        if has_config_read {
            findings.push(Finding {
                rule_id: "sync_mutex_for_readonly_config".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: unlock_line.line,
                message: format!(
                    "function {} locks around read-mostly config access",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "RLock/RUnlock config read at lines {}-{}",
                        bl.line, unlock_line.line
                    ),
                    "atomic.Value avoids lock overhead on the hot read path".into(),
                ],
            });
        }
    }
    findings
}

// C3
fn sync_pool_ignored_small_allocs(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        let hot = bl.in_loop || is_request_path_function(file, function);
        let alloc = bl.text.contains("make([]byte, 1024")
            || bl.text.contains("make([]byte, 2048")
            || bl.text.contains("make([]byte, 4096")
            || bl.text.contains("make([]byte, 8192")
            || bl.text.contains("make([]byte, 16384")
            || bl.text.contains("make([]byte, 32768")
            || bl.text.contains("new(bytes.Buffer)")
            || bl.text.contains("new(strings.Builder)");
        if hot && alloc {
            findings.push(Finding {
                rule_id: "sync_pool_ignored_for_frequent_small_allocs".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} repeatedly allocates reusable scratch objects",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("scratch allocation at line {}", bl.line),
                    "sync.Pool can amortize hot-path buffer allocations after warmup".into(),
                ],
            });
        }
    }
    findings
}

// C4
fn mutex_value_receiver(
    file: &ParsedFile,
    function: &ParsedFunction,
    _lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let Some(receiver) = &function.fingerprint.receiver_type else {
        return findings;
    };
    if function.signature_text.contains(&format!("*{receiver}")) {
        return findings;
    }
    let Some(go_struct) = file
        .go_structs
        .iter()
        .find(|summary| summary.name == *receiver)
    else {
        return findings;
    };
    if go_struct.fields.iter().any(|field| {
        field.type_text.contains("sync.Mutex") || field.type_text.contains("sync.RWMutex")
    }) {
        findings.push(Finding {
            rule_id: "mutex_value_receiver".into(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "method {} uses a value receiver on a type containing a mutex",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "value receiver on {receiver} at line {}",
                    function.fingerprint.start_line
                ),
                "copying a mutex by value is unsafe; use a pointer receiver".into(),
            ],
        });
    }
    findings
}

// C5
fn time_now_tight_loop(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "time") {
        for bl in lines {
            if bl.in_loop && bl.text.contains(&format!("{alias}.Now()")) {
                findings.push(Finding {
                    rule_id: "time_now_in_tight_loop".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} calls time.Now() inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Now() in loop at line {}", alias, bl.line),
                        "cache timestamp before loop if millisecond precision suffices".into(),
                    ],
                });
            }
        }
    }
    findings
}

// C6
fn defer_in_tight_loop(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.in_loop
            && bl.text.starts_with("defer ")
            && !bl.text.contains("wg.Done()")
            && !bl.text.contains(".Done()")
        {
            findings.push(Finding {
                rule_id: "defer_in_tight_loop".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} uses defer inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("defer in loop at line {}", bl.line),
                    "defers accumulate until function exit; extract to helper function".into(),
                ],
            });
        }
    }
    findings
}

// C7
fn select_single_case(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        if bl.text.starts_with("select {") || bl.text == "select{" {
            let mut case_count = 0;
            let mut has_default = false;
            for next in lines.iter().skip(i + 1).take(10) {
                if next.text.starts_with("case ") {
                    case_count += 1;
                }
                if next.text.starts_with("default:") {
                    has_default = true;
                }
                if next.text == "}" {
                    break;
                }
            }
            if case_count == 1 && !has_default {
                findings.push(Finding {
                    rule_id: "select_with_single_case".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses select with only one case",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("single-case select at line {}", bl.line),
                        "direct channel receive is ~40% faster".into(),
                    ],
                });
            }
        }
    }
    findings
}

// C8
fn goroutine_for_sync(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (i, bl) in lines.iter().enumerate() {
        if bl.text.contains("go func()") {
            for next in lines.iter().skip(i + 1).take(10) {
                if next.text.contains("<-") && !next.text.contains("go ") {
                    findings.push(Finding {
                        rule_id: "goroutine_for_sync_work".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: next.line,
                        message: format!(
                            "function {} spawns goroutine for immediately-awaited work",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "go func() at line {}, awaited at line {}",
                                bl.line, next.line
                            ),
                            "direct function call avoids ~1μs goroutine spawn overhead".into(),
                        ],
                    });
                    break;
                }
            }
        }
    }
    findings
}

// C9
fn unbuffered_channel_known_count(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, bl) in lines.iter().enumerate() {
        let Some((left, right)) = split_assignment(&bl.text) else {
            continue;
        };
        if !right.contains("make(chan") {
            continue;
        }
        let make_args = right.split("make(chan").nth(1).unwrap_or("");
        let inside = make_args.split(')').next().unwrap_or("");
        if inside.contains(',') {
            continue;
        }
        let channel = left
            .trim()
            .split(',')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim();
        if !is_identifier_name(channel) {
            continue;
        }
        let has_loop_send = lines
            .iter()
            .skip(index + 1)
            .take(12)
            .any(|line| line.text.contains(&format!("{channel} <-")) && line.in_loop);
        if has_loop_send {
            findings.push(Finding {
                rule_id: "unbuffered_channel_for_known_producer_count".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} creates an unbuffered channel for a bounded producer loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("make(chan T) at line {} with looped sends", bl.line),
                    "a bounded buffer can avoid repeated park/unpark synchronization".into(),
                ],
            });
        }
    }
    findings
}

// C10
fn waitgroup_add_in_loop(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.in_loop && bl.text.contains(".Add(1)") && bl.text.contains("wg") {
            findings.push(Finding {
                rule_id: "waitgroup_add_inside_loop".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} calls wg.Add(1) inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("wg.Add(1) in loop at line {}", bl.line),
                    "wg.Add(n) before the loop uses one atomic op instead of n".into(),
                ],
            });
        }
    }
    findings
}

// ── Section D — I/O And Encoding ──

fn io_encoding_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(ioutil_readall(file, function, lines));
    findings.extend(json_marshal_then_write(file, function, lines));
    findings.extend(binary_read_single_field(file, function, lines));
    findings.extend(json_number_without_use_number(file, function, lines));
    findings.extend(xml_decoder_trusted_strict(file, function, lines));
    findings.extend(csv_reader_reuse(file, function, lines));
    findings.extend(scanner_small_buffer(file, function, lines));
    findings.extend(http_body_readall_no_limit(file, function, lines));
    findings
}

// D1
fn ioutil_readall(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "io/ioutil") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.ReadAll(")) {
                findings.push(Finding {
                    rule_id: "ioutil_readall_still_used".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses deprecated ioutil.ReadAll",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.ReadAll at line {}", alias, bl.line),
                        "io.ReadAll is canonical since Go 1.16".into(),
                    ],
                });
            }
        }
    }
    findings
}

// D2
fn json_marshal_then_write(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "encoding/json") {
        for (i, bl) in lines.iter().enumerate() {
            if bl.text.contains(&format!("{alias}.Marshal(")) && !bl.text.contains("MarshalIndent")
            {
                for next in lines.iter().skip(i + 1).take(5) {
                    if next.text.contains(".Write(") || next.text.contains("w.Write(") {
                        findings.push(Finding {
                            rule_id: "json_marshal_then_write".into(),
                            severity: Severity::Info,
                            path: file.path.clone(),
                            function_name: Some(function.fingerprint.name.clone()),
                            start_line: bl.line, end_line: next.line,
                            message: format!("function {} marshals JSON then writes separately", function.fingerprint.name),
                            evidence: vec![
                                format!("{}.Marshal at line {}, Write at line {}", alias, bl.line, next.line),
                                "json.NewEncoder(w).Encode(v) streams directly, saving one allocation".into(),
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

// D3
fn binary_read_single_field(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "encoding/binary") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.Read(")) && bl.text.contains('&') {
                findings.push(Finding {
                    rule_id: "binary_read_for_single_field".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses binary.Read for a single scalar field",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Read(..., &value) at line {}", alias, bl.line),
                        "binary.ByteOrder.Uint32/Uint64 avoids reflection for scalar reads".into(),
                    ],
                });
            }
        }
    }
    findings
}

// D4
fn json_number_without_use_number(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let decodes_to_dynamic_map = lines.iter().any(|line| {
        line.text.contains("map[string]any") || line.text.contains("map[string]interface{}")
    });
    if !decodes_to_dynamic_map || lines.iter().any(|line| line.text.contains("UseNumber()")) {
        return findings;
    }
    for alias in import_aliases_for(file, "encoding/json") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.Unmarshal(")) || bl.text.contains(".Decode(&") {
                findings.push(Finding {
                    rule_id: "json_number_vs_float64_decode".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} decodes JSON numbers into map[string]any without UseNumber",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("dynamic JSON decode at line {}", bl.line),
                        "decoder.UseNumber() preserves large integer precision instead of coercing to float64"
                            .into(),
                    ],
                });
                break;
            }
        }
    }
    findings
}

// D5
fn xml_decoder_trusted_strict(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let name_lower = function.fingerprint.name.to_lowercase();
    let trusted_context = name_lower.contains("trusted")
        || name_lower.contains("internal")
        || name_lower.contains("feed");
    if !trusted_context {
        return findings;
    }
    for alias in import_aliases_for(file, "encoding/xml") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.NewDecoder("))
                && !lines
                    .iter()
                    .any(|line| line.text.contains(".Strict = false"))
            {
                findings.push(Finding {
                    rule_id: "xml_decoder_without_strict".into(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} parses trusted XML without relaxing Strict mode",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("trusted XML decoder at line {}", bl.line),
                        "for trusted feeds, decoder.Strict = false can avoid extra validation work"
                            .into(),
                    ],
                });
            }
        }
    }
    findings
}

// D6
fn csv_reader_reuse(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "encoding/csv") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.NewReader(")) {
                let has_reuse = lines
                    .iter()
                    .any(|l| l.text.contains("ReuseRecord") && l.text.contains("true"));
                if !has_reuse {
                    findings.push(Finding {
                        rule_id: "csv_reader_reuse_record".into(),
                        severity: Severity::Info,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} creates csv.Reader without ReuseRecord",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!("{}.NewReader at line {}", alias, bl.line),
                            "ReuseRecord = true reduces allocations from N to 1".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

// D7
fn scanner_small_buffer(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let name_lower = function.fingerprint.name.to_lowercase();
    let large_line_context =
        name_lower.contains("line") || name_lower.contains("log") || name_lower.contains("scan");
    if !large_line_context {
        return findings;
    }
    for alias in import_aliases_for(file, "bufio") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.NewScanner("))
                && !lines.iter().any(|line| line.text.contains(".Buffer("))
            {
                findings.push(Finding {
                    rule_id: "bufio_scanner_small_buffer_for_large_lines".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses bufio.Scanner without raising the token buffer",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("bufio.NewScanner at line {}", bl.line),
                        "scanner.Buffer(...) is required when large lines can exceed the 64KB default"
                            .into(),
                    ],
                });
            }
        }
    }
    findings
}

// D8
fn http_body_readall_no_limit(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    if !is_request_path_function(file, function) {
        return findings;
    }
    for alias in import_aliases_for(file, "io") {
        for bl in lines {
            if bl.text.contains(&format!("{alias}.ReadAll("))
                && (bl.text.contains("req.Body")
                    || bl.text.contains("r.Body")
                    || bl.text.contains("request.Body"))
            {
                let has_limit = lines
                    .iter()
                    .any(|l| l.text.contains("LimitReader") || l.text.contains("MaxBytesReader"));
                if !has_limit {
                    findings.push(Finding {
                        rule_id: "http_body_readall_without_limitreader".into(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: bl.line,
                        end_line: bl.line,
                        message: format!(
                            "function {} reads HTTP body without size limit",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "{}.ReadAll(req.Body) at line {} without LimitReader",
                                alias, bl.line
                            ),
                            "unbounded body read is a DoS vector".into(),
                        ],
                    });
                }
            }
        }
    }
    findings
}

// ── Section E — Error Handling And Interface Patterns ──

fn error_interface_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(type_assertion_no_comma(file, function, lines));
    findings.extend(type_switch_repeated(file, function, lines));
    findings.extend(errors_new_hot_path(file, function, lines));
    findings.extend(errorf_no_wrap(file, function, lines));
    findings.extend(error_string_compare(file, function, lines));
    findings.extend(empty_interface_parameter_overuse(file, function, lines));
    findings.extend(panic_for_expected(file, function, lines));
    findings
}

// E1
fn type_assertion_no_comma(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains(".(")
            && !bl.text.contains(", ok")
            && !bl.text.contains(",ok")
            && !bl.text.contains("switch")
            && !bl.text.contains(".(type)")
        {
            let has_assign = bl.text.contains(":=") || bl.text.contains(" = ");
            if has_assign && bl.text.contains(")") {
                findings.push(Finding {
                    rule_id: "type_assertion_without_comma_ok".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses type assertion without comma-ok",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("unchecked type assertion at line {}", bl.line),
                        "v, ok := i.(T) prevents runtime panics".into(),
                    ],
                });
            }
        }
    }
    findings
}

// E2
fn type_switch_repeated(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut assertion_lines: Vec<usize> = Vec::new();
    for bl in lines {
        if bl.text.contains("if _, ok :=") && bl.text.contains(".(") {
            assertion_lines.push(bl.line);
        }
    }
    if assertion_lines.len() >= 3 {
        findings.push(Finding {
            rule_id: "type_switch_vs_repeated_assertions".into(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: assertion_lines[0],
            end_line: assertion_lines[assertion_lines.len() - 1],
            message: format!(
                "function {} uses sequential type assertions instead of type switch",
                function.fingerprint.name
            ),
            evidence: vec![
                format!(
                    "{} sequential type assertions at lines {}",
                    assertion_lines.len(),
                    join_lines(&assertion_lines)
                ),
                "switch v := i.(type) compiles to a single dispatch".into(),
            ],
        });
    }
    findings
}

// E3
fn errors_new_hot_path(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "errors") {
        let mut error_new_lines: Vec<usize> = Vec::new();
        for bl in lines {
            if bl.text.contains(&format!("{alias}.New(\"")) {
                error_new_lines.push(bl.line);
            }
        }
        if error_new_lines.len() >= 2 {
            findings.push(Finding {
                rule_id: "errors_new_for_static_sentinel".into(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: error_new_lines[0],
                end_line: error_new_lines[error_new_lines.len() - 1],
                message: format!(
                    "function {} calls errors.New multiple times with static strings",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "{}.New called at lines {}",
                        alias,
                        join_lines(&error_new_lines)
                    ),
                    "package-level sentinel errors avoid repeated allocations".into(),
                ],
            });
        }
    }
    findings
}

// E4
fn errorf_no_wrap(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for alias in import_aliases_for(file, "fmt") {
        for bl in lines {
            let pat = format!("{alias}.Errorf(");
            if bl.text.contains(&pat)
                && bl.text.contains("%v")
                && bl.text.contains("err")
                && !bl.text.contains("%w")
            {
                findings.push(Finding {
                    rule_id: "fmt_errorf_without_wrap_verb".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses %v instead of %w for error wrapping",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.Errorf with %v at line {}", alias, bl.line),
                        "%w wraps the error preserving errors.Is/As chain".into(),
                    ],
                });
            }
        }
    }
    findings
}

// E5
fn error_string_compare(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.contains(".Error()") && (bl.text.contains("== \"") || bl.text.contains("!= \""))
        {
            findings.push(Finding {
                rule_id: "error_string_comparison".into(),
                severity: Severity::Warning,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: bl.line,
                end_line: bl.line,
                message: format!(
                    "function {} compares errors by string value",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("err.Error() == \"...\" at line {}", bl.line),
                    "errors.Is(err, sentinel) is faster and semantically correct".into(),
                ],
            });
        }
    }
    findings
}

// E6
fn empty_interface_parameter_overuse(
    file: &ParsedFile,
    function: &ParsedFunction,
    _lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let exported = function
        .fingerprint
        .name
        .chars()
        .next()
        .is_some_and(char::is_uppercase);
    if exported
        && (function.fingerprint.contains_any_type || function.fingerprint.contains_empty_interface)
        && (function.signature_text.contains(" any")
            || function.signature_text.contains(" interface{}")
            || function.signature_text.contains("(any")
            || function.signature_text.contains("(interface{}"))
    {
        findings.push(Finding {
            rule_id: "empty_interface_parameter_overuse".into(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: function.fingerprint.start_line,
            end_line: function.fingerprint.start_line,
            message: format!(
                "exported function {} overuses any/interface{{}} in its signature",
                function.fingerprint.name
            ),
            evidence: vec![
                format!("signature at line {} accepts any/interface{{}}", function.fingerprint.start_line),
                "concrete types or generics avoid heap escapes and make the API easier to reason about"
                    .into(),
            ],
        });
    }
    findings
}

// E7
fn panic_for_expected(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bl in lines {
        if bl.text.starts_with("panic(") || bl.text.starts_with("panic (") {
            let msg = bl.text.to_lowercase();
            if msg.contains("invalid")
                || msg.contains("missing")
                || msg.contains("not found")
                || msg.contains("unsupported")
                || msg.contains("unexpected")
            {
                findings.push(Finding {
                    rule_id: "panic_for_expected_errors".into(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: bl.line,
                    end_line: bl.line,
                    message: format!(
                        "function {} uses panic for expected error conditions",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("panic with expected-error message at line {}", bl.line),
                        "returning an error is ~200× cheaper and doesn't crash the process".into(),
                    ],
                });
            }
        }
    }
    findings
}
