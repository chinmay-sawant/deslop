#[path = "support/mod.rs"]
mod support;

use std::fs;

use deslop::{ExportOptions, ScanOptions, scan_repository};
use support::FixtureWorkspace;

const EXPORT_HANDLE_FMT_POSITIVE: &str =
    "go/rules_fixtures/export_context_handle_fmt/export_context_handle_fmt_positive.txt";

#[test]
fn export_writes_context_and_chunk_files() {
    let workspace = FixtureWorkspace::new();
    workspace.write_fixture_file(EXPORT_HANDLE_FMT_POSITIVE, "main.go");

    let output = scan_repository(&ScanOptions {
        root: workspace.root().to_path_buf(),
        respect_ignore: false,
    })
    .expect("scan should succeed");

    let context_dir = workspace.root().join("context");
    let chunks_dir = workspace.root().join("chunks");
    let summary = output
        .export_context(
            &output.report,
            &ExportOptions {
                export_context: true,
                export_chunks: true,
                chunk_size: 25,
                context_output_dir: context_dir.clone(),
                chunks_output_dir: chunks_dir.clone(),
                details: false,
            },
        )
        .expect("export should succeed");

    assert!(summary.context_files_written > 0);
    assert_eq!(summary.chunk_files_written, 1);
    assert!(context_dir.join("1.txt").is_file());
    let chunk_name = fs::read_dir(&chunks_dir)
        .expect("chunks dir should exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .find(|name| name.starts_with("Chunk_"))
        .expect("chunk file should exist");
    assert!(chunk_name.ends_with(".txt"));

    let first_block =
        fs::read_to_string(context_dir.join("1.txt")).expect("context file should exist");
    assert!(first_block.contains("Finding 1/"));
    assert!(first_block.contains("Source:"));
    assert!(first_block.contains("Rule description:"));
    assert!(first_block.contains("Auto triage note:"));
    assert!(first_block.contains("Function:"));
    assert!(first_block.contains("func Handle()"));
}
