use deslop::ScanReport;

use super::super::support::{
    FixtureWorkspace, assert_rules_absent, assert_rules_present, scan_files,
};

fn scan(files: &[(&str, &str)]) -> ScanReport {
    scan_files(files)
}

#[test]
fn test_project_agnostic_architecture_rules_positive() {
    let report = scan(&[
        (
            "app/service_layer.py",
            r#"
import os

GLOBAL_CLIENT = Client()
app.run()

class ServiceConfig:
    def __init__(self):
        self.token = os.getenv("API_TOKEN")
"#,
        ),
        (
            "app/main.py",
            r#"
def main(customer):
    repo = Repository()
    cache = Cache()
    client = Client()
    if customer.is_premium:
        return {"data": customer, "status": 200}
    return {"data": customer, "status": 202}
"#,
        ),
    ]);

    assert_rules_present(
        &report,
        &[
            "constructor_reads_global_config_inline",
            "entrypoint_builds_dependency_graph_inside_hot_function",
            "function_returns_domain_value_and_transport_metadata",
            "module_exposes_mutable_singleton_client",
            "module_import_starts_runtime_bootstrap",
        ],
    );
}

#[test]
fn test_project_agnostic_architecture_rules_negative() {
    let report = scan(&[(
        "app/main.py",
        r#"
class ServiceConfig:
    def __init__(self, token):
        self.token = token

def main(customer, repo, cache, client):
    return customer
"#,
    )]);

    assert_rules_absent(
        &report,
        &[
            "constructor_reads_global_config_inline",
            "entrypoint_builds_dependency_graph_inside_hot_function",
            "function_returns_domain_value_and_transport_metadata",
        ],
    );
}

#[test]
fn test_project_agnostic_boundary_and_discipline_rules_positive() {
    let report = scan(&[(
        "app/api.py",
        r#"
from typing import Mapping

def mutate(items=[], path="tmp.txt", metadata: Mapping[str, int] = {}):
    callbacks = []
    for value in items:
        callbacks.append(lambda: value)
    payload = open(path).read()
    if not items:
        raise ValueError("missing items")
    if metadata:
        metadata.update({"loaded": True})
    return payload

def process(data, strict: bool = False, mode: str = "fast", retries: int = 0, timeout: int = 1, limit: int = 10):
    response = open("data.txt").read()
    if not data:
        raise ValueError("bad")
    if strict:
        return {"kind": "strict"}
    if mode == "fast":
        return {"kind": "fast"}
    return {"kind": "slow"}
"#,
    )]);

    assert_rules_present(
        &report,
        &[
            "mutable_default_argument_leaks_state_across_calls",
            "closure_captures_loop_variable_without_binding",
            "path_boundary_accepts_unexpanded_or_relative_input_without_normalization",
            "function_accepts_mapping_protocol_but_mutates_input",
            "boolean_flag_parameter_controls_unrelated_behaviors",
            "expensive_work_starts_before_input_validation",
            "function_returns_multiple_unlabeled_shape_variants",
        ],
    );
}

#[test]
fn test_project_agnostic_hotpath_and_performance_rules_positive() {
    let report = scan(&[(
        "app/processor.py",
        r#"
import json
import re
import subprocess

def crunch(values, mapping, raw):
    text = ""
    if list(values):
        pass
    for value in values:
        matcher = re.compile(r"\d+")
        text += "," + value
        if value in ["a", "b", "c"]:
            pass
        if value in list(mapping.keys()):
            pass
        subprocess.run(["echo", value])
        json.loads(json.dumps({"value": value, "copy": True}))
    return text

def export_payload(path):
    data = open(path).read()
    more = open(path).read()
    blob = data.encode().decode()
    values = list(item for item in data)
    if len(values):
        return blob + more
    return ""

def summarize(items):
    return any([item.strip().lower() for item in items])
"#,
    )]);

    assert_rules_present(
        &report,
        &[
            "regex_compiled_on_each_hot_call",
            "json_roundtrip_used_for_object_copy",
            "membership_test_against_list_or_tuple_literal_inside_loop",
            "list_of_keys_materialized_for_membership_check",
            "subprocess_or_shell_call_inside_record_processing_loop",
            "repeated_file_open_for_same_resource_within_single_operation",
            "bytes_text_bytes_roundtrip_without_transformation",
            "quadratic_string_building_via_plus_equals",
            "generator_materialized_to_tuple_or_list_only_for_len_or_truthiness",
            "any_or_all_wraps_list_comprehension_instead_of_generator",
        ],
    );
}

#[test]
fn test_project_agnostic_quality_maintainability_observability_structure_rules_positive() {
    let report = scan(&[
        (
            "pkg/helpers.py",
            r#"
import logging
from uuid import uuid4

def build_report(mode: str, first: str, second: str, third: int, fourth: bool, fifth: float) -> dict[str, bool]:
    logger = logging.getLogger(__name__)
    request_id = uuid4()
    request_id = uuid4()
    ids = []
    names = []
    for record in [1, 2, 3]:
        ids.append(record)
        names.append(str(record))
        logger.debug(f"payload {record}")
    try:
        return None
    except Exception:
        logger.error("payload=%s", {"value": mode})
        return {"ok": True}

def summarize_payload(first, second, third, fourth):
    return (first, second, third, fourth)
"#,
        ),
        (
            "pkg/common_manager.py",
            r#"
REGISTRY = {}
register("default", REGISTRY)

class BaseRecord:
    id = 0
    name = ""

class DataProcessor(BaseRecord):
    def __init__(self):
        self.client = Client()
        self.repo = Repo()
        self.cache = Cache()

    async def run(self):
        return self.render()

    def create(self):
        return {}

    def parse(self):
        return {}

    def save(self):
        return {}

    def render(self):
        return {}
"#,
        ),
    ]);

    assert_rules_present(
        &report,
        &[
            "tuple_return_with_three_or_more_positional_fields_in_public_api",
            "parallel_lists_used_instead_of_record_object",
            "logger_instance_created_inside_function_body",
            "expensive_log_argument_built_without_is_enabled_guard",
            "correlation_id_recomputed_multiple_times_in_same_workflow",
            "public_api_returns_none_or_value_without_explicit_optional_contract",
            "fallback_branch_swallows_invariant_violation_and_returns_plausible_default",
            "module_global_registry_mutated_from_import_time_registration",
            "class_mixes_factory_parsing_persistence_and_presentation_roles",
            "sync_and_async_contracts_mixed_on_same_interface_family",
        ],
    );
}

#[test]
fn test_project_agnostic_packaging_rules_positive() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        (
            "pkg/__init__.py",
            r#"
import pandas
import click
from importlib.metadata import version
from .alpha import Tool
from .beta import Tool
from .gamma import Helper
from .delta import Extra

PACKAGE_VERSION = version("pkg")
"#,
        ),
        (
            "pkg/core.py",
            r#"
def run_plugins(items):
    import pkg.alpha
    plugin_names = []
    for item in items:
        for name in pkgutil.iter_modules():
            plugin_names.append(name)
            print(name)
    return items
"#,
        ),
        (
            "pkg/alpha.py",
            "from common.helpers import helper\nfrom common.config import setting\n",
        ),
        (
            "pkg/beta.py",
            "from common.helpers import helper\nfrom common.config import setting\n",
        ),
        (
            "pkg/gamma.py",
            "from common.helpers import helper\nfrom common.config import setting\n",
        ),
        (
            "pkg/delta.py",
            "from common.helpers import helper\nfrom common.config import setting\n",
        ),
        ("pkg/fake_client.py", "VALUE = 1\n"),
        ("common/helpers.py", "def helper():\n    return 1\n"),
        ("common/config.py", "setting = 1\n"),
    ]);

    let report = workspace.scan();

    assert_rules_present(
        &report,
        &[
            "heavy_optional_dependency_imported_by_package_root",
            "cli_only_dependency_imported_by_library_entry_module",
            "package_init_performs_metadata_version_lookup_on_import",
            "circular_import_hidden_by_function_local_import_on_hot_path",
            "plugin_discovery_scans_filesystem_each_invocation",
            "test_helpers_shipped_inside_production_package_path",
            "public_api_surface_defined_only_by_import_side_effects",
            "package_root_reexports_large_dependency_tree_by_default",
            "monolithic_common_package_becomes_transitive_dependency_for_most_modules",
        ],
    );
}
