use super::{
    Severity, assert_rules_absent, assert_rules_present, find_rule, report_has_rule,
    scan_generated_files, scan_python_files,
};

#[test]
fn test_python_phase5_instance_attribute_escalation() {
    let report = scan_python_files(&[(
        "pkg/heavy_state.py",
        python_fixture!("structure/heavy_state_positive.txt"),
    )]);

    let finding = find_rule(&report, "too_many_instance_attributes")
        .expect("expected too_many_instance_attributes finding");
    assert!(matches!(finding.severity, Severity::Warning));
    assert!(
        finding
            .evidence
            .iter()
            .any(|evidence| evidence == "tier=20_plus"),
        "expected the escalated 20-plus evidence tier"
    );
}

#[test]
fn test_python_structure_rule_family_positive() {
    let report = scan_python_files(&[
        (
            "pkg/god_function.py",
            python_fixture!("structure/god_function_positive.txt"),
        ),
        (
            "pkg/mixed_concerns.py",
            python_fixture!("structure/mixed_concerns_positive.txt"),
        ),
        (
            "pkg/presenter.py",
            python_fixture!("structure/over_abstracted_wrapper_positive.txt"),
        ),
        (
            "pkg/session_state.py",
            python_fixture!("structure/too_many_instance_attributes_positive.txt"),
        ),
        (
            "pkg/parser.py",
            python_fixture!("structure/name_responsibility_positive.txt"),
        ),
        (
            "pkg/billing.py",
            python_fixture!("structure/god_class_positive.txt"),
        ),
    ]);

    assert_rules_present(
        &report,
        &[
            "god_function",
            "mixed_concerns_function",
            "over_abstracted_wrapper",
            "too_many_instance_attributes",
            "name_responsibility_mismatch",
            "god_class",
        ],
    );
}

#[test]
fn test_python_structure_rule_family_negative() {
    let report = scan_python_files(&[
        (
            "pkg/god_function.py",
            python_fixture!("structure/god_function_negative.txt"),
        ),
        (
            "pkg/mixed_concerns.py",
            python_fixture!("structure/mixed_concerns_negative.txt"),
        ),
        (
            "pkg/presenter.py",
            python_fixture!("structure/over_abstracted_wrapper_negative.txt"),
        ),
        (
            "pkg/session_state.py",
            python_fixture!("structure/too_many_instance_attributes_negative.txt"),
        ),
        (
            "pkg/parser.py",
            python_fixture!("structure/name_responsibility_negative.txt"),
        ),
        (
            "pkg/billing.py",
            python_fixture!("structure/god_class_negative.txt"),
        ),
    ]);

    assert_rules_absent(
        &report,
        &[
            "god_function",
            "mixed_concerns_function",
            "over_abstracted_wrapper",
            "too_many_instance_attributes",
            "name_responsibility_mismatch",
            "god_class",
        ],
    );
}

#[test]
fn test_python_phase5_monolithic_module_rule() {
    let report = scan_generated_files(|workspace| {
        let mut module = String::from(python_fixture!(
            "integration/phase5/monolithic_module_prefix.txt"
        ));
        for index in 0..320 {
            module.push_str(&format!(
                "\ndef helper_{index}(payload):\n    record = str(payload).strip()\n    if not record:\n        return ''\n    return record.lower()\n"
            ));
        }
        workspace.write_file("pkg/module.py", &module);
    });

    assert!(
        report_has_rule(&report, "monolithic_module"),
        "expected monolithic_module to fire"
    );
}

#[test]
fn test_python_phase5_over_abstracted_wrapper_expansion() {
    let report = scan_python_files(&[(
        "pkg/presenter.py",
        python_fixture!("structure/over_abstracted_wrapper_positive.txt"),
    )]);

    assert!(
        report_has_rule(&report, "over_abstracted_wrapper"),
        "expected over_abstracted_wrapper to fire for a ceremonial wrapper class"
    );
}

#[test]
fn test_python_phase5_over_abstracted_wrapper_skips_lifecycle_classes() {
    let report = scan_python_files(&[(
        "pkg/runtime.py",
        python_fixture!("structure/over_abstracted_wrapper_negative.txt"),
    )]);

    assert!(
        !report_has_rule(&report, "over_abstracted_wrapper"),
        "did not expect over_abstracted_wrapper for lifecycle-heavy classes"
    );
}

#[test]
fn test_python_phase5_over_abstracted_wrapper_skips_dataclass_wrappers() {
    let report = scan_python_files(&[(
        "pkg/presenter.py",
        python_fixture!("structure/over_abstracted_wrapper_dataclass_negative.txt"),
    )]);

    assert!(
        !report_has_rule(&report, "over_abstracted_wrapper"),
        "did not expect over_abstracted_wrapper for dataclass wrappers"
    );
}

#[test]
fn test_python_phase5_name_responsibility_mismatch_expansion() {
    let report = scan_python_files(&[
        (
            "pkg/parser.py",
            python_fixture!("structure/name_responsibility_parser_positive.txt"),
        ),
        (
            "pkg/report_helper.py",
            python_fixture!("structure/name_responsibility_helper_positive.txt"),
        ),
    ]);

    assert!(
        report.findings.iter().any(|finding| {
            finding.rule_id == "name_responsibility_mismatch"
                && (finding.function_name.as_deref() == Some("parse_user")
                    || finding.path.ends_with("pkg/report_helper.py"))
        }),
        "expected expanded name_responsibility_mismatch anchors to fire"
    );
}

#[test]
fn test_python_phase5_name_responsibility_mismatch_skips_honest_transformers() {
    let report = scan_python_files(&[(
        "pkg/parser.py",
        python_fixture!("structure/name_responsibility_negative.txt"),
    )]);

    assert!(
        !report_has_rule(&report, "name_responsibility_mismatch"),
        "did not expect name_responsibility_mismatch for honest parse helpers"
    );
}

#[test]
fn test_python_phase5_monolithic_module_skips_broad_legitimate_modules() {
    let report = scan_generated_files(|workspace| {
        let mut registry_module = String::from(python_fixture!(
            "integration/phase5/legit_registry_prefix.txt"
        ));
        for index in 0..500 {
            registry_module.push_str(&format!(
                "\ndef provide_{index}():\n    value = 'entry_{index}'\n    register(value, value)\n    return REGISTRY[value]\n"
            ));
        }

        let mut schema_module = String::from(python_fixture!(
            "integration/phase5/legit_schemas_prefix.txt"
        ));
        for index in 0..320 {
            schema_module.push_str(&format!(
                "\nclass EventSchema{index}:\n    event_id = 'event_{index}'\n    source = 'api'\n    kind = 'schema'\n    version = {index}\n"
            ));
        }

        let mut api_surface_module = String::from(python_fixture!(
            "integration/phase5/legit_api_surface_prefix.txt"
        ));
        for index in 0..520 {
            api_surface_module.push_str(&format!(
                "\ndef route_{index}(request):\n    payload = {{'route': {index}, 'request': request}}\n    return render(payload)\n"
            ));
        }

        workspace.write_file("pkg/registry.py", &registry_module);
        workspace.write_file("pkg/schemas.py", &schema_module);
        workspace.write_file("pkg/api_surface.py", &api_surface_module);
    });

    let flagged_paths = report
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "monolithic_module")
        .map(|finding| finding.path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        flagged_paths.is_empty(),
        "did not expect broad-but-legitimate modules to be flagged: {flagged_paths:?}"
    );
}
