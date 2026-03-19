use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::common::import_alias_lookup;

const WEAK_CRYPTO_IMPORTS: &[&str] = &["crypto/md5", "crypto/sha1", "crypto/des", "crypto/rc4"];

pub(super) fn weak_crypto_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let import_aliases = import_alias_lookup(&file.imports);
    let mut findings = Vec::new();

    for call in &function.calls {
        let Some(receiver) = &call.receiver else {
            continue;
        };
        let Some(import_path) = import_aliases.get(receiver) else {
            continue;
        };
        if !WEAK_CRYPTO_IMPORTS.contains(&import_path.as_str()) {
            continue;
        }

        findings.push(Finding {
            rule_id: "weak_crypto".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: call.line,
            end_line: call.line,
            message: format!(
                "function {} uses weak cryptographic primitive {}",
                function.fingerprint.name, import_path
            ),
            evidence: vec![
                format!("import alias {receiver} resolves to {import_path}"),
                format!("weak crypto call: {receiver}.{}", call.name),
            ],
        });
    }

    findings
}
