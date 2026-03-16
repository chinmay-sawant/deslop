use anyhow::Result;
use clap::{Parser, Subcommand};

mod cli;

use std::path::PathBuf;

use crate::cli::{format_scan_report, format_scan_report_json, print_benchmark_report};
use goslop::{benchmark_repository, scan_repository, BenchmarkOptions, ScanOptions};

#[derive(Debug, Parser)]
#[command(author, version, about = "Scan Go repositories for likely AI slop patterns")]
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
        } => {
            let report = scan_repository(&ScanOptions {
                root: path,
                respect_ignore: !no_ignore,
            })?;

            if json {
                println!("{}", format_scan_report_json(&report, details)?);
            } else {
                print!("{}", format_scan_report(&report, details));
            }
        }
        Command::Bench {
            path,
            repeats,
            warmups,
            json,
            no_ignore,
        } => {
            let report = benchmark_repository(&BenchmarkOptions {
                root: path,
                repeats,
                warmups,
                respect_ignore: !no_ignore,
            })?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_benchmark_report(&report);
            }
        }
    }

    Ok(())
}
