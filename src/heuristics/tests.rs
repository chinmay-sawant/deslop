use super::common::{
    identifier_token_count, is_generic_name, is_title_case_comment, is_tutorial_style_comment,
    looks_like_global_symbol, normalize_name,
};

#[test]
fn detects_generic_names() {
    assert!(is_generic_name(&normalize_name("processData")));
    assert!(is_generic_name(&normalize_name("formatResponse")));
    assert!(!is_generic_name(&normalize_name("BuildCustomerLedger")));
}

#[test]
fn exported_names_look_global() {
    assert!(looks_like_global_symbol("SanitizeEmail"));
    assert!(!looks_like_global_symbol("sanitizeEmail"));
}

#[test]
fn counts_identifier_tokens() {
    assert_eq!(identifier_token_count("processUserInputAndValidateIt"), 6);
    assert_eq!(identifier_token_count("process_user_input"), 3);
}

#[test]
fn detects_title_case_comments() {
    assert!(is_title_case_comment("Run Processes Incoming Payloads"));
    assert!(!is_title_case_comment("Run processes incoming payloads."));
}

#[test]
fn detects_tutorial_style_comments() {
    assert!(is_tutorial_style_comment(
        "Run Processes Incoming Payloads\nThis function does X by doing Y because Z"
    ));
    assert!(!is_tutorial_style_comment("Run validates invoices."));
}
