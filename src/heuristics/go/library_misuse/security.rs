mod auth_session;
mod concurrency;
mod crypto;
mod data_exposure;
mod injection;
mod network;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::super::framework_patterns::{
    BodyLine, has_import_path, import_aliases_for, is_request_path_function,
};

use self::auth_session::auth_session_findings;
use self::concurrency::concurrency_security_findings;
use self::crypto::crypto_findings;
use self::data_exposure::data_exposure_findings;
use self::injection::injection_findings;
use self::network::network_tls_findings;

pub(super) fn security_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[BodyLine],
) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(crypto_findings(file, function, lines));
    findings.extend(injection_findings(file, function, lines));
    findings.extend(auth_session_findings(file, function, lines));
    findings.extend(concurrency_security_findings(file, function, lines));
    findings.extend(network_tls_findings(file, function, lines));
    findings.extend(data_exposure_findings(file, function, lines));
    findings
}

pub(super) fn file_security_findings(file: &ParsedFile) -> Vec<Finding> {
    data_exposure::file_security_findings(file)
}
