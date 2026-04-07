use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

#[path = "hot_path/repeated_work.rs"]
mod repeated_work;

use super::{
    binding_matches, body_lines, first_line_with_any, has_prior_loop_line, import_aliases_for,
    is_request_path_function, repeated_parse_findings,
};
use repeated_work::core_repeated_work_findings;

pub(crate) fn core_hot_path_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines = body_lines(function);

    findings.extend(repeated_parse_findings(file, function));
    findings.extend(core_repeated_work_findings(file, function, &lines));

    for alias in import_aliases_for(file, "regexp") {
        for body_line in &lines {
            if body_line.in_loop
                && (body_line.text.contains(&format!("{alias}.Compile("))
                    || body_line.text.contains(&format!("{alias}.MustCompile(")))
            {
                findings.push(Finding {
                    rule_id: "regexp_compile_in_hot_path".to_string(),
                    severity: Severity::Warning,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} compiles regular expressions inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!(
                            "{}.Compile(...) or {}.MustCompile(...) observed inside a loop at line {}",
                            alias, alias, body_line.line
                        ),
                        "reusing a compiled regexp is usually cheaper than compiling per iteration"
                            .to_string(),
                    ],
                });
            }
        }
    }

    if is_request_path_function(file, function) {
        let mut template_aliases = import_aliases_for(file, "html/template");
        template_aliases.extend(import_aliases_for(file, "text/template"));
        for alias in template_aliases {
            for body_line in &lines {
                if body_line.text.contains(&format!("{alias}.ParseFiles("))
                    || body_line.text.contains(&format!("{alias}.ParseGlob("))
                    || body_line.text.contains(&format!("{alias}.ParseFS("))
                {
                    findings.push(Finding {
                        rule_id: "template_parse_in_hot_path".to_string(),
                        severity: Severity::Warning,
                        path: file.path.clone(),
                        function_name: Some(function.fingerprint.name.clone()),
                        start_line: body_line.line,
                        end_line: body_line.line,
                        message: format!(
                            "function {} parses templates on a request path",
                            function.fingerprint.name
                        ),
                        evidence: vec![
                            format!(
                                "{}.Parse* call observed at line {} in a handler-like function",
                                alias, body_line.line
                            ),
                            "template parsing is usually better cached during startup than repeated on request paths"
                                .to_string(),
                        ],
                    });
                }
            }
        }
    }

    for alias in import_aliases_for(file, "encoding/json") {
        for body_line in &lines {
            if body_line.in_loop && body_line.text.contains(&format!("{alias}.NewEncoder(")) {
                findings.push(Finding {
                    rule_id: "json_encoder_recreated_per_item".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} recreates a JSON encoder inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.NewEncoder(...) observed inside a loop at line {}", alias, body_line.line),
                        "reusing a stable encoder or stream writer usually avoids repeated setup work"
                            .to_string(),
                    ],
                });
            }

            if body_line.in_loop && body_line.text.contains(&format!("{alias}.NewDecoder(")) {
                findings.push(Finding {
                    rule_id: "json_decoder_recreated_per_item".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: body_line.line,
                    end_line: body_line.line,
                    message: format!(
                        "function {} recreates a JSON decoder inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("{}.NewDecoder(...) observed inside a loop at line {}", alias, body_line.line),
                        "reusing a decoder or restructuring the loop often avoids repeated decode setup work"
                            .to_string(),
                    ],
                });
            }
        }
    }

    let mut gzip_markers = import_aliases_for(file, "compress/gzip")
        .into_iter()
        .flat_map(|alias| {
            vec![
                format!("{alias}.NewWriter("),
                format!("{alias}.NewWriterLevel("),
                format!("{alias}.NewReader("),
            ]
        })
        .collect::<Vec<_>>();
    gzip_markers.extend([
        "gzip.NewWriter(".to_string(),
        "gzip.NewWriterLevel(".to_string(),
        "gzip.NewReader(".to_string(),
    ]);

    for body_line in &lines {
        if body_line.in_loop
            && gzip_markers
                .iter()
                .any(|marker| body_line.text.contains(marker))
        {
            findings.push(Finding {
                rule_id: "gzip_reader_writer_recreated_per_item".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: body_line.line,
                end_line: body_line.line,
                message: format!(
                    "function {} recreates gzip readers or writers inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!(
                        "gzip constructor observed inside a loop at line {}",
                        body_line.line
                    ),
                    "reusing compression state per stream is usually cheaper than rebuilding it per item"
                        .to_string(),
                ],
            });
        }
    }

    if !findings
        .iter()
        .any(|finding| finding.rule_id == "gzip_reader_writer_recreated_per_item")
    {
        let fallback_line = first_line_with_any(
            function,
            &["gzip.NewWriter(", "gzip.NewWriterLevel(", "gzip.NewReader("],
        );
        if let Some(fallback_line) = fallback_line
            && has_prior_loop_line(function, fallback_line)
        {
            findings.push(Finding {
                rule_id: "gzip_reader_writer_recreated_per_item".to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: fallback_line,
                end_line: fallback_line,
                message: format!(
                    "function {} recreates gzip readers or writers inside a loop",
                    function.fingerprint.name
                ),
                evidence: vec![
                    format!("gzip constructor observed in loop-bearing function at line {fallback_line}"),
                    "reusing compression state per stream is usually cheaper than rebuilding it per item"
                        .to_string(),
                ],
            });
        }
    }

    let csv_aliases = import_aliases_for(file, "encoding/csv");
    if !csv_aliases.is_empty() {
        let writer_patterns = csv_aliases
            .iter()
            .map(|alias| format!("{alias}.NewWriter("))
            .collect::<Vec<_>>();
        let writer_pattern_refs = writer_patterns
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        for (name, line, _) in binding_matches(&lines, &writer_pattern_refs) {
            let flush_line = lines
                .iter()
                .find(|body_line| {
                    body_line.in_loop && body_line.text.contains(&format!("{name}.Flush()"))
                })
                .map(|body_line| body_line.line);

            if let Some(flush_line) = flush_line {
                findings.push(Finding {
                    rule_id: "csv_writer_flush_per_row".to_string(),
                    severity: Severity::Info,
                    path: file.path.clone(),
                    function_name: Some(function.fingerprint.name.clone()),
                    start_line: flush_line,
                    end_line: flush_line,
                    message: format!(
                        "function {} flushes a csv.Writer inside a loop",
                        function.fingerprint.name
                    ),
                    evidence: vec![
                        format!("csv writer {name} created at line {line}"),
                        format!(
                            "{}.Flush() observed inside a loop at line {}",
                            name, flush_line
                        ),
                        "flushing once per row usually reduces buffering effectiveness".to_string(),
                    ],
                });
            }
        }
    }

    findings
}
