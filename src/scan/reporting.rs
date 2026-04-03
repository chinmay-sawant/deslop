use crate::analysis::ParsedFile;
use crate::model::FileReport;

pub(super) fn file_reports(parsed_files: &[ParsedFile]) -> Vec<FileReport> {
    parsed_files.iter().map(file_report).collect()
}

fn file_report(parsed_file: &ParsedFile) -> FileReport {
    FileReport {
        path: parsed_file.path.clone(),
        package_name: parsed_file.package_name.clone(),
        syntax_error: parsed_file.syntax_error,
        byte_size: parsed_file.byte_size,
        functions: parsed_file
            .functions
            .iter()
            .map(|function| function.fingerprint.clone())
            .collect(),
    }
}
