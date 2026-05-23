use std::fs;
use std::path::{Path, PathBuf};

use crate::analysis::ParsedFile;
use crate::model::ScanReport;
use crate::{Error, Result};

use super::block::{FindingBlock, block_separator, build_finding_block, visible_findings};
use super::chunk::{build_chunk_content, chunk_output_path};

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub export_context: bool,
    pub export_chunks: bool,
    pub chunk_size: usize,
    pub context_output_dir: PathBuf,
    pub chunks_output_dir: PathBuf,
    pub details: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExportSummary {
    pub context_files_written: usize,
    pub chunk_files_written: usize,
}

pub fn export_finding_context(
    report: &ScanReport,
    parsed_files: &[ParsedFile],
    options: &ExportOptions,
) -> Result<ExportSummary> {
    if !options.export_context && !options.export_chunks {
        return Ok(ExportSummary::default());
    }

    let findings = visible_findings(report, options.details);
    if findings.is_empty() {
        return Ok(ExportSummary::default());
    }

    let mut cached_lines = std::collections::HashMap::new();
    let blocks: Vec<FindingBlock> = findings
        .iter()
        .enumerate()
        .map(|(index, finding)| {
            build_finding_block(
                finding,
                report,
                parsed_files,
                &mut cached_lines,
                index + 1,
                findings.len(),
                options.details,
            )
        })
        .collect();

    let mut summary = ExportSummary::default();

    if options.export_context {
        summary.context_files_written = write_context_files(&blocks, &options.context_output_dir)?;
    }

    if options.export_chunks {
        summary.chunk_files_written = write_chunk_files(
            &blocks,
            &options.chunks_output_dir,
            options.chunk_size.max(1),
        )?;
    }

    Ok(summary)
}

fn write_context_files(blocks: &[FindingBlock], output_dir: &Path) -> Result<usize> {
    fs::create_dir_all(output_dir).map_err(|error| Error::io(output_dir, error))?;
    clean_txt_files(output_dir)?;

    for (index, block) in blocks.iter().enumerate() {
        let output_path = output_dir.join(format!("{}.txt", index + 1));
        fs::write(&output_path, &block.text).map_err(|error| Error::io(&output_path, error))?;
    }

    Ok(blocks.len())
}

fn write_chunk_files(
    blocks: &[FindingBlock],
    output_dir: &Path,
    chunk_size: usize,
) -> Result<usize> {
    fs::create_dir_all(output_dir).map_err(|error| Error::io(output_dir, error))?;
    clean_chunk_files(output_dir)?;

    let separator = block_separator();
    let total = blocks.len();
    let mut chunk_count = 0;

    for (chunk_idx, chunk) in blocks.chunks(chunk_size).enumerate() {
        let start_index = chunk_idx * chunk_size + 1;
        let end_index = start_index + chunk.len() - 1;
        let content = build_chunk_content(chunk, start_index, end_index, total, separator);
        let output_path = chunk_output_path(output_dir, start_index, end_index);
        fs::write(&output_path, content).map_err(|error| Error::io(&output_path, error))?;
        chunk_count += 1;
    }

    Ok(chunk_count)
}

fn clean_txt_files(output_dir: &Path) -> Result<()> {
    if !output_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(output_dir).map_err(|error| Error::io(output_dir, error))? {
        let entry = entry.map_err(|error| Error::io(output_dir, error))?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "txt") {
            fs::remove_file(&path).map_err(|error| Error::io(&path, error))?;
        }
    }

    Ok(())
}

fn clean_chunk_files(output_dir: &Path) -> Result<()> {
    if !output_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(output_dir).map_err(|error| Error::io(output_dir, error))? {
        let entry = entry.map_err(|error| Error::io(output_dir, error))?;
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("Chunk_") && name.ends_with(".txt"))
        {
            fs::remove_file(&path).map_err(|error| Error::io(&path, error))?;
        }
    }

    Ok(())
}
