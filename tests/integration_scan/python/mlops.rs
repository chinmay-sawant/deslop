use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::super::create_temp_workspace;
use super::write_files;

fn assert_rules_present(report: &deslop::ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == *rule_id),
            "expected rule {rule_id} to fire"
        );
    }
}

fn assert_rules_absent(report: &deslop::ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == *rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }
}

const MLOPS_RULES: &[&str] = &[
    "pandas_iterrows_in_loop",
    "pandas_apply_with_simple_vectorizable_op",
    "pandas_concat_in_loop",
    "pandas_read_csv_without_dtypes",
    "pandas_to_dict_records_in_loop",
    "pandas_merge_without_validation",
    "pandas_full_dataframe_print_in_production",
    "pandas_eval_string_manipulation",
    "pandas_copy_in_loop",
    "numpy_python_loop_over_array",
    "numpy_append_in_loop",
    "numpy_vstack_hstack_in_loop",
    "model_eval_mode_missing",
    "torch_no_grad_missing_in_inference",
    "training_loop_without_zero_grad",
    "llm_api_call_in_loop_without_batching",
    "prompt_template_string_concat_in_loop",
    "hardcoded_api_key_in_source",
    "wandb_mlflow_log_in_tight_loop",
    "global_state_in_data_pipeline",
    "print_metrics_instead_of_logging",
    "entire_dataframe_copied_for_transform",
];

const ADVANCED_MLOPS_RULES: &[&str] = &[
    "vector_store_client_created_per_request",
    "langchain_chain_built_per_request",
    "tokenizer_encode_in_loop_without_cache",
];

#[test]
fn test_python_mlops_positive() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/mlops_code.py",
            python_fixture!("integration/mlops/mlops_positive.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_present(&report, MLOPS_RULES);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_mlops_clean() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/mlops_code.py",
            python_fixture!("integration/mlops/mlops_clean.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_absent(&report, MLOPS_RULES);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_mlops_phase3_advanced_positive() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/mlops_phase3_advanced.py",
            python_fixture!("integration/mlops/mlops_phase3_advanced_positive.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_present(&report, ADVANCED_MLOPS_RULES);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_mlops_phase3_advanced_clean() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/mlops_phase3_advanced.py",
            python_fixture!("integration/mlops/mlops_phase3_advanced_clean.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_absent(&report, ADVANCED_MLOPS_RULES);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
