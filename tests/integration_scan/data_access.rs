use super::FixtureWorkspace;

macro_rules! assert_rule_pair {
    ($rule_id:literal, $positive_fixture:literal, $clean_fixture:literal $(,)?) => {{
        let workspace = FixtureWorkspace::new();
        workspace.write_file("db.go", go_fixture!($positive_fixture));

        let report = workspace.scan();
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == $rule_id),
            "missing positive rule: {}",
            $rule_id
        );

        let workspace = FixtureWorkspace::new();
        workspace.write_file("db.go", go_fixture!($clean_fixture));

        let report = workspace.scan();
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == $rule_id),
            "unexpected clean rule: {}",
            $rule_id
        );
    }};
}

#[test]
fn test_db_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("db.go", go_fixture!("db_query_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "n_plus_one_query")
    );
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "go_perf_layer_database_access_query_inside_loop_without_batching"
    }));
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "wide_select_query")
    );
    assert!(report.findings.iter().any(|finding| {
        finding.rule_id == "go_perf_layer_database_access_select_star_on_hot_query"
    }));
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "likely_unindexed_query")
    );
}

#[test]
fn test_db_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("db.go", go_fixture!("db_query_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "n_plus_one_query")
    );
    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "go_perf_layer_database_access_query_inside_loop_without_batching"
    }));
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "wide_select_query")
    );
    assert!(!report.findings.iter().any(|finding| {
        finding.rule_id == "go_perf_layer_database_access_select_star_on_hot_query"
    }));
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "likely_unindexed_query")
    );
}

#[test]
fn test_default_transaction_enabled_for_bulk_create() {
    assert_rule_pair!(
        "default_transaction_enabled_for_bulk_create",
        "data_access_default_transaction_enabled_for_bulk_create_positive.txt",
        "data_access_default_transaction_enabled_for_bulk_create_clean.txt",
    );
}

#[test]
fn test_findinbatches_candidate_for_large_scan() {
    assert_rule_pair!(
        "findinbatches_candidate_for_large_scan",
        "data_access_findinbatches_candidate_for_large_scan_positive.txt",
        "data_access_findinbatches_candidate_for_large_scan_clean.txt",
    );
}

#[test]
fn test_many_column_or_filter_chain() {
    assert_rule_pair!(
        "many_column_or_filter_chain",
        "data_access_many_column_or_filter_chain_positive.txt",
        "data_access_many_column_or_filter_chain_clean.txt",
    );
}

#[test]
fn test_pgx_collectrows_unbounded_materialization() {
    assert_rule_pair!(
        "pgx_collectrows_unbounded_materialization",
        "data_access_pgx_collectrows_unbounded_materialization_positive.txt",
        "data_access_pgx_collectrows_unbounded_materialization_clean.txt",
    );
}

#[test]
fn test_rows_to_struct_allocation_per_row_without_reuse() {
    assert_rule_pair!(
        "rows_to_struct_allocation_per_row_without_reuse",
        "data_access_rows_to_struct_allocation_per_row_without_reuse_positive.txt",
        "data_access_rows_to_struct_allocation_per_row_without_reuse_clean.txt",
    );
}

#[test]
fn test_sqlx_select_large_slice_without_limit() {
    assert_rule_pair!(
        "sqlx_select_large_slice_without_limit",
        "data_access_sqlx_select_large_slice_without_limit_positive.txt",
        "data_access_sqlx_select_large_slice_without_limit_clean.txt",
    );
}

#[test]
fn test_unbounded_in_clause_expansion() {
    assert_rule_pair!(
        "unbounded_in_clause_expansion",
        "data_access_unbounded_in_clause_expansion_positive.txt",
        "data_access_unbounded_in_clause_expansion_clean.txt",
    );
}

#[test]
fn test_updates_map_allocated_per_row() {
    assert_rule_pair!(
        "updates_map_allocated_per_row",
        "data_access_updates_map_allocated_per_row_positive.txt",
        "data_access_updates_map_allocated_per_row_clean.txt",
    );
}
