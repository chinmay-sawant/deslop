use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

mod cli;

use std::path::PathBuf;

use crate::cli::{execute_bench, execute_rules, execute_scan};
use deslop::{RuleLanguage, RuleStatus};

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
        } => execute_scan(
            path,
            json,
            details,
            no_ignore,
            enable_semantic,
            ignore,
            no_fail,
        ),
        Command::Bench {
            path,
            repeats,
            warmups,
            json,
            no_ignore,
            enable_semantic,
        } => execute_bench(path, repeats, warmups, json, no_ignore, enable_semantic),
        Command::Rules {
            json,
            language,
            status,
        } => execute_rules(
            json,
            language.map(Into::into),
            status.map(Into::into),
        ),
    }
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
