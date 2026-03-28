use std::fs;

use deslop::{ScanOptions, Severity, scan_repository};

use super::{create_temp_workspace, write_fixture};

#[test]
fn test_python_fingerprints() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "app.py", python_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(report.functions_found, 2);
    assert!(report.parse_failures.is_empty());
    assert_eq!(report.files[0].package_name.as_deref(), Some("app"));

    let names = report.files[0]
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["build_summary", "render"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_syntax() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "broken.py", python_fixture!("broken.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert!(report.files[0].syntax_error);
    assert!(report.parse_failures.is_empty());

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_mixed_repo() {
    let temp_dir = create_temp_workspace();
    write_fixture(&temp_dir, "app.py", python_fixture!("simple.txt"));
    write_fixture(&temp_dir, "main.go", go_fixture!("simple.go"));
    write_fixture(&temp_dir, "src/main.rs", rust_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 3);
    assert_eq!(report.files_analyzed, 3);
    assert!(report.parse_failures.is_empty());

    let analyzed_paths = report
        .files
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(&temp_dir)
                .expect("report path should stay under the temp dir")
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(analyzed_paths, vec!["app.py", "main.go", "src/main.rs"]);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_rust_mixed_repo() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/render/service.py",
        python_fixture!("simple.txt"),
    );
    write_fixture(&temp_dir, "pkg/render/lib.rs", rust_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 2);
    assert!(report.parse_failures.is_empty());

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_same_directory_mixed_repo() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/render/__init__.py",
        python_fixture!("simple.txt"),
    );
    write_fixture(&temp_dir, "pkg/render/main.go", go_fixture!("simple.go"));
    write_fixture(&temp_dir, "pkg/render/lib.rs", rust_fixture!("simple.txt"));

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert_eq!(report.files_discovered, 3);
    assert_eq!(report.files_analyzed, 3);
    assert!(report.parse_failures.is_empty());

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "service.py",
        python_fixture!("rule_pack_positive.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_sync_io_in_async")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "exception_swallowed")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "eval_exec_usage")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "print_debugging_leftover")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_rule_suppressions() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "service.py",
        python_fixture!("rule_pack_negative.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "string_concat_in_loop")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_sync_io_in_async")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "full_dataset_load")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "exception_swallowed")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "eval_exec_usage")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "print_debugging_leftover")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_test_rule_suppressions() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "tests/test_service.py",
        python_fixture!("rule_pack_test_only.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(!report.findings.iter().any(|finding| {
        matches!(
            finding.rule_id.as_str(),
            "string_concat_in_loop"
                | "blocking_sync_io_in_async"
                | "full_dataset_load"
                | "exception_swallowed"
                | "eval_exec_usage"
                | "print_debugging_leftover"
        )
    }));

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase4_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/__init__.py",
        python_fixture!("phase4_positive.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "blocking_sync_io_in_async",
        "none_comparison",
        "side_effect_comprehension",
        "redundant_return_none",
        "hardcoded_path_string",
        "variadic_public_api",
        "temporary_collection_in_loop",
        "recursive_traversal_risk",
        "list_membership_in_loop",
        "repeated_len_in_loop",
        "builtin_reduction_candidate",
        "broad_exception_handler",
        "missing_context_manager",
        "public_api_missing_type_hints",
        "mixed_sync_async_module",
        "textbook_docstring_small_helper",
        "mixed_naming_conventions",
        "god_function",
        "god_class",
        "monolithic_init_module",
        "too_many_instance_attributes",
        "eager_constructor_collaborators",
        "over_abstracted_wrapper",
        "list_materialization_first_element",
        "deque_candidate_queue",
        "mixed_concerns_function",
        "name_responsibility_mismatch",
        "unrelated_heavy_import",
        "obvious_commentary",
        "enthusiastic_commentary",
        "commented_out_code",
        "repeated_string_literal",
        "duplicate_error_handler_block",
        "duplicate_validation_pipeline",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase4_suppressions() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/module.py",
        python_fixture!("phase4_negative.txt"),
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "blocking_sync_io_in_async",
        "none_comparison",
        "side_effect_comprehension",
        "redundant_return_none",
        "hardcoded_path_string",
        "variadic_public_api",
        "temporary_collection_in_loop",
        "recursive_traversal_risk",
        "list_membership_in_loop",
        "repeated_len_in_loop",
        "builtin_reduction_candidate",
        "broad_exception_handler",
        "missing_context_manager",
        "public_api_missing_type_hints",
        "mixed_sync_async_module",
        "textbook_docstring_small_helper",
        "mixed_naming_conventions",
        "god_function",
        "god_class",
        "monolithic_init_module",
        "too_many_instance_attributes",
        "eager_constructor_collaborators",
        "over_abstracted_wrapper",
        "list_materialization_first_element",
        "deque_candidate_queue",
        "mixed_concerns_function",
        "name_responsibility_mismatch",
        "unrelated_heavy_import",
        "obvious_commentary",
        "enthusiastic_commentary",
        "commented_out_code",
        "repeated_string_literal",
        "duplicate_error_handler_block",
        "duplicate_validation_pipeline",
    ] {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase4_repo_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/root.py",
        r#"
class Root:
    pass
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/base.py",
        r#"
from pkg.root import Root

SHARED_LITERAL = "this repeated literal belongs in one shared constant"

class Base(Root):
    pass

def normalize_payload(payload):
    cleaned = []
    for item in payload:
        cleaned.append(str(item).strip())
    return cleaned
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/mid.py",
        r#"
from pkg.base import Base

SHARED_LITERAL = "this repeated literal belongs in one shared constant"

class Mid(Base):
    pass
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/helpers.py",
        r#"
SHARED_LITERAL = "this repeated literal belongs in one shared constant"

def helper(payload):
    return [item for item in payload]
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/models.py",
        r#"
class Model:
    pass
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/services.py",
        r#"
class Service:
    pass
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/adapters.py",
        r#"
class Adapter:
    pass
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/leaf.py",
        r#"
from pkg.mid import Mid
from pkg.helpers import helper
from pkg.models import Model
from pkg.services import Service
from pkg.adapters import Adapter

SHARED_LITERAL = "this repeated literal belongs in one shared constant"

class Leaf(Mid):
    pass

def build_system(payload):
    model = Model()
    service = Service()
    adapter = Adapter()
    return helper([payload, model, service, adapter])
"#,
    );
    write_fixture(
        &temp_dir,
        "tests/test_helpers.py",
        r#"
def normalize_payload(payload):
    cleaned = []
    for item in payload:
        cleaned.append(str(item).strip())
    return cleaned
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "deep_inheritance_hierarchy",
        "tight_module_coupling",
        "duplicate_test_utility_logic",
        "cross_file_repeated_literal",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected repo rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_instance_attribute_escalation() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/heavy_state.py",
        r#"
class HeavyState:
    def __init__(self):
        self.a0 = 0
        self.a1 = 1
        self.a2 = 2
        self.a3 = 3
        self.a4 = 4
        self.a5 = 5
        self.a6 = 6
        self.a7 = 7
        self.a8 = 8
        self.a9 = 9
        self.a10 = 10
        self.a11 = 11
        self.a12 = 12
        self.a13 = 13
        self.a14 = 14
        self.a15 = 15
        self.a16 = 16
        self.a17 = 17
        self.a18 = 18
        self.a19 = 19
        self.a20 = 20

    def snapshot(self):
        return self.a0

    def describe(self):
        return self.a1
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.rule_id == "too_many_instance_attributes")
        .expect("expected too_many_instance_attributes finding");
    assert!(matches!(finding.severity, Severity::Warning));
    assert!(
        finding
            .evidence
            .iter()
            .any(|evidence| evidence == "tier=20_plus"),
        "expected the escalated 20-plus evidence tier"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_duplicate_query_fragment_rule() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/base.py",
        r#"
QUERY = "select id, status from reports where status = 'open' order by created_at"

def fetch_open_reports(cursor):
    return cursor.execute(QUERY)
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/helpers.py",
        r#"
QUERY = "SELECT id, status FROM reports WHERE status = 'open' ORDER BY created_at"

def build_query():
    return QUERY
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/services.py",
        r#"
QUERY = "SELECT  id,  status  FROM reports WHERE status = 'open' ORDER BY created_at"

def fetch(cursor):
    return cursor.execute(QUERY)
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "duplicate_query_fragment"),
        "expected duplicate_query_fragment to fire"
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "cross_file_repeated_literal"),
        "did not expect generic cross_file_repeated_literal for query-like strings"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_duplicate_query_fragment_skips_shared_constants_and_migrations() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/query_constants.py",
        r#"
OPEN_REPORTS_QUERY = "SELECT id, status FROM reports WHERE status = 'open' ORDER BY created_at"
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/query_templates.py",
        r#"
REPORT_STATUS_TEMPLATE = "SELECT id, status FROM reports WHERE status = '{status}' ORDER BY created_at"
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/service_a.py",
        r#"
from pkg.query_constants import OPEN_REPORTS_QUERY
from pkg.query_templates import REPORT_STATUS_TEMPLATE

def fetch_open_reports(cursor):
    return cursor.execute(OPEN_REPORTS_QUERY)

def fetch_by_status(cursor, status):
    return cursor.execute(REPORT_STATUS_TEMPLATE.format(status=status))
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/service_b.py",
        r#"
from pkg.query_constants import OPEN_REPORTS_QUERY
from pkg.query_templates import REPORT_STATUS_TEMPLATE

def load_open_reports(cursor):
    return cursor.execute(OPEN_REPORTS_QUERY)

def load_by_status(cursor, status):
    return cursor.execute(REPORT_STATUS_TEMPLATE.format(status=status))
"#,
    );
    write_fixture(
        &temp_dir,
        "migrations/0001_backfill_reports.py",
        r#"
SQL = "UPDATE reports SET status = 'open' WHERE status = 'draft'"

def upgrade(cursor):
    cursor.execute(SQL)
"#,
    );
    write_fixture(
        &temp_dir,
        "migrations/0002_backfill_reports.py",
        r#"
SQL = "UPDATE reports SET status = 'open' WHERE status = 'draft'"

def upgrade(cursor):
    cursor.execute(SQL)
"#,
    );
    write_fixture(
        &temp_dir,
        "migrations/0003_backfill_reports.py",
        r#"
SQL = "UPDATE reports SET status = 'open' WHERE status = 'draft'"

def upgrade(cursor):
    cursor.execute(SQL)
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "duplicate_query_fragment"),
        "did not expect duplicate_query_fragment for centralized constants, shared templates, or migrations"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_cross_file_copy_paste_rule() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/service_a.py",
        r#"
def build_profile(records):
    output = []
    for record in records:
        cleaned = str(record).strip().lower()
        if not cleaned:
            continue
        payload = {"value": cleaned, "length": len(cleaned)}
        output.append(payload)
    return output
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/service_b.py",
        r#"
def build_account(records):
    output = []
    for record in records:
        cleaned = str(record).strip().lower()
        if not cleaned:
            continue
        payload = {"value": cleaned, "length": len(cleaned)}
        output.append(payload)
    return output
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "cross_file_copy_paste_function"),
        "expected cross_file_copy_paste_function to fire"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_duplicate_transformation_pipeline_rule() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/ingest_a.py",
        r#"
def parse_payload(raw):
    return raw

def validate_payload(payload):
    return payload

def normalize_payload(payload):
    return payload

def enrich_payload(payload):
    return payload

def serialize_payload(payload):
    return payload

def build_report(raw):
    payload = parse_payload(raw)
    payload = validate_payload(payload)
    payload = normalize_payload(payload)
    payload = enrich_payload(payload)
    return serialize_payload(payload)
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/ingest_b.py",
        r#"
def parse_record(raw):
    return raw

def validate_record(payload):
    return payload

def transform_record(payload):
    return payload

def fetch_metadata(payload):
    return payload

def render_record(payload):
    return payload

def build_snapshot(raw):
    payload = parse_record(raw)
    payload = validate_record(payload)
    payload = transform_record(payload)
    payload = fetch_metadata(payload)
    return render_record(payload)
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "duplicate_transformation_pipeline"),
        "expected duplicate_transformation_pipeline to fire"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_duplicate_transformation_pipeline_skips_short_helpers() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/helpers_a.py",
        r#"
def parse_payload(raw):
    return raw.strip()

def normalize_payload(payload):
    return payload.lower()

def build_label(raw):
    payload = parse_payload(raw)
    return normalize_payload(payload)
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/helpers_b.py",
        r#"
def parse_record(raw):
    return raw.strip()

def transform_record(payload):
    return payload.lower()

def build_slug(raw):
    payload = parse_record(raw)
    return transform_record(payload)
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "duplicate_transformation_pipeline"),
        "did not expect duplicate_transformation_pipeline for short helper chains"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_monolithic_module_rule() {
    let temp_dir = create_temp_workspace();
    let mut module = String::from(
        r#"
import os
import json
import pathlib
import requests
import sqlite3
import csv
import logging
import tempfile
import hashlib
import itertools
import collections
import time

class ReportRow:
    def render(self):
        return "row"

class ReportBuilder:
    def build(self):
        return "report"

def load_config(path):
    return pathlib.Path(path).read_text()

def parse_rows(payload):
    return json.loads(payload)

def fetch_remote(url):
    response = requests.get(url)
    return response.text

def write_cache(path, payload):
    pathlib.Path(path).write_text(payload)

def export_rows(path, rows):
    with open(path, "w") as handle:
        writer = csv.writer(handle)
        for row in rows:
            writer.writerow(row)

def sync_reports(db_path, cache_path, url):
    logger = logging.getLogger("sync")
    temp_dir = tempfile.mkdtemp()
    logger.info("starting sync")
    connection = sqlite3.connect(db_path)
    payload = fetch_remote(url)
    rows = parse_rows(payload)
    digest = hashlib.sha256(payload.encode("utf-8")).hexdigest()
    deduped_rows = list(itertools.islice(rows, 0, len(rows)))
    queue = collections.deque(deduped_rows)
    while queue:
        row = queue.popleft()
        logger.debug("processing %s", row)
    write_cache(cache_path, payload)
    export_rows(cache_path + ".csv", deduped_rows)
    pathlib.Path(temp_dir).joinpath("digest.txt").write_text(digest)
    connection.commit()
    connection.close()
    time.sleep(0)
    return deduped_rows

def publish_reports(db_path, cache_path, url):
    logger = logging.getLogger("publish")
    temp_dir = tempfile.mkdtemp()
    connection = sqlite3.connect(db_path)
    payload = fetch_remote(url)
    rows = parse_rows(payload)
    digest = hashlib.sha256(payload.encode("utf-8")).hexdigest()
    queue = collections.deque(rows)
    while queue:
        row = queue.popleft()
        logger.info("publishing %s", row)
    write_cache(cache_path + ".bak", payload)
    export_rows(cache_path + ".published.csv", rows)
    pathlib.Path(temp_dir).joinpath("publish.txt").write_text(digest)
    connection.commit()
    connection.close()
    time.sleep(0)
    return rows
"#,
    );
    for index in 0..320 {
        module.push_str(&format!(
            "\ndef helper_{index}(payload):\n    record = str(payload).strip()\n    if not record:\n        return ''\n    return record.lower()\n"
        ));
    }
    write_fixture(&temp_dir, "pkg/module.py", &module);

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "monolithic_module"),
        "expected monolithic_module to fire"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_over_abstracted_wrapper_expansion() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/presenter.py",
        r#"
class ExportPresenter:
    def __init__(self, renderer, prefix):
        self.renderer = renderer
        self.prefix = prefix

    def render(self, payload):
        return self.renderer.render(f"{self.prefix}:{payload}")
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "over_abstracted_wrapper"),
        "expected over_abstracted_wrapper to fire for a ceremonial wrapper class"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_over_abstracted_wrapper_skips_lifecycle_classes() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/runtime.py",
        r#"
class BaseRuntime:
    pass

class ManagedRuntime(BaseRuntime):
    def __init__(self, client, logger):
        self.client = client
        self.logger = logger

    def start(self):
        self.logger.info("starting")
        return self.client.connect()

    def stop(self):
        return self.client.close()
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "over_abstracted_wrapper"),
        "did not expect over_abstracted_wrapper for lifecycle-heavy classes"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_name_responsibility_mismatch_expansion() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/parser.py",
        r#"
def parse_user(cursor, payload):
    cursor.execute("INSERT INTO users VALUES (?)", payload)
    cursor.commit()
    return payload
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/report_helper.py",
        r#"
import requests
import sqlite3

def helper_sync_reports(url, db_path):
    connection = sqlite3.connect(db_path)
    response = requests.get(url)
    response.raise_for_status()
    connection.execute("INSERT INTO reports VALUES (?)", (response.text,))
    connection.commit()
    return response.text
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report.findings.iter().any(|finding| {
            finding.rule_id == "name_responsibility_mismatch"
                && (finding.function_name.as_deref() == Some("parse_user")
                    || finding.path.ends_with("pkg/report_helper.py"))
        }),
        "expected expanded name_responsibility_mismatch anchors to fire"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_name_responsibility_mismatch_skips_honest_transformers() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/parser.py",
        r#"
import json

def parse_user(payload):
    return json.loads(payload)
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "name_responsibility_mismatch"),
        "did not expect name_responsibility_mismatch for honest parse helpers"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_business_magic_and_utility_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/policy.py",
        r#"
def decide_discount_tier(order_total, risk_score):
    if order_total >= 5000 and risk_score < 0.2:
        return "priority"
    elif order_total >= 2500:
        return "standard"
    return "manual_review"
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/archive.py",
        r#"
def rotate_archive(size, plan):
    if size > 5000:
        return "archive"
    elif plan == "enterprise" and size > 5000:
        return "archive"
    return "keep"
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/flatteners.py",
        r#"
import itertools

def flatten_batches(batches):
    output = []
    for batch in batches:
        for item in batch:
            output.append(item)
    return output
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "hardcoded_business_rule",
        "magic_value_branching",
        "reinvented_utility",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_business_magic_and_utility_suppressions() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/policy.py",
        r#"
PREMIUM_THRESHOLD = 5000
STANDARD_THRESHOLD = 2500

def decide_discount_tier(order_total, risk_score):
    if order_total >= PREMIUM_THRESHOLD and risk_score < 0.2:
        return "priority"
    elif order_total >= STANDARD_THRESHOLD:
        return "standard"
    return "manual_review"
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/archive.py",
        r#"
ARCHIVE_LIMIT = 5000

def rotate_archive(size, plan):
    if size > ARCHIVE_LIMIT:
        return "archive"
    elif plan == "enterprise":
        return "archive"
    return "keep"
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/flatteners.py",
        r#"
import itertools

def flatten_batches(batches):
    return list(itertools.chain.from_iterable(batches))
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "hardcoded_business_rule",
        "magic_value_branching",
        "reinvented_utility",
    ] {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_boundary_robustness_rules() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/network_sync.py",
        r#"
import requests

def sync_reports(url):
    response = requests.get(url)
    return response.json()
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/config_loader.py",
        r#"
import os

def load_runtime_config():
    api_url = os.getenv("API_URL")
    return {"api_url": api_url}
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/cli.py",
        r#"
import json
import sys

def run_cli():
    payload = json.loads(sys.argv[1])
    return payload["user"]
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "network_boundary_without_timeout",
        "environment_boundary_without_fallback",
        "external_input_without_validation",
    ] {
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "expected rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_boundary_robustness_suppressions() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/network_sync.py",
        r#"
import requests

def sync_reports(url):
    response = requests.get(url, timeout=5)
    return response.json()
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/config_loader.py",
        r#"
import os

def load_runtime_config():
    api_url = os.getenv("API_URL")
    if not api_url:
        raise ValueError("API_URL is required")
    return {"api_url": api_url}
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/cli.py",
        r#"
import json
import sys

def run_cli():
    if len(sys.argv) < 2:
        raise ValueError("missing payload")
    payload = json.loads(sys.argv[1])
    return payload["user"]
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    for rule_id in [
        "network_boundary_without_timeout",
        "environment_boundary_without_fallback",
        "external_input_without_validation",
    ] {
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.rule_id == rule_id),
            "did not expect rule {rule_id} to fire"
        );
    }

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_phase5_monolithic_module_skips_broad_legitimate_modules() {
    let temp_dir = create_temp_workspace();

    let mut registry_module = String::from(
        r#"
import os
import json
import pathlib
import logging
import hashlib
import itertools
import collections
import decimal
import fractions
import statistics
import datetime
import uuid

REGISTRY = {}

def register(name, value):
    REGISTRY[name] = value
    return value
"#,
    );
    for index in 0..500 {
        registry_module.push_str(&format!(
            "\ndef provide_{index}():\n    value = 'entry_{index}'\n    register(value, value)\n    return REGISTRY[value]\n"
        ));
    }
    write_fixture(&temp_dir, "pkg/registry.py", &registry_module);

    let mut schema_module = String::from(
        r#"
import datetime
import decimal
import typing
import uuid
import pathlib
import collections
import fractions
import statistics
import itertools
import hashlib
import json
import os
"#,
    );
    for index in 0..320 {
        schema_module.push_str(&format!(
            "\nclass EventSchema{index}:\n    event_id = 'event_{index}'\n    source = 'api'\n    kind = 'schema'\n    version = {index}\n"
        ));
    }
    write_fixture(&temp_dir, "pkg/schemas.py", &schema_module);

    let mut api_surface_module = String::from(
        r#"
import json
import pathlib
import typing
import logging
import urllib
import http
import dataclasses
import enum
import collections
import itertools
import datetime
import uuid

class Response:
    def __init__(self, payload):
        self.payload = payload

def render(payload):
    return Response(payload)
"#,
    );
    for index in 0..520 {
        api_surface_module.push_str(&format!(
            "\ndef route_{index}(request):\n    payload = {{'route': {index}, 'request': request}}\n    return render(payload)\n"
        ));
    }
    write_fixture(&temp_dir, "pkg/api_surface.py", &api_surface_module);

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

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

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}

#[test]
fn test_python_hallucination_rule() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "pkg/target.py",
        r#"
def existing_function():
    pass

class RealImportedClass:
    pass
"#,
    );
    write_fixture(
        &temp_dir,
        "pkg/caller.py",
        r#"
import pkg.target
from pathlib import Path
from third_party import ThirdPartyClient
from pkg.target import RealImportedClass, MissingImportedClass
from dataclasses import dataclass

@dataclass
class SessionBundle:
    name: str = "snapback"

class SnapBackTranscriptionClient:
    pass

def do_work():
    pkg.target.existing_function()
    pkg.target.imaginary_function()
    pkg.target.RealImportedClass()
    pkg.target.MissingQualifiedClass()

def do_local_work():
    RealLocalFunction()
    FakeLocalFunction()
    RealImportedClass()
    MissingImportedClass()
    Path("notes.md")
    ThirdPartyClient()
    RuntimeError("boom")
    SessionBundle()
    SnapBackTranscriptionClient()

def RealLocalFunction():
    pass
"#,
    );

    let report = scan_repository(&ScanOptions {
        root: temp_dir.clone(),
        respect_ignore: true,
    })
    .expect("scan should succeed");

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("imaginary_function")),
        "expected hallucinated_import_call to fire for imaginary_function"
    );

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("MissingQualifiedClass")),
        "expected hallucinated_import_call to fire for MissingQualifiedClass"
    );

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("MissingImportedClass")),
        "expected hallucinated_import_call to fire for MissingImportedClass"
    );

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_local_call"
                && finding.message.contains("FakeLocalFunction")),
        "expected hallucinated_local_call to fire for FakeLocalFunction"
    );

    // Ensure we don't fire for valid calls:
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("existing_function")),
        "did not expect finding for existing_function"
    );

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_import_call"
                && finding.message.contains("RealImportedClass")),
        "did not expect finding for RealImportedClass"
    );

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "hallucinated_local_call"
                && finding.message.contains("RealLocalFunction")),
        "did not expect finding for RealLocalFunction"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding.message.contains("Path")),
        "did not expect finding for imported stdlib class Path"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding.message.contains("ThirdPartyClient")),
        "did not expect finding for unresolved third-party import alias"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding.message.contains("RuntimeError")),
        "did not expect finding for builtin exception RuntimeError"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding.message.contains("SessionBundle")),
        "did not expect finding for local dataclass SessionBundle"
    );

    assert!(
        !report.findings.iter().any(|finding| matches!(
            finding.rule_id.as_str(),
            "hallucinated_import_call" | "hallucinated_local_call"
        ) && finding
            .message
            .contains("SnapBackTranscriptionClient")),
        "did not expect finding for local class SnapBackTranscriptionClient"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
