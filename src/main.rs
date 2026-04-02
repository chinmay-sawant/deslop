use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

mod cli;

use std::path::PathBuf;

use crate::cli::{
    filtered_rules, format_rules_report, format_rules_report_json, format_scan_report,
    format_scan_report_json, print_benchmark_report,
};
use deslop::{
    BenchmarkOptions, RuleLanguage, RuleStatus, ScanOptions, benchmark_repository, scan_repository,
};

const GO_SEMANTIC_ENV_VAR: &str = "DESLOP_ENABLE_GO_SEMANTIC";

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Scan Go, Python, and Rust repositories for likely AI slop patterns"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Scan {
        path: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        details: bool,
        #[arg(long)]
        no_ignore: bool,
        #[arg(long)]
        enable_semantic: bool,
        #[arg(long, value_delimiter = ',')]
        ignore: Vec<String>,
        /// Exit 0 even when findings are present (useful for informational runs).
        #[arg(long)]
        no_fail: bool,
    },
    Bench {
        path: PathBuf,
        #[arg(long, default_value_t = 5)]
        repeats: usize,
        #[arg(long, default_value_t = 1)]
        warmups: usize,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        no_ignore: bool,
        #[arg(long)]
        enable_semantic: bool,
    },
    Rules {
        #[arg(long)]
        json: bool,
        #[arg(long, value_enum)]
        language: Option<RuleLanguageArg>,
        #[arg(long, value_enum)]
        status: Option<RuleStatusArg>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Scan {
            path,
            json,
            details,
            no_ignore,
            enable_semantic,
            ignore,
            no_fail,
        } => {
            set_go_semantic_env(enable_semantic);
            let scan_root = path.clone();
            let mut report = scan_repository(&ScanOptions {
                root: path,
                respect_ignore: !no_ignore,
            })
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
                    .filter(|f| details || f.rule_id != "full_dataset_load")
                    .count();
                if finding_count > 0 {
                    std::process::exit(1);
                }
            }
        }
        Command::Bench {
            path,
            repeats,
            warmups,
            json,
            no_ignore,
            enable_semantic,
        } => {
            set_go_semantic_env(enable_semantic);
            let bench_root = path.clone();
            let report = benchmark_repository(&BenchmarkOptions {
                root: path,
                repeats,
                warmups,
                respect_ignore: !no_ignore,
            })
            .with_context(|| format!("benchmark failed for {}", bench_root.display()))?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_benchmark_report(&report);
            }
        }
        Command::Rules {
            json,
            language,
            status,
        } => {
            let language = language.map(Into::into);
            let status = status.map(Into::into);
            let rules = filtered_rules(language.clone(), status.clone());

            if json {
                println!("{}", format_rules_report_json(&rules)?);
            } else {
                print!("{}", format_rules_report(&rules, language, status));
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, ValueEnum)]
enum RuleLanguageArg {
    Common,
    Go,
    Python,
    Rust,
}

impl From<RuleLanguageArg> for RuleLanguage {
    fn from(value: RuleLanguageArg) -> Self {
        match value {
            RuleLanguageArg::Common => RuleLanguage::Common,
            RuleLanguageArg::Go => RuleLanguage::Go,
            RuleLanguageArg::Python => RuleLanguage::Python,
            RuleLanguageArg::Rust => RuleLanguage::Rust,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum RuleStatusArg {
    Stable,
    Experimental,
    Research,
}

impl From<RuleStatusArg> for RuleStatus {
    fn from(value: RuleStatusArg) -> Self {
        match value {
            RuleStatusArg::Stable => RuleStatus::Stable,
            RuleStatusArg::Experimental => RuleStatus::Experimental,
            RuleStatusArg::Research => RuleStatus::Research,
        }
    }
}

fn set_go_semantic_env(enable_semantic: bool) {
    if enable_semantic {
        // SAFETY: This CLI sets the process environment before starting repository work,
        // and the variable is used as a simple opt-in flag for child logic in the same process.
        unsafe {
            std::env::set_var(GO_SEMANTIC_ENV_VAR, "1");
        }
    }
}
