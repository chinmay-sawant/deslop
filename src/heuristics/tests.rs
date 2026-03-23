use super::common::{
    identifier_token_count, is_generic_name, is_global_sym, is_title_doc, is_tutorial_doc,
    normalize_name,
};

#[test]
fn detects_generic_names() {
    assert!(is_generic_name(&normalize_name("processData")));
    assert!(is_generic_name(&normalize_name("formatResponse")));
    assert!(!is_generic_name(&normalize_name("BuildCustomerLedger")));
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
