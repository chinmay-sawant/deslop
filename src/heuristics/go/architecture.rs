use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{NamedLiteral, ParsedFile, ParsedFunction, StructTag};
use crate::io::{DEFAULT_MAX_BYTES, read_to_string_limited};
use crate::model::{Finding, Severity};

use super::super::common::is_generic_name;
use super::framework_patterns::{
    body_lines, has_import_path, import_aliases_for, is_gin_handler, is_request_path_function,
};

pub(crate) const BINDING_LOCATION: &str = file!();

const SERVICE_ROLE_HINTS: &[&str] = &["service", "services", "usecase", "usecases"];
const REPOSITORY_ROLE_HINTS: &[&str] = &[
    "repository",
    "repositories",
    "repo",
    "repos",
    "store",
    "stores",
    "dao",
    "daos",
];
const MODEL_ROLE_HINTS: &[&str] = &["model", "models", "entity", "entities", "schema", "schemas"];
const TRANSPORT_ROLE_HINTS: &[&str] = &[
    "handler",
    "handlers",
    "transport",
    "api",
    "controller",
    "controllers",
    "dto",
    "dtos",
];
const ROUTER_ROLE_HINTS: &[&str] = &["router", "routers", "route", "routes"];
const MIDDLEWARE_ROLE_HINTS: &[&str] = &["middleware", "middlewares"];
const VALIDATION_ROLE_HINTS: &[&str] = &["validation", "validator", "validators"];
const MAPPER_ROLE_HINTS: &[&str] = &["mapper", "mappers", "mapping"];
const UOW_ROLE_HINTS: &[&str] = &["uow", "unitofwork", "unit_of_work", "transaction", "tx"];

const REQUEST_STRUCT_SUFFIXES: &[&str] =
    &["Request", "Input", "Params", "Query", "Filter", "Payload"];
const RESPONSE_STRUCT_SUFFIXES: &[&str] = &["Response", "Result", "View", "Envelope"];
const API_ERROR_STRUCT_SUFFIXES: &[&str] = &["ErrorResponse", "ErrorEnvelope", "APIError"];
const GORM_HOOK_METHODS: &[&str] = &[
    "BeforeCreate",
    "AfterCreate",
    "BeforeSave",
    "AfterSave",
    "BeforeUpdate",
    "AfterUpdate",
    "BeforeDelete",
    "AfterDelete",
    "AfterFind",
];

pub(crate) fn go_architecture_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    let looks_like_test_source = file.is_test_file
        || file
            .imports
            .iter()
            .any(|import| import.path == "testing" || import.path.ends_with("/testing"));

    if looks_like_test_source {
        findings.extend(test_architecture_findings(file));
    }
    if file.is_test_file {
        return findings;
    }

    findings.extend(file_architecture_findings(file));

    for function in &file.functions {
        if function.is_test_function {
            continue;
        }
        findings.extend(function_architecture_findings(file, function));
        let body = function.body_text.as_str();
        if body.contains("Decode(&")
            && (body.contains("Vars(") || body.contains(".URL.Query().Get("))
        {
            findings.push(Finding {
                rule_id: "multiple_bind_sources_into_same_struct_without_precedence_contract"
                    .to_string(),
                severity: Severity::Info,
                path: file.path.clone(),
                function_name: Some(function.fingerprint.name.clone()),
                start_line: function.fingerprint.start_line,
                end_line: function.fingerprint.start_line,
                message: "handler merges several request sources into one DTO inline".to_string(),
                evidence: vec![
                    "body decode and path/query merges were observed in one handler flow"
                        .to_string(),
                ],
            });
        }
    }

    findings
}

pub(crate) fn go_architecture_repo_findings(files: &[&ParsedFile]) -> Vec<Finding> {
    let mut findings = Vec::new();

    if let Some(file) = files.iter().find(|file| {
        let lower_path = file.path.to_string_lossy().to_ascii_lowercase();
        !is_router_file(file)
            && is_transport_file(file)
            && lower_path.contains("route")
            && file.functions.iter().any(|function| {
                let name = function.fingerprint.name.as_str();
                name.contains("Route") || name.contains("Register")
            })
    }) {
        findings.push(Finding {
            rule_id: "route_setup_scattered_without_router_package".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "route setup lives outside a router-oriented package".to_string(),
            evidence: vec![
                "larger Gin services usually benefit from giving route setup a dedicated router home"
                    .to_string(),
            ],
        });
    }

    findings.extend(project_agnostic_repo_shape_findings(files));

    findings
}

include!("architecture/file_rules.rs");
include!("architecture/file_rules_handlers.rs");
include!("architecture/function_rules_transport.rs");
include!("architecture/function_rules_domain.rs");
include!("architecture/repo_rules.rs");
include!("architecture/test_rules.rs");
include!("architecture/helpers_core.rs");
include!("architecture/helpers_patterns.rs");
