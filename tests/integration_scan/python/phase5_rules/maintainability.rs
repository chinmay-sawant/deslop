use super::{assert_rules_absent, assert_rules_present, scan_python_files};

#[test]
fn test_python_phase5_business_magic_and_utility_rules() {
    let report = scan_python_files(&[(
        "pkg/business_rules.py",
        python_fixture!("maintainability/business_rules_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "hardcoded_business_rule",
            "magic_value_branching",
            "reinvented_utility",
        ],
    );
}

#[test]
fn test_python_phase5_business_magic_and_utility_suppressions() {
    let report = scan_python_files(&[(
        "pkg/business_rules.py",
        python_fixture!("maintainability/business_rules_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "hardcoded_business_rule",
            "magic_value_branching",
            "reinvented_utility",
        ],
    );
}

#[test]
fn test_python_phase5_boundary_robustness_rules() {
    let report = scan_python_files(&[(
        "pkg/boundaries.py",
        python_fixture!("maintainability/boundaries_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "network_boundary_without_timeout",
            "environment_boundary_without_fallback",
            "external_input_without_validation",
        ],
    );
}

#[test]
fn test_python_phase5_boundary_robustness_suppressions() {
    let report = scan_python_files(&[(
        "pkg/boundaries.py",
        python_fixture!("maintainability/boundaries_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "network_boundary_without_timeout",
            "environment_boundary_without_fallback",
            "external_input_without_validation",
        ],
    );
}

#[test]
fn test_python_exception_and_debug_rules_positive() {
    let report = scan_python_files(&[(
        "pkg/debug_rules.py",
        python_fixture!("maintainability/exception_and_debug_positive.txt"),
    )]);

    assert_rules_present(
        &report,
        &[
            "exception_swallowed",
            "broad_exception_handler",
            "eval_exec_usage",
            "print_debugging_leftover",
            "commented_out_code",
        ],
    );
}

#[test]
fn test_python_exception_and_debug_rules_negative() {
    let report = scan_python_files(&[(
        "pkg/debug_rules.py",
        python_fixture!("maintainability/exception_and_debug_negative.txt"),
    )]);

    assert_rules_absent(
        &report,
        &[
            "exception_swallowed",
            "broad_exception_handler",
            "eval_exec_usage",
            "print_debugging_leftover",
            "commented_out_code",
        ],
    );
}
