use super::FixtureWorkspace;

#[test]
fn test_concat_loops() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("concat.go", go_fixture!("string_concat_loop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
}

#[test]
fn test_numeric_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("concat.go", go_fixture!("string_concat_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
}

#[test]
fn test_json_loops() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("json.go", go_fixture!("json_marshal_loop.txt"));

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "repeated_json_marshaling"
            && finding.function_name.as_deref() == Some("EncodeAll")
    }));
}

#[test]
fn test_json_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("json.go", go_fixture!("json_marshal_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "repeated_json_marshaling")
    );
}

#[test]
fn test_hot_path() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("hot_path.go", go_fixture!("hot_path_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "allocation_churn_in_loop")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "fmt_hot_path")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "reflection_hot_path")
    );
}

#[test]
fn test_hot_path_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("hot_path.go", go_fixture!("hot_path_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "allocation_churn_in_loop")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "fmt_hot_path")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "reflection_hot_path")
    );
}

#[test]
fn test_dataset_load() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("full_dataset.go", go_fixture!("full_dataset_load_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
}

#[test]
fn test_streaming_ok() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "full_dataset.go",
        go_fixture!("full_dataset_load_clean.txt"),
    );

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
}

#[test]
fn test_semantic_n_squared_rules_are_opt_in() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("alloc.go", go_fixture!("n_squared_alloc_slop.txt"));
    workspace.write_file("concat.go", go_fixture!("n_squared_concat_slop.txt"));

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.rule_id.as_str(),
            "likely_n_squared_allocation" | "likely_n_squared_string_concat"
        )
    }));
}

#[test]
fn test_semantic_n_squared_rules() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = true\n");
    workspace.write_file("alloc.go", go_fixture!("n_squared_alloc_slop.txt"));
    workspace.write_file("concat.go", go_fixture!("n_squared_concat_slop.txt"));

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "likely_n_squared_allocation"
            && finding.function_name.as_deref() == Some("Expand")
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "likely_n_squared_string_concat"
            && finding.function_name.as_deref() == Some("Render")
    }));
}

#[test]
fn test_semantic_n_squared_clean_fixtures() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = true\n");
    workspace.write_file("alloc.go", go_fixture!("n_squared_alloc_clean.txt"));
    workspace.write_file("concat.go", go_fixture!("n_squared_concat_clean.txt"));

    let report = workspace.scan();

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.rule_id.as_str(),
            "likely_n_squared_allocation" | "likely_n_squared_string_concat"
        )
    }));
}

#[test]
fn test_semantic_nested_query_escalation() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = true\n");
    workspace.write_file("query.go", go_fixture!("n_squared_query_slop.txt"));

    let report = workspace.scan();

    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "n_plus_one_query"
            && matches!(finding.severity, deslop::Severity::Error)
            && finding.function_name.as_deref() == Some("Load")
    }));
}

#[test]
fn test_semantic_nested_query_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".deslop.toml", "go_semantic_experimental = true\n");
    workspace.write_file("query.go", go_fixture!("n_squared_query_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "n_plus_one_query")
    );
}
