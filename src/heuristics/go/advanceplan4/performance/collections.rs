use super::*;

pub(super) fn slice_map_findings(
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
