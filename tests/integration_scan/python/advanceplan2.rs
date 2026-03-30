use std::fs;

use deslop::{ScanOptions, scan_repository};

use super::super::create_temp_workspace;
use super::write_files;

fn assert_rules_present(report: &deslop::ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            report.findings.iter().any(|finding| finding.rule_id == *rule_id),
            "expected rule {rule_id} to fire"
        );
    }
}

fn assert_rules_absent(report: &deslop::ScanReport, rule_ids: &[&str]) {
    for rule_id in rule_ids {
        assert!(
            !report.findings.iter().any(|finding| finding.rule_id == *rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }
}

#[test]
fn test_python_advanceplan2_async_rules() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/async_service.py",
            python_fixture!("integration/advanceplan2/async_positive.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_present(
        &report,
        &[
            "untracked_asyncio_task",
            "background_task_exception_unobserved",
            "async_lock_held_across_await",
            "async_retry_sleep_without_backoff",
        ],
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_advanceplan2_async_clean() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/async_service.py",
            python_fixture!("integration/advanceplan2/async_clean.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_absent(
        &report,
        &[
            "untracked_asyncio_task",
            "background_task_exception_unobserved",
            "async_lock_held_across_await",
            "async_retry_sleep_without_backoff",
        ],
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_advanceplan2_contract_rules() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/contracts.py",
            python_fixture!("integration/advanceplan2/contracts_positive.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_present(
        &report,
        &[
            "mutable_default_argument",
            "dataclass_mutable_default",
            "dataclass_heavy_post_init",
            "option_bag_model",
            "public_any_type_leak",
            "typeddict_unchecked_access",
        ],
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_advanceplan2_contract_clean() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/contracts.py",
            python_fixture!("integration/advanceplan2/contracts_clean.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_absent(
        &report,
        &[
            "mutable_default_argument",
            "dataclass_mutable_default",
            "dataclass_heavy_post_init",
            "option_bag_model",
            "public_any_type_leak",
            "typeddict_unchecked_access",
        ],
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_advanceplan2_import_time_rules() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/bootstrap.py",
            python_fixture!("integration/advanceplan2/import_time_positive.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_present(
        &report,
        &[
            "import_time_network_call",
            "import_time_file_io",
            "import_time_subprocess",
            "module_singleton_client_side_effect",
            "mutable_module_global_state",
            "import_time_config_load",
        ],
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_advanceplan2_import_time_clean() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/bootstrap.py",
            python_fixture!("integration/advanceplan2/import_time_clean.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_absent(
        &report,
        &[
            "import_time_network_call",
            "import_time_file_io",
            "import_time_subprocess",
            "module_singleton_client_side_effect",
            "mutable_module_global_state",
            "import_time_config_load",
        ],
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_advanceplan2_boundary_rules() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/boundary.py",
            python_fixture!("integration/advanceplan2/boundary_positive.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_present(
        &report,
        &[
            "unsafe_yaml_loader",
            "pickle_deserialization_boundary",
            "subprocess_shell_true",
            "tar_extractall_unfiltered",
            "tempfile_without_cleanup",
        ],
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_advanceplan2_boundary_clean() {
    let temp_dir = create_temp_workspace();
    write_files(
        &temp_dir,
        &[(
            "pkg/boundary.py",
            python_fixture!("integration/advanceplan2/boundary_clean.txt"),
        )],
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_rules_absent(
        &report,
        &[
            "unsafe_yaml_loader",
            "pickle_deserialization_boundary",
            "subprocess_shell_true",
            "tar_extractall_unfiltered",
            "tempfile_without_cleanup",
        ],
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}