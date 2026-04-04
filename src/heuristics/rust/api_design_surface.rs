use crate::analysis::{ParsedFile, ParsedFunction};
use crate::model::{Finding, Severity};

use super::{
    builder_internal, function_finding, is_borrowed_pathbuf_type, is_borrowed_string_type,
    is_borrowed_vec_type, is_library_like, is_public_api, normalized_type, parameter_entries,
    parameter_name_and_type, return_type_text, return_type_uses_anyhow_like_result,
    return_type_uses_box_dyn_error,
};

pub(super) fn api_surface_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    if !is_public_api(function) {
        return Vec::new();
    }

    let mut findings = Vec::new();

    if is_library_like(file)
        && let Some(return_type) = return_type_text(&function.signature_text)
    {
        if return_type_uses_anyhow_like_result(file, &return_type) {
            findings.push(function_finding(
                file,
                function,
                "rust_public_anyhow_result",
                Severity::Warning,
                function.fingerprint.start_line,
                format!(
                    "public API {} returns an application-edge anyhow-style Result",
                    function.fingerprint.name
                ),
                vec![format!("return_type={return_type}")],
            ));
        }

        if return_type_uses_box_dyn_error(file, &return_type) {
            findings.push(function_finding(
                file,
                function,
                "rust_public_box_dyn_error",
                Severity::Warning,
                function.fingerprint.start_line,
                format!(
                    "public API {} exposes Box<dyn Error> instead of a clearer error surface",
                    function.fingerprint.name
                ),
                vec![format!("return_type={return_type}")],
            ));
        }
    }

    let parameters = parameter_entries(&function.signature_text);
    let bool_params = parameters
        .iter()
        .filter_map(|entry| parameter_name_and_type(entry))
        .filter(|(_, type_text)| normalized_type(type_text) == "bool")
        .collect::<Vec<_>>();

    if bool_params.len() == 1 && !builder_internal(function) {
        let (param_name, _) = &bool_params[0];
        findings.push(function_finding(
            file,
            function,
            "rust_public_bool_parameter_api",
            Severity::Info,
            function.fingerprint.start_line,
            format!(
                "public API {} exposes a raw boolean mode switch",
                function.fingerprint.name
            ),
            vec![format!("bool_parameter={param_name}")],
        ));
    }

    if !builder_internal(function) {
        for (param_name, type_text) in parameters
            .iter()
            .filter_map(|entry| parameter_name_and_type(entry))
        {
            if is_borrowed_string_type(&type_text) {
                findings.push(function_finding(
                    file,
                    function,
                    "rust_borrowed_string_api",
                    Severity::Info,
                    function.fingerprint.start_line,
                    format!(
                        "public API {} borrows String where &str would be more general",
                        function.fingerprint.name
                    ),
                    vec![
                        format!("parameter={param_name}"),
                        format!("type={type_text}"),
                    ],
                ));
            }

            if is_borrowed_vec_type(&type_text) {
                findings.push(function_finding(
                    file,
                    function,
                    "rust_borrowed_vec_api",
                    Severity::Info,
                    function.fingerprint.start_line,
                    format!(
                        "public API {} borrows Vec directly where a slice would be more flexible",
                        function.fingerprint.name
                    ),
                    vec![
                        format!("parameter={param_name}"),
                        format!("type={type_text}"),
                    ],
                ));
            }

            if is_borrowed_pathbuf_type(file, &type_text) {
                findings.push(function_finding(
                    file,
                    function,
                    "rust_borrowed_pathbuf_api",
                    Severity::Info,
                    function.fingerprint.start_line,
                    format!(
                        "public API {} borrows PathBuf where &Path would better match the contract",
                        function.fingerprint.name
                    ),
                    vec![
                        format!("parameter={param_name}"),
                        format!("type={type_text}"),
                    ],
                ));
            }
        }
    }

    findings
}
