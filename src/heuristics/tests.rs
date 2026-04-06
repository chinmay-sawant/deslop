use super::common::{
    identifier_token_count, is_generic_name, is_global_sym, is_title_doc, is_tutorial_doc,
    normalize_name,
};

#[test]
fn detects_generic_names() {
    assert!(is_generic_name(&normalize_name("processData")));
    assert!(is_generic_name(&normalize_name("formatResponse")));
    assert!(is_generic_name("convertPayload"));
    assert!(!is_generic_name(&normalize_name("BuildCustomerLedger")));
}

#[test]
fn rejects_domain_mixed_names() {
    // Names that mix generic tokens with domain-specific tokens must not fire.
    assert!(!is_generic_name("handleGetFonts")); // two action tokens + domain noun
    assert!(!is_generic_name("applyTransform")); // two action tokens, no object
    assert!(!is_generic_name("generateAllContentWithImages")); // domain tokens present
    assert!(!is_generic_name("ConvertPDFDateToXMP")); // acronym domain tokens
    assert!(!is_generic_name("GenerateICCProfileObject")); // domain iccprofile token
    assert!(!is_generic_name("generateOutlineObjects")); // domain outline token
    assert!(!is_generic_name("createPKCS7SignedData")); // domain pkcs7signed token
    assert!(!is_generic_name("GetUserInfo")); // info is not a generic object token
    assert!(!is_generic_name("handleGenerateTemplatePDF")); // two actions + domain
}

#[test]
fn test_global_sym() {
    assert!(is_global_sym("SanitizeEmail"));
    assert!(!is_global_sym("sanitizeEmail"));
}

#[test]
fn counts_identifier_tokens() {
    assert_eq!(identifier_token_count("processUserInputAndValidateIt"), 6);
    assert_eq!(identifier_token_count("process_user_input"), 3);
}

#[test]
fn test_title_doc() {
    assert!(is_title_doc("Run Processes Incoming Payloads"));
    assert!(!is_title_doc("Run processes incoming payloads."));
}

#[test]
fn test_tutorial_doc() {
    assert!(is_tutorial_doc(
        "Run Processes Incoming Payloads\nThis function does X by doing Y because Z"
    ));
    assert!(!is_tutorial_doc("Run validates invoices."));
}
