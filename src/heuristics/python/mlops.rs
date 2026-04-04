mod inference;
mod llm;
mod pandas_numpy;
mod pipeline;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

pub(crate) use self::inference::model_inference_findings;
pub(crate) use self::llm::llm_findings;
pub(crate) use self::pandas_numpy::{numpy_findings, pandas_findings};
pub(crate) use self::pipeline::{data_pipeline_findings, mlops_extra_findings};

fn indent_level(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

// ── Import-gating helpers ─────────────────────────────────────────────────────

fn has_import(file: &ParsedFile, module: &str) -> bool {
    file.imports
        .iter()
        .any(|imp| imp.path.contains(module) || imp.alias.contains(module))
}

fn has_any_import(file: &ParsedFile, modules: &[&str]) -> bool {
    modules.iter().any(|m| has_import(file, m))
}

fn is_handler_or_view(function: &ParsedFunction) -> bool {
    let sig = &function.signature_text;
    sig.contains("@app.route")
        || sig.contains("@bp.route")
        || sig.contains("@router.")
        || sig.contains("@api_view")
        || sig.contains("@app.get")
        || sig.contains("@app.post")
}

fn find_line(body: &str, needle: &str, base_line: usize) -> Option<usize> {
    for (i, line) in body.lines().enumerate() {
        if line.contains(needle) {
            return Some(base_line + i);
        }
    }
    None
}

fn make_finding(
    rule_id: &str,
    severity: Severity,
    file: &ParsedFile,
    function: &ParsedFunction,
    line: usize,
    msg_suffix: &str,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: format!("function {} {msg_suffix}", function.fingerprint.name),
        evidence: vec![format!("rule={rule_id}")],
    }
}
