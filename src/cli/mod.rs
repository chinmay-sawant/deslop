mod report;
mod rules;

pub(crate) use report::{format_scan_report, format_scan_report_json, print_benchmark_report};
pub(crate) use rules::{filtered_rules, format_rules_report, format_rules_report_json};

use anyhow::{Context, Result};
use deslop::{
    BenchmarkOptions, RuleLanguage, RuleStatus, ScanOptions, benchmark_repository_with_go_semantic,
    is_detail_only_rule, scan_repository_with_go_semantic,
};
use std::path::PathBuf;

pub(crate) fn execute_scan(
    path: PathBuf,
    json: bool,
    details: bool,
    no_ignore: bool,
    enable_semantic: bool,
    ignore: Vec<String>,
    no_fail: bool,
) -> Result<()> {
    let scan_root = path.clone();
    let mut report = scan_repository_with_go_semantic(
        &ScanOptions {
            root: path,
            respect_ignore: !no_ignore,
        },
        enable_semantic,
    )
    .with_context(|| format!("scan failed for {}", scan_root.display()))?;

    if !ignore.is_empty() {
        report
            .findings
            .retain(|finding| !ignore.iter().any(|rule_id| rule_id == &finding.rule_id));
    }

    if json {
        println!("{}", format_scan_report_json(&report, details)?);
    } else {
        print!("{}", format_scan_report(&report, details));
    }

    if !no_fail {
        let finding_count = report
            .findings
            .iter()
            .filter(|f| details || !is_detail_only_rule(f.rule_id.as_str()))
            .count();
        if finding_count > 0 {
            std::process::exit(1);
        }
    }
    Ok(())
}

pub(crate) fn execute_bench(
    path: PathBuf,
    repeats: usize,
    warmups: usize,
    json: bool,
    no_ignore: bool,
    enable_semantic: bool,
) -> Result<()> {
    let bench_root = path.clone();
    let report = benchmark_repository_with_go_semantic(
        &BenchmarkOptions {
            root: path,
            repeats,
            warmups,
            respect_ignore: !no_ignore,
        },
        enable_semantic,
    )
    .with_context(|| format!("benchmark failed for {}", bench_root.display()))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_benchmark_report(&report);
    }
    Ok(())
}

pub(crate) fn execute_rules(
    json: bool,
    language: Option<RuleLanguage>,
    status: Option<RuleStatus>,
) -> Result<()> {
    let rules = filtered_rules(language.clone(), status.clone());

    if json {
        println!("{}", format_rules_report_json(&rules)?);
    } else {
        print!("{}", format_rules_report(&rules, language, status));
    }
    Ok(())
}
