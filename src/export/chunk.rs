use std::path::Path;

use super::block::FindingBlock;

pub(crate) fn build_chunk_content(
    blocks: &[FindingBlock],
    start_index: usize,
    end_index: usize,
    total: usize,
    separator: &str,
) -> String {
    let mut parts = vec![
        format!("Findings {start_index}-{end_index} of {total}"),
        String::new(),
    ];

    for (offset, block) in blocks.iter().enumerate() {
        if offset > 0 {
            parts.push(separator.to_string());
            parts.push(String::new());
        }
        parts.push(block.text.trim_end().to_string());
    }

    format!("{}\n", parts.join("\n"))
}

pub(crate) fn chunk_filename(start_index: usize, end_index: usize) -> String {
    format!("Chunk_{start_index}_{end_index}.txt")
}

pub(crate) fn chunk_output_path(
    output_dir: &Path,
    start_index: usize,
    end_index: usize,
) -> std::path::PathBuf {
    output_dir.join(chunk_filename(start_index, end_index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_filename_format() {
        assert_eq!(chunk_filename(1, 25), "Chunk_1_25.txt");
        assert_eq!(chunk_filename(26, 50), "Chunk_26_50.txt");
    }
}
