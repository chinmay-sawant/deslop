use crate::analysis::ParsedFile;
use crate::model::FileReport;
use crate::rules::RuleLanguage;

pub(super) fn file_reports(parsed_files: &[ParsedFile]) -> Vec<FileReport> {
    parsed_files.iter().map(file_report).collect()
}

fn file_report(parsed_file: &ParsedFile) -> FileReport {
    FileReport {
        language: rule_language(parsed_file.language),
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

const fn rule_language(language: crate::analysis::Language) -> RuleLanguage {
    match language {
        crate::analysis::Language::Go => RuleLanguage::Go,
        crate::analysis::Language::Python => RuleLanguage::Python,
        crate::analysis::Language::Rust => RuleLanguage::Rust,
    }
}
