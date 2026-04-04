use crate::analysis::ParsedFile;
use crate::model::Finding;

pub(crate) const BINDING_LOCATION: &str = file!();

use super::{
    business_value_like, credential_like, enabled_like, field_type_mentions, file_finding,
    secret_like, sensitive_default_like, struct_severity,
};

pub(crate) fn domain_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for summary in file.structs() {
        for field in &summary.fields {
            if business_value_like(&field.name) && field.is_primitive {
                findings.push(file_finding(
                    file,
                    "rust_domain_raw_primitive",
                    struct_severity(summary),
                    field.line,
                    format!(
                        "struct {} uses a raw primitive for business data",
                        summary.name
                    ),
                    vec![format!(
                        "field {} has primitive type {}",
                        field.name, field.type_text
                    )],
                ));
            }

            if matches!(field.type_text.as_str(), "f32" | "f64")
                && (field.name.to_ascii_lowercase().contains("price")
                    || field.name.to_ascii_lowercase().contains("amount")
                    || field.name.to_ascii_lowercase().contains("money"))
            {
                findings.push(file_finding(
                    file,
                    "rust_domain_float_for_money",
                    struct_severity(summary),
                    field.line,
                    format!(
                        "struct {} uses floating-point storage for money-like data",
                        summary.name
                    ),
                    vec![format!("field {} has type {}", field.name, field.type_text)],
                ));
            }
        }

        let has_toggle = summary
            .fields
            .iter()
            .find(|field| field.is_bool && enabled_like(&field.name));
        let credential = summary
            .fields
            .iter()
            .find(|field| field.is_option && credential_like(&field.name));
        if let (Some(toggle), Some(credential)) = (has_toggle, credential) {
            findings.push(file_finding(
                file,
                "rust_domain_impossible_combination",
                struct_severity(summary),
                toggle.line,
                format!(
                    "struct {} mixes a toggle boolean with optional credentials",
                    summary.name
                ),
                vec![
                    format!("toggle field: {}", toggle.name),
                    format!("credential field: {}", credential.name),
                ],
            ));
        }

        if (summary.has_default_derive || summary.impl_default)
            && summary
                .fields
                .iter()
                .any(|field| sensitive_default_like(&field.name))
        {
            findings.push(file_finding(
                file,
                "rust_domain_default_produces_invalid",
                struct_severity(summary),
                summary.line,
                format!(
                    "struct {} has a Default implementation that may create invalid state",
                    summary.name
                ),
                vec![
                    format!("derives={}", summary.derives.join(", ")),
                    "sensitive or config-like fields are present on a type with Default"
                        .to_string(),
                ],
            ));
        }

        if summary.has_debug_derive && summary.fields.iter().any(|field| secret_like(&field.name)) {
            findings.push(file_finding(
                file,
                "rust_debug_secret",
                struct_severity(summary),
                summary.line,
                format!(
                    "struct {} derives Debug while carrying secret-like fields",
                    summary.name
                ),
                vec![
                    format!("derives={}", summary.derives.join(", ")),
                    "derive(Debug) can accidentally expose credentials in logs".to_string(),
                ],
            ));
        }

        if summary.has_deserialize_derive
            && summary
                .fields
                .iter()
                .any(|field| secret_like(&field.name) || sensitive_default_like(&field.name))
        {
            findings.push(file_finding(
                file,
                "rust_serde_sensitive_deserialize",
                struct_severity(summary),
                summary.line,
                format!(
                    "struct {} derives Deserialize for sensitive fields",
                    summary.name
                ),
                vec![
                    format!("derives={}", summary.derives.join(", ")),
                    "consider validation or custom Deserialize logic for sensitive inputs"
                        .to_string(),
                ],
            ));
        }

        if summary.has_serialize_derive
            && summary.fields.iter().any(|field| secret_like(&field.name))
        {
            findings.push(file_finding(
                file,
                "rust_serde_sensitive_serialize",
                struct_severity(summary),
                summary.line,
                format!(
                    "struct {} derives Serialize for secret-like fields",
                    summary.name
                ),
                vec![
                    format!("derives={}", summary.derives.join(", ")),
                    "consider skip_serializing or redaction for secret material".to_string(),
                ],
            ));
        }

        if summary
            .fields
            .iter()
            .any(|field| field_type_mentions(field, "Option<") && secret_like(&field.name))
            && summary.has_default_derive
        {
            findings.push(file_finding(
                file,
                "rust_domain_optional_secret_default",
                struct_severity(summary),
                summary.line,
                format!(
                    "struct {} defaults an optional secret-like field",
                    summary.name
                ),
                vec![
                    "defaults can hide whether sensitive configuration is actually valid"
                        .to_string(),
                ],
            ));
        }
    }

    findings
}
