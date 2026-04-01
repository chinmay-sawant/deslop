use crate::analysis::{AnalysisConfig, ParsedFile};
use crate::index::RepositoryIndex;
use crate::model::Finding;

use super::registry::{
    GO_CONFIGURABLE_FUNCTION_RULES, GO_FILE_FUNCTION_RULES, GO_FILE_RULES, GO_FUNCTION_RULES,
    GO_INDEXED_FUNCTION_RULES, GO_REPO_RULES, PYTHON_FILE_RULES, PYTHON_FUNCTION_RULES,
    PYTHON_INDEXED_FUNCTION_RULES, PYTHON_REPO_RULES, SHARED_FILE_RULES, SHARED_FUNCTION_RULES,
    SHARED_OPTIONAL_FUNCTION_RULES,
};

pub(crate) fn evaluate_shared(files: &[ParsedFile], _index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    for file in files {
        for rule in SHARED_FILE_RULES {
            findings.extend(rule(file));
        }

        for function in &file.functions {
            for rule in SHARED_OPTIONAL_FUNCTION_RULES {
                if let Some(finding) = rule(file, function) {
                    findings.push(finding);
                }
            }

            for rule in SHARED_FUNCTION_RULES {
                findings.extend(rule(file, function));
            }
        }
    }

    findings
}

pub(crate) fn evaluate_go_file(
    file: &ParsedFile,
    index: &RepositoryIndex,
    analysis_config: &AnalysisConfig,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for rule in GO_FILE_RULES {
        findings.extend(rule(file));
    }

    for function in &file.functions {
        for rule in GO_FUNCTION_RULES {
            findings.extend(rule(file, function));
        }

        for rule in GO_FILE_FUNCTION_RULES {
            findings.extend(rule(file, function, &file.imports));
        }

        for rule in GO_INDEXED_FUNCTION_RULES {
            findings.extend(rule(file, function, index));
        }

        for rule in GO_CONFIGURABLE_FUNCTION_RULES {
            findings.extend(rule(file, function, analysis_config.enable_go_semantic));
        }
    }

    findings
}

pub(crate) fn evaluate_go_repo(files: &[&ParsedFile], _index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    for rule in GO_REPO_RULES {
        findings.extend(rule(files));
    }

    findings
}

pub(crate) fn evaluate_python_file(file: &ParsedFile, index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    for rule in PYTHON_FILE_RULES {
        findings.extend(rule(file));
    }

    for function in &file.functions {
        for rule in PYTHON_FUNCTION_RULES {
            findings.extend(rule(file, function));
        }

        for rule in PYTHON_INDEXED_FUNCTION_RULES {
            findings.extend(rule(file, function, index));
        }
    }

    findings
}

pub(crate) fn evaluate_python_repo(files: &[&ParsedFile], index: &RepositoryIndex) -> Vec<Finding> {
    let mut findings = Vec::new();

    for rule in PYTHON_REPO_RULES {
        findings.extend(rule(files, index));
    }

    findings
}
