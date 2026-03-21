use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

pub(crate) fn format_scan_report(report: &deslop::ScanReport, details: bool) -> String {
    let mut output = String::new();
    let findings = visible_findings(report, details);

    writeln!(&mut output, "deslop scan root: {}", report.root.display()).expect("write to string");
    writeln!(
        &mut output,
        "Go files discovered: {}",
        report.files_discovered
    )
    .expect("write to string");
    writeln!(&mut output, "Go files analyzed: {}", report.files_analyzed).expect("write to string");
    writeln!(
        &mut output,
        "Functions fingerprinted: {}",
        report.functions_found
    )
    .expect("write to string");
    writeln!(&mut output, "Findings: {}", findings.len()).expect("write to string");
    writeln!(
        &mut output,
        "Index summary: packages={} symbols={} imports={}",
        report.index_summary.package_count,
        report.index_summary.symbol_count,
        report.index_summary.import_count
    )
    .expect("write to string");
    writeln!(
        &mut output,
        "Parse failures: {}",
        report.parse_failures.len()
    )
    .expect("write to string");
    writeln!(
        &mut output,
        "Timings: discover={}ms parse={}ms index={}ms heuristics={}ms total={}ms",
        report.timings.discover_ms,
        report.timings.parse_ms,
        report.timings.index_ms,
        report.timings.heuristics_ms,
        report.timings.total_ms
    )
    .expect("write to string");

    if details {
        for file in &report.files {
            writeln!(&mut output).expect("write to string");
            writeln!(&mut output, "{}", file.path.display()).expect("write to string");
            writeln!(
                &mut output,
                "  package={} syntax_error={} functions={}",
                file.package_name.as_deref().unwrap_or("<unknown>"),
                file.syntax_error,
                file.functions.len()
            )
            .expect("write to string");

            for function in &file.functions {
                writeln!(
                    &mut output,
                    "  - {} [{}:{}] complexity={} comment_ratio={:.2} symmetry={:.2} any={} iface={} calls={}",
                    function.name,
                    function.start_line,
                    function.end_line,
                    function.complexity_score,
                    function.comment_to_code_ratio,
                    function.symmetry_score,
                    function.contains_any_type,
                    function.contains_empty_interface,
                    function.call_count
                )
                .expect("write to string");
            }
        }
    }

    if !findings.is_empty() {
        writeln!(&mut output).expect("write to string");
        writeln!(&mut output, "Findings:").expect("write to string");
        for finding in findings {
            writeln!(
                &mut output,
                "  - {}:{} {} [{}]",
                finding.path.display(),
                finding.start_line,
                finding.message,
                finding.rule_id
            )
            .expect("write to string");
        }
    }

    if !report.parse_failures.is_empty() {
        writeln!(&mut output).expect("write to string");
        writeln!(&mut output, "Parse failures:").expect("write to string");
        for failure in &report.parse_failures {
            writeln!(
                &mut output,
                "  - {}: {}",
                failure.path.display(),
                failure.message
            )
            .expect("write to string");
        }
    }

    output
}

pub(crate) fn format_scan_report_json(
    report: &deslop::ScanReport,
    details: bool,
) -> Result<String> {
    if details {
        Ok(serde_json::to_string_pretty(report)?)
    } else {
        Ok(serde_json::to_string_pretty(&ScanReportSummary::from(
            report,
        ))?)
    }
}

fn visible_findings<'a>(report: &'a deslop::ScanReport, details: bool) -> Vec<&'a deslop::Finding> {
    report
        .findings
        .iter()
        .filter(|finding| details || !is_detail_only_finding(finding.rule_id.as_str()))
        .collect()
}

fn is_detail_only_finding(rule_id: &str) -> bool {
    matches!(rule_id, "full_dataset_load")
}

pub(crate) fn print_benchmark_report(report: &deslop::BenchmarkReport) {
    println!("deslop bench root: {}", report.root.display());
    println!(
        "Warmups={} Repeats={} Files={} Functions={} Findings={}",
        report.warmups,
        report.repeats,
        report.files_analyzed,
        report.functions_found,
        report.findings_found
    );
    println!(
        "Total ms: min={} max={} mean={:.2} median={:.2}",
        report.total.min_ms, report.total.max_ms, report.total.mean_ms, report.total.median_ms
    );
    println!(
        "Parse ms: min={} max={} mean={:.2} median={:.2}",
        report.parse.min_ms, report.parse.max_ms, report.parse.mean_ms, report.parse.median_ms
    );
    println!(
        "Index ms: min={} max={} mean={:.2} median={:.2}",
        report.index.min_ms, report.index.max_ms, report.index.mean_ms, report.index.median_ms
    );
    println!(
        "Heuristics ms: min={} max={} mean={:.2} median={:.2}",
        report.heuristics.min_ms,
        report.heuristics.max_ms,
        report.heuristics.mean_ms,
        report.heuristics.median_ms
    );
}

#[derive(Debug, Serialize)]
struct ScanReportSummary<'a> {
    root: &'a PathBuf,
    files_discovered: usize,
    files_analyzed: usize,
    functions_found: usize,
    files: Vec<FileReportSummary<'a>>,
    findings: Vec<&'a deslop::Finding>,
    index_summary: &'a deslop::IndexSummary,
    parse_failures: &'a [deslop::ParseFailure],
    timings: &'a deslop::TimingBreakdown,
}

impl<'a> From<&'a deslop::ScanReport> for ScanReportSummary<'a> {
    fn from(report: &'a deslop::ScanReport) -> Self {
        Self {
            root: &report.root,
            files_discovered: report.files_discovered,
            files_analyzed: report.files_analyzed,
            functions_found: report.functions_found,
            files: report.files.iter().map(FileReportSummary::from).collect(),
            findings: visible_findings(report, false),
            index_summary: &report.index_summary,
            parse_failures: &report.parse_failures,
            timings: &report.timings,
        }
    }
}

#[derive(Debug, Serialize)]
struct FileReportSummary<'a> {
    path: &'a PathBuf,
    package_name: Option<&'a str>,
    syntax_error: bool,
    function_count: usize,
    functions: Vec<FunctionSummary<'a>>,
}

impl<'a> From<&'a deslop::FileReport> for FileReportSummary<'a> {
    fn from(file: &'a deslop::FileReport) -> Self {
        Self {
            path: &file.path,
            package_name: file.package_name.as_deref(),
            syntax_error: file.syntax_error,
            function_count: file.functions.len(),
            functions: file.functions.iter().map(FunctionSummary::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct FunctionSummary<'a> {
    name: &'a str,
    kind: &'a str,
    receiver_type: Option<&'a str>,
    start_line: usize,
    end_line: usize,
}

impl<'a> From<&'a deslop::FunctionFingerprint> for FunctionSummary<'a> {
    fn from(function: &'a deslop::FunctionFingerprint) -> Self {
        Self {
            name: &function.name,
            kind: &function.kind,
            receiver_type: function.receiver_type.as_deref(),
            start_line: function.start_line,
            end_line: function.end_line,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{format_scan_report, format_scan_report_json};

    fn sample_report() -> deslop::ScanReport {
        deslop::ScanReport {
            root: PathBuf::from("/tmp/sample"),
            files_discovered: 1,
            files_analyzed: 1,
            functions_found: 1,
            files: vec![deslop::FileReport {
                path: PathBuf::from("/tmp/sample/main.go"),
                package_name: Some("main".to_string()),
                syntax_error: false,
                byte_size: 128,
                functions: vec![deslop::FunctionFingerprint {
                    name: "Run".to_string(),
                    kind: "function".to_string(),
                    receiver_type: None,
                    start_line: 10,
                    end_line: 24,
                    line_count: 15,
                    comment_lines: 2,
                    code_lines: 13,
                    comment_to_code_ratio: 0.15,
                    complexity_score: 4,
                    symmetry_score: 0.25,
                    boilerplate_err_guards: 1,
                    contains_any_type: false,
                    contains_empty_interface: false,
                    type_assertion_count: 0,
                    call_count: 7,
                }],
            }],
            findings: vec![
                deslop::Finding {
                    rule_id: "full_dataset_load".to_string(),
                    severity: deslop::Severity::Info,
                    path: PathBuf::from("/tmp/sample/main.go"),
                    function_name: Some("Run".to_string()),
                    start_line: 12,
                    end_line: 12,
                    message: "function Run loads an entire payload into memory".to_string(),
                    evidence: Vec::new(),
                },
                deslop::Finding {
                    rule_id: "placeholder_test_body".to_string(),
                    severity: deslop::Severity::Info,
                    path: PathBuf::from("/tmp/sample/main_test.go"),
                    function_name: Some("TestRun".to_string()),
                    start_line: 20,
                    end_line: 20,
                    message: "test TestRun looks like a placeholder rather than a validating test"
                        .to_string(),
                    evidence: Vec::new(),
                },
            ],
            index_summary: deslop::IndexSummary {
                package_count: 1,
                symbol_count: 2,
                import_count: 1,
            },
            parse_failures: Vec::new(),
            timings: deslop::TimingBreakdown {
                discover_ms: 1,
                parse_ms: 2,
                index_ms: 3,
                heuristics_ms: 4,
                total_ms: 10,
            },
        }
    }

    #[test]
    fn default_text_output_shows_findings_without_function_listing() {
        let output = format_scan_report(&sample_report(), false);

        assert!(output.contains("Findings: 1"));
        assert!(!output.contains("full_dataset_load"));
        assert!(output.contains("placeholder_test_body"));
        assert!(!output.contains("package=main syntax_error=false functions=1"));
        assert!(!output.contains("  - Run [10:24]"));
        assert!(!output.contains("complexity="));
        assert!(!output.contains("calls="));
    }

    #[test]
    fn detailed_text_output_keeps_per_file_function_listing() {
        let output = format_scan_report(&sample_report(), true);

        assert!(output.contains("Findings: 2"));
        assert!(output.contains("full_dataset_load"));
        assert!(output.contains("package=main syntax_error=false functions=1"));
        assert!(output.contains("  - Run [10:24] complexity=4"));
    }

    #[test]
    fn default_json_output_omits_fingerprint_metrics() {
        let output = format_scan_report_json(&sample_report(), false).expect("json should render");

        assert!(output.contains("\"name\": \"Run\""));
        assert!(output.contains("\"function_count\": 1"));
        assert!(!output.contains("full_dataset_load"));
        assert!(output.contains("placeholder_test_body"));
        assert!(!output.contains("complexity_score"));
        assert!(!output.contains("call_count"));
    }

    #[test]
    fn detailed_json_output_keeps_full_fingerprint_metrics() {
        let output = format_scan_report_json(&sample_report(), true).expect("json should render");

        assert!(output.contains("full_dataset_load"));
        assert!(output.contains("complexity_score"));
        assert!(output.contains("call_count"));
    }
}
