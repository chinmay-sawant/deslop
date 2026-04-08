use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{NamedLiteral, ParsedFile, ParsedFunction, StructTag};
use crate::model::{Finding, Severity};

use super::framework_patterns::{
    body_lines, has_import_path, import_aliases_for, is_gin_handler, is_request_path_function,
};
use super::super::common::is_generic_name;

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

const REQUEST_STRUCT_SUFFIXES: &[&str] = &["Request", "Input", "Params", "Query", "Filter", "Payload"];
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

    if file.is_test_file {
        findings.extend(test_architecture_findings(file));
        return findings;
    }

    findings.extend(file_architecture_findings(file));

    for function in &file.functions {
        if function.is_test_function {
            continue;
        }
        findings.extend(function_architecture_findings(file, function));
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

    findings
}

fn file_architecture_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    let import_line = file.imports.first().map(|import| import.line).unwrap_or(1);
    let tag_map = struct_tag_map(file);
    let gorm_structs = gorm_struct_names(file, &tag_map);

    if is_service_file(file) && has_import_path(file, "github.com/gin-gonic/gin") {
        findings.push(file_finding(
            file,
            "service_imports_gin_directly",
            Severity::Warning,
            import_line,
            "service package depends on Gin directly",
            vec!["service-oriented files should prefer transport-neutral contracts over *gin.Context or Gin-only helpers".to_string()],
        ));
    }

    if is_repository_file(file)
        && (has_import_path(file, "github.com/gin-gonic/gin") || has_import_path(file, "net/http"))
    {
        findings.push(file_finding(
            file,
            "repository_depends_on_gin_or_http",
            Severity::Warning,
            import_line,
            "repository package imports transport-facing HTTP libraries",
            vec!["repositories should stay below the transport boundary".to_string()],
        ));
    }

    if is_repository_file(file) && import_path_has_any_role(file, SERVICE_ROLE_HINTS) {
        findings.push(file_finding(
            file,
            "repository_depends_on_service_package",
            Severity::Warning,
            import_line,
            "repository package depends on a service package",
            vec!["this inverts the usual dependency direction from service to repository".to_string()],
        ));
    }

    if is_model_file(file)
        && (has_import_path(file, "github.com/gin-gonic/gin")
            || has_import_path(file, "net/http")
            || import_path_has_any_role(file, TRANSPORT_ROLE_HINTS))
    {
        findings.push(file_finding(
            file,
            "model_package_depends_on_transport_or_gin",
            Severity::Warning,
            import_line,
            "model-oriented package depends on transport-facing code",
            vec!["models should not need Gin, net/http, or handler DTO packages".to_string()],
        ));
    }

    if is_service_file(file) && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS) {
        findings.push(file_finding(
            file,
            "service_calls_handler_helper",
            Severity::Warning,
            import_line,
            "service file imports transport or handler helper packages",
            vec!["service logic should stay above repositories and below transport adapters".to_string()],
        ));
    }

    if is_transport_file(file)
        && file
            .go_structs()
            .iter()
            .any(|go_struct| go_struct.fields.iter().any(|field| field.type_text.contains("*gin.Context")))
    {
        let line = file
            .go_structs()
            .iter()
            .flat_map(|go_struct| go_struct.fields.iter())
            .find(|field| field.type_text.contains("*gin.Context"))
            .map(|field| field.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "gin_handler_stores_context_in_struct_field",
            Severity::Warning,
            line,
            "transport struct stores *gin.Context in a field",
            vec!["Gin request contexts are request-scoped and usually should not be retained on long-lived structs".to_string()],
        ));
    }

    if let Some(var) = file.package_vars().iter().find(|var| {
        is_transport_file(file)
            && (global_singleton_name(&var.name)
                || type_looks_like_global_singleton(var.type_text.as_deref().unwrap_or_default()))
    }) {
        findings.push(file_finding(
            file,
            "gin_handler_uses_global_singletons",
            Severity::Info,
            var.line,
            "transport package defines mutable singleton-like state",
            vec![format!("package variable {} looks like a shared runtime dependency", var.name)],
        ));
    }

    if file.package_vars().iter().any(|var| {
            var.type_text
                .as_deref()
                .is_some_and(|text| text.contains("*gin.Engine"))
        })
    {
        let line = file
            .package_vars()
            .iter()
            .find(|var| {
                var.type_text
                    .as_deref()
                    .is_some_and(|text| text.contains("*gin.Engine"))
            })
            .map(|var| var.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "gin_engine_as_global_singleton",
            Severity::Warning,
            line,
            "package defines a package-level *gin.Engine singleton",
            vec!["explicit startup-owned router construction is usually easier to test and compose".to_string()],
        ));
    }

    if file_matches_any_role(file, &["helper", "helpers", "util", "utils", "common"])
        && (import_path_has_any_role(file, SERVICE_ROLE_HINTS)
            || import_path_has_any_role(file, REPOSITORY_ROLE_HINTS)
            || import_path_has_any_role(file, MODEL_ROLE_HINTS)
            || has_import_path(file, "gorm.io/gorm"))
    {
        findings.push(file_finding(
            file,
            "helper_or_utils_package_contains_domain_logic",
            Severity::Info,
            import_line,
            "generic helper or utils package imports domain-specific layers",
            vec!["generic packages usually become hard to govern when they absorb concrete business behavior".to_string()],
        ));
    }

    if file_matches_any_role(file, &["cmd", "main"])
        && (import_path_has_any_role(file, REPOSITORY_ROLE_HINTS)
            || has_import_path(file, "gorm.io/gorm")
            || has_sql_like_import(file))
    {
        findings.push(file_finding(
            file,
            "cmd_or_main_contains_domain_rules",
            Severity::Info,
            import_line,
            "main or cmd package depends directly on business persistence behavior",
            vec!["entrypoints are usually clearer when they focus on bootstrap and delegate business rules".to_string()],
        ));
    }

    if file.go_structs().iter().any(|s| s.name.ends_with("Validator")) && !file_matches_any_role(file, VALIDATION_ROLE_HINTS) {
        let line = file
            .go_structs()
            .iter()
            .find(|s| s.name.ends_with("Validator"))
            .map(|s| s.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "validator_outside_validation_package",
            Severity::Info,
            line,
            "validator type lives outside a validation-oriented package",
            vec!["shared validators are usually easier to find and reuse when they have a dedicated home".to_string()],
        ));
    }

    if file.go_structs().iter().any(|s| s.name.ends_with("Middleware")) && !file_matches_any_role(file, MIDDLEWARE_ROLE_HINTS) {
        let line = file
            .go_structs()
            .iter()
            .find(|s| s.name.ends_with("Middleware"))
            .map(|s| s.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "middleware_type_outside_middleware_package",
            Severity::Info,
            line,
            "middleware type lives outside a middleware-oriented package",
            vec!["cross-cutting middleware is usually easier to govern when it stays in a dedicated package".to_string()],
        ));
    }

    if let Some(literal) = file.pkg_strings.iter().find(|literal| {
        !is_repository_file(file) && !file.is_test_file && looks_like_sql_literal(&literal.value)
    }) {
        findings.push(file_finding(
            file,
            "sql_query_constants_outside_repository_package",
            Severity::Info,
            literal.line,
            "SQL query template is declared outside a repository-oriented package",
            vec![format!("query-like literal assigned to {}", literal.name)],
        ));
    }

    if let Some(function) = file.functions.iter().find(|function| {
        !is_repository_file(file)
            && !file_matches_any_role(file, UOW_ROLE_HINTS)
            && transaction_helper_name(&function.fingerprint.name)
    }) {
        findings.push(file_finding(
            file,
            "transaction_helper_outside_repository_or_uow_package",
            Severity::Info,
            function.fingerprint.start_line,
            "transaction helper lives outside repository or unit-of-work packages",
            vec!["transaction orchestration helpers are usually easier to reason about when grouped near repository or unit-of-work code".to_string()],
        ));
    }

    if file_matches_any_role(file, &["common", "base", "shared", "utils"])
        && mixed_role_symbol_count(file) >= 3
    {
        findings.push(file_finding(
            file,
            "shared_package_named_common_base_utils_with_mixed_exports",
            Severity::Info,
            1,
            "shared package mixes several architectural roles under one generic namespace",
            vec!["generic shared packages often become dumping grounds for unrelated concerns".to_string()],
        ));
    }

    if cross_layer_import_violation(file) {
        findings.push(file_finding(
            file,
            "cross_layer_import_violation_by_package_name",
            Severity::Warning,
            import_line,
            "package imports appear to violate the expected layer direction",
            vec!["the import graph suggests a lower layer reaching upward into transport or service code".to_string()],
        ));
    }

    if let Some((line, role, mismatch)) = role_drift(file) {
        findings.push(file_finding(
            file,
            "package_name_role_drift",
            Severity::Info,
            line,
            format!("package named {role} mainly exports {mismatch}-oriented symbols"),
            vec!["package naming becomes misleading when most exported types belong to another role".to_string()],
        ));
    }

    for go_struct in file.go_structs() {
        if go_struct.name.ends_with("Service") && !is_service_file(file) {
            findings.push(file_finding(
                file,
                "service_type_outside_service_package",
                Severity::Info,
                go_struct.line,
                format!("service type {} lives outside a service-oriented package", go_struct.name),
                vec!["layered repos usually keep concrete services under service or services packages".to_string()],
            ));
        }

        if repository_type_name(&go_struct.name) && !is_repository_file(file) {
            findings.push(file_finding(
                file,
                "repository_type_outside_repository_package",
                Severity::Info,
                go_struct.line,
                format!(
                    "repository-like type {} lives outside a repository-oriented package",
                    go_struct.name
                ),
                vec!["repository, repo, or store implementations are easier to govern when they live together".to_string()],
            ));
        }

        if request_struct_name(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["json:", "form:", "uri:", "binding:", "validate:"])
            && !is_transport_file(file)
        {
            findings.push(file_finding(
                file,
                "request_dto_outside_transport_package",
                Severity::Info,
                go_struct.line,
                format!("request DTO {} lives outside a transport-oriented package", go_struct.name),
                vec!["request binding contracts are usually easier to own at the API boundary".to_string()],
            ));
        }

        if response_struct_name(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["json:"])
            && !is_transport_file(file)
        {
            findings.push(file_finding(
                file,
                "response_dto_outside_transport_package",
                Severity::Info,
                go_struct.line,
                format!(
                    "response DTO {} lives outside a transport-oriented package",
                    go_struct.name
                ),
                vec!["response contracts are usually easier to evolve when they stay at the boundary".to_string()],
            ));
        }

        if api_error_struct_name(&go_struct.name) && !is_transport_file(file) {
            findings.push(file_finding(
                file,
                "api_error_type_outside_transport_package",
                Severity::Info,
                go_struct.line,
                format!(
                    "API-facing error contract {} lives outside a transport-oriented package",
                    go_struct.name
                ),
                vec!["client error payload types usually belong with transport contracts".to_string()],
            ));
        }

        if gorm_structs.contains(&go_struct.name) && !is_model_file(file) {
            findings.push(file_finding(
                file,
                "gorm_model_outside_models_package",
                Severity::Info,
                go_struct.line,
                format!("GORM-backed struct {} lives outside a model-oriented package", go_struct.name),
                vec!["repos that already separate handlers, services, and repositories usually benefit from a model or models package for persistence structs".to_string()],
            ));
        }
    }

    for go_struct in file.go_structs() {
        if request_struct_name(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["gorm:"])
        {
            findings.push(file_finding(
                file,
                "request_dto_contains_gorm_tags",
                Severity::Warning,
                go_struct.line,
                format!("request DTO {} carries GORM tags", go_struct.name),
                vec!["request binding contracts should not double as persistence schema".to_string()],
            ));
        }

        if response_struct_name(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["gorm:"])
        {
            findings.push(file_finding(
                file,
                "response_dto_contains_gorm_tags",
                Severity::Warning,
                go_struct.line,
                format!("response DTO {} carries GORM tags", go_struct.name),
                vec!["response contracts should not leak persistence mapping tags".to_string()],
            ));
        }

        if gorm_structs.contains(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["binding:", "validate:"])
        {
            findings.push(file_finding(
                file,
                "gorm_model_contains_binding_or_validate_tags",
                Severity::Warning,
                go_struct.line,
                format!(
                    "GORM model {} carries request validation or binding tags",
                    go_struct.name
                ),
                vec!["persistence structs are usually cleaner when boundary validation stays in dedicated DTOs".to_string()],
            ));
        }

        if gorm_structs.contains(&go_struct.name)
            && has_any_tag_for_struct(&tag_map, &go_struct.name, &["form:", "uri:"])
        {
            findings.push(file_finding(
                file,
                "persistence_model_contains_http_form_or_uri_tags",
                Severity::Warning,
                go_struct.line,
                format!(
                    "GORM model {} carries form or URI binding tags",
                    go_struct.name
                ),
                vec!["persistence models should not have to understand HTTP binding details".to_string()],
            ));
        }
    }

    if is_repository_file(file)
        && file
            .imports
            .iter()
            .any(|import| import.path.contains("prometheus") || import.path.contains("otel"))
    {
        findings.push(file_finding(
            file,
            "transport_metrics_emitted_from_repository_layer",
            Severity::Info,
            import_line,
            "repository package imports transport-oriented metrics or tracing dependencies",
            vec!["repository metrics can be valid, but HTTP-route or request-surface metrics usually belong higher in the stack".to_string()],
        ));
    }

    if is_model_file(file)
        && (file.comments.iter().any(|comment| {
            let lower = comment.text.to_ascii_lowercase();
            lower.contains("@summary") || lower.contains("@router") || lower.contains("@param")
        })
            || file
                .imports
                .iter()
                .any(|import| import.path.contains("swag") || import.path.contains("/docs")))
    {
        let line = file
            .comments
            .iter()
            .find(|comment| {
                let lower = comment.text.to_ascii_lowercase();
                lower.contains("@summary") || lower.contains("@router") || lower.contains("@param")
            })
            .map(|comment| comment.line)
            .or_else(|| {
                file.imports
                    .iter()
                    .find(|import| import.path.contains("swag") || import.path.contains("/docs"))
                    .map(|import| import.line)
            })
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "swagger_or_openapi_annotations_on_persistence_models",
            Severity::Info,
            line,
            "persistence model file carries OpenAPI or Swagger annotations",
            vec!["public docs contracts usually read more clearly when they describe transport DTOs instead of persistence models".to_string()],
        ));
    }

    if (is_repository_file(file) || is_service_file(file))
        && file
            .imports
            .iter()
            .any(|import| import.path.contains("swag") || import.path.contains("/docs"))
    {
        findings.push(file_finding(
            file,
            "repository_or_service_packages_import_docs_or_generator_annotations",
            Severity::Info,
            import_line,
            "core package imports docs or generator-oriented dependencies",
            vec!["business and persistence packages usually stay cleaner when docs tooling stays at the boundary".to_string()],
        ));
    }

    if let Some(line) = is_transport_file(file)
        .then(|| first_domain_constant_line(file))
        .flatten()
    {
        findings.push(file_finding(
            file,
            "domain_constants_declared_in_handler_package",
            Severity::Info,
            line,
            "transport package declares domain status, role, or state constants",
            vec!["domain lifecycle constants are usually easier to reuse from domain-facing packages".to_string()],
        ));
    }

    if let Some(line) = (!file_matches_any_role(file, MAPPER_ROLE_HINTS)
        && import_path_has_any_role(file, REPOSITORY_ROLE_HINTS))
        .then(|| first_mapper_symbol_line(file))
        .flatten()
    {
        findings.push(file_finding(
            file,
            "mapper_outside_mapper_package_when_repo_uses_mappers",
            Severity::Info,
            line,
            "mapping helpers live outside a mapper-oriented package",
            vec!["repos that already lean on explicit mapping usually benefit from keeping those adapters together".to_string()],
        ));
    }

    if let Some(line) = (!is_router_file(file) && is_transport_file(file))
        .then(|| route_registration_line_in_file(file))
        .flatten()
    {
        findings.push(file_finding(
            file,
            "route_setup_scattered_without_router_package",
            Severity::Info,
            line,
            "route registration lives outside a router-oriented package",
            vec!["larger Gin services usually stay easier to navigate when route setup has a clear home".to_string()],
        ));
    }

    if !is_router_file(file)
        && is_transport_file(file)
        && file.functions.iter().any(|function| {
            let name = function.fingerprint.name.as_str();
            (name.contains("Route") || name.contains("Register"))
                && body_lines(function).iter().any(|line| {
                    [".GET(", ".POST(", ".PUT(", ".PATCH(", ".DELETE("]
                        .iter()
                        .any(|marker| line.text.contains(marker))
                })
        })
    {
        let line = file
            .functions
            .iter()
            .find(|function| {
                let name = function.fingerprint.name.as_str();
                (name.contains("Route") || name.contains("Register"))
                    && body_lines(function).iter().any(|line| {
                        [".GET(", ".POST(", ".PUT(", ".PATCH(", ".DELETE("]
                            .iter()
                            .any(|marker| line.text.contains(marker))
                    })
            })
            .map(|function| function.fingerprint.start_line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "route_setup_scattered_without_router_package",
            Severity::Info,
            line,
            "route registration is spread into a non-router transport file",
            vec!["larger Gin services usually benefit from giving route setup a dedicated router home".to_string()],
        ));
    }

    if !is_router_file(file)
        && is_transport_file(file)
        && file.path.to_string_lossy().to_ascii_lowercase().contains("route")
    {
        findings.push(file_finding(
            file,
            "route_setup_scattered_without_router_package",
            Severity::Info,
            1,
            "route setup lives in a non-router transport file",
            vec!["larger Gin services usually benefit from giving route setup a dedicated router home".to_string()],
        ));
    }

    if !is_router_file(file) {
        let lower_path = file.path.to_string_lossy().to_ascii_lowercase();
        if lower_path.contains("transport") && lower_path.ends_with("routes.go") {
            findings.push(file_finding(
                file,
                "route_setup_scattered_without_router_package",
                Severity::Info,
                1,
                "route setup file lives under transport instead of a router package",
                vec!["larger Gin services usually benefit from giving route setup a dedicated router home".to_string()],
            ));
        }
    }

    if let Some(line) = (is_transport_file(file) || is_service_file(file))
        .then(|| mutable_config_package_var(file))
        .flatten()
    {
        findings.push(file_finding(
            file,
            "package_level_mutable_config_used_by_handlers_services",
            Severity::Info,
            line,
            "package-level mutable config is available to request-path code",
            vec!["request-path code is usually easier to reason about when config is injected rather than read from shared mutable globals".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| {
        repeated_tagged_struct_use(&tag_map, go_struct, &["gorm:"])
            && repeated_tagged_struct_use(&tag_map, go_struct, &["json:", "binding:", "validate:"])
    }) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                repeated_tagged_struct_use(&tag_map, go_struct, &["gorm:"])
                    && repeated_tagged_struct_use(&tag_map, go_struct, &["json:", "binding:", "validate:"])
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "same_struct_used_for_bind_persist_and_respond",
            Severity::Warning,
            line,
            "one struct appears to serve transport, validation, and persistence concerns at once",
            vec!["shared bind-persist-respond structs usually make contracts harder to evolve".to_string()],
        ));
    }

    if file_matches_any_role(file, &["entity", "domain"])
        && file.go_structs().iter().any(|go_struct| {
            repeated_tagged_struct_use(&tag_map, go_struct, &["json:"])
                && !request_struct_name(&go_struct.name)
                && !response_struct_name(&go_struct.name)
        })
    {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                repeated_tagged_struct_use(&tag_map, go_struct, &["json:"])
                    && !request_struct_name(&go_struct.name)
                    && !response_struct_name(&go_struct.name)
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "domain_entity_contains_json_tags_without_boundary_exception",
            Severity::Info,
            line,
            "domain or entity struct carries JSON transport tags",
            vec!["core entities are usually cleaner when serialization concerns stay at the boundary".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| {
        (request_struct_name(&go_struct.name) || response_struct_name(&go_struct.name))
            && struct_has_sql_null_field(go_struct)
    }) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                (request_struct_name(&go_struct.name) || response_struct_name(&go_struct.name))
                    && struct_has_sql_null_field(go_struct)
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "nullable_sql_types_leak_into_api_dto",
            Severity::Info,
            line,
            "API DTO exposes sql.Null* field types",
            vec!["boundary-facing contracts usually read better with transport-friendly optional types".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| first_non_pointer_patch_field(go_struct).is_some()) {
        let line = file
            .go_structs()
            .iter()
            .find_map(first_non_pointer_patch_field)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "patch_dto_uses_non_pointer_fields_for_optional_updates",
            Severity::Warning,
            line,
            "PATCH-like DTO uses non-pointer scalar fields",
            vec!["partial update contracts usually need pointer or explicit field-intent semantics to distinguish omitted from zero".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| {
        (go_struct.name.contains("CreateUpdate") || go_struct.name.contains("Upsert"))
            && repeated_tagged_struct_use(&tag_map, go_struct, &["binding:"])
    }) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                (go_struct.name.contains("CreateUpdate") || go_struct.name.contains("Upsert"))
                    && repeated_tagged_struct_use(&tag_map, go_struct, &["binding:"])
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "create_and_update_share_same_dto_despite_conflicting_requiredness",
            Severity::Info,
            line,
            "single DTO appears to cover both create and update semantics",
            vec!["create and update contracts often diverge on requiredness and are usually clearer as separate types".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| {
        response_struct_name(&go_struct.name) && repeated_tagged_struct_use(&tag_map, go_struct, &[",omitempty"])
    }) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| {
                response_struct_name(&go_struct.name)
                    && repeated_tagged_struct_use(&tag_map, go_struct, &[",omitempty"])
            })
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "response_dto_uses_omitempty_on_required_contract_fields",
            Severity::Info,
            line,
            "response DTO uses omitempty on contract fields",
            vec!["stable API contracts usually read more clearly when required fields are always present".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| gorm_structs.contains(&go_struct.name) && model_has_calculated_field(go_struct).is_some()) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| gorm_structs.contains(&go_struct.name) && model_has_calculated_field(go_struct).is_some())
            .and_then(model_has_calculated_field)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "gorm_model_contains_calculated_response_fields",
            Severity::Info,
            line,
            "GORM model carries calculated presentation-oriented fields",
            vec!["API presentation fields are usually easier to evolve in DTOs or mappers than in persistence structs".to_string()],
        ));
    }

    if file.go_structs().iter().any(|go_struct| gorm_structs.contains(&go_struct.name) && model_spans_multiple_subdomains(go_struct)) {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| gorm_structs.contains(&go_struct.name) && model_spans_multiple_subdomains(go_struct))
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "giant_model_struct_spans_multiple_subdomains",
            Severity::Info,
            line,
            "large persistence model appears to mix several subdomain concerns",
            vec!["splitting oversized models often makes ownership boundaries easier to maintain".to_string()],
        ));
    }

    if is_repository_file(file)
        && file.package_vars().iter().any(|var| {
            var.type_text
                .as_deref()
                .is_some_and(|text| text.contains("*gorm.DB"))
                && var.value_text
                    .as_deref()
                    .is_some_and(|value| value.contains(".Session(") || value.contains(".WithContext("))
        })
    {
        let line = file
            .package_vars()
            .iter()
            .find(|var| {
                var.type_text
                    .as_deref()
                    .is_some_and(|text| text.contains("*gorm.DB"))
                    && var.value_text
                        .as_deref()
                        .is_some_and(|value| value.contains(".Session(") || value.contains(".WithContext("))
            })
            .map(|var| var.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "shared_gorm_db_state_mutated_and_reused_across_requests",
            Severity::Warning,
            line,
            "repository package mutates and stores shared *gorm.DB state",
            vec!["shared mutable GORM handles can make request-path behavior harder to predict".to_string()],
        ));
    }

    if file.functions.iter().filter(|function| function.fingerprint.name == "TableName").count() >= 2 {
        let line = file
            .functions
            .iter()
            .find(|function| function.fingerprint.name == "TableName")
            .map(|function| function.fingerprint.start_line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "table_name_override_or_scope_logic_duplicated_across_models",
            Severity::Info,
            line,
            "several TableName overrides live together in one model file",
            vec!["repeated model-level table customization can be a sign that a shared convention is missing".to_string()],
        ));
    }

    if !file.is_test_file {
        for function in file.functions.iter().filter(|function| !function.is_test_function) {
            if test_fixture_builder_line(function).is_some() {
                findings.push(file_finding(
                    file,
                    "test_fixture_builders_live_in_production_packages",
                    Severity::Info,
                    function.fingerprint.start_line,
                    "production file exports test fixture builder helpers",
                    vec!["fixture and test factory helpers are usually clearer when they stay in test-only packages".to_string()],
                ));
                break;
            }
        }
    }

    if !file.is_test_file
        && file
            .imports
            .iter()
            .any(|import| import.path.contains("/test") || import.path.contains("testsupport") || import.path.contains("/fixtures"))
    {
        findings.push(file_finding(
            file,
            "test_bootstrap_package_reused_by_production_wiring",
            Severity::Info,
            import_line,
            "production file imports test-only bootstrap or fixture helpers",
            vec!["production wiring usually stays easier to govern when it does not depend on test support packages".to_string()],
        ));
    }

    let handler_functions = file
        .functions
        .iter()
        .filter(|function| is_request_path_function(file, function))
        .collect::<Vec<_>>();
    let middleware_functions = file
        .functions
        .iter()
        .filter(|function| {
            let lower = function.fingerprint.name.to_ascii_lowercase();
            lower.contains("middleware") || lower.contains("auth") || lower.contains("recover")
        })
        .collect::<Vec<_>>();

    if handler_functions.iter().filter(|function| handler_recovery_line(function).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| handler_recovery_line(function))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "custom_recovery_logic_repeated_across_handlers",
            Severity::Info,
            line,
            "several handlers implement custom recover logic inline",
            vec!["panic-to-response translation is usually easier to keep consistent in middleware".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| request_identity_extraction_line(&body_lines(function)).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| request_identity_extraction_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "auth_or_tenant_extraction_duplicated_across_handlers",
            Severity::Info,
            line,
            "auth or tenant extraction is duplicated across handlers",
            vec!["shared boundary helpers or middleware usually keep request identity extraction more consistent".to_string()],
        ));
        findings.push(file_finding(
            file,
            "request_context_value_extraction_duplicated_across_handlers",
            Severity::Info,
            line,
            "request context value extraction is duplicated across handlers",
            vec!["shared helpers often reduce drift in user, tenant, and locale extraction logic".to_string()],
        ));
    }

    if !is_middleware_file(file)
        && handler_functions.iter().filter(|function| request_id_generation_line(&body_lines(function)).is_some()).count() >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| request_id_generation_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "request_id_generation_duplicated_outside_middleware",
            Severity::Info,
            line,
            "request-id generation is duplicated outside middleware",
            vec!["request correlation IDs are usually easier to keep consistent in middleware".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| pagination_binding_line(&body_lines(function)).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| pagination_binding_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "pagination_binding_duplicated_outside_boundary_helper",
            Severity::Info,
            line,
            "pagination binding logic is duplicated across handlers",
            vec!["shared boundary helpers usually reduce drift in paging defaults and parsing".to_string()],
        ));
    }

    if middleware_functions.iter().any(|function| !body_lines(function).is_empty() && function.go_evidence().gin_calls.iter().any(|call| is_body_bind_operation(&call.operation)))
        && handler_functions.iter().any(|function| function.go_evidence().gin_calls.iter().any(|call| is_body_bind_operation(&call.operation)))
    {
        let line = middleware_functions
            .iter()
            .find_map(|function| body_lines(function).first().map(|line| line.line))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "body_binding_done_in_middleware_and_handler",
            Severity::Info,
            line,
            "request body binding happens in both middleware and handlers",
            vec!["ownership of request parsing is usually clearer when one boundary layer owns body binding".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| response_envelope_line(&body_lines(function)).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| response_envelope_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "response_envelope_shaping_duplicated_across_handlers",
            Severity::Info,
            line,
            "response envelope shaping is duplicated across handlers",
            vec!["shared render helpers often keep transport envelopes more consistent".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| required_validation_line(&body_lines(function)).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| required_validation_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "validation_logic_duplicated_across_handlers",
            Severity::Info,
            line,
            "manual validation checks are duplicated across handlers",
            vec!["shared validators or DTO-based validation usually reduce repeated required-field checks".to_string()],
        ));
    }

    let validation_shapes = handler_functions
        .iter()
        .filter_map(|function| validation_error_shape(&body_lines(function)))
        .collect::<BTreeSet<_>>();
    if validation_shapes.len() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| validation_error_shape(&body_lines(function)).and(Some(function.fingerprint.start_line)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "validation_error_response_shape_inconsistent",
            Severity::Info,
            line,
            "validation failures serialize several different payload shapes in one module",
            vec!["consistent validation error envelopes are usually easier for clients to consume".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| default_injection_line(&body_lines(function)).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| default_injection_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "default_value_injection_scattered_across_handlers",
            Severity::Info,
            line,
            "default value injection is duplicated across handlers",
            vec!["shared contracts or bind helpers usually keep defaults more consistent".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| path_param_parse_line(&body_lines(function)).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| path_param_parse_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "path_param_parsing_duplicated_across_handlers",
            Severity::Info,
            line,
            "path parameter parsing is duplicated across handlers",
            vec!["shared param helpers usually reduce repeated strconv or UUID parsing boilerplate".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| pagination_bounds_line(&body_lines(function)).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| pagination_bounds_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "pagination_validation_missing_shared_bound_helper",
            Severity::Info,
            line,
            "page or page-size bound checks are repeated across handlers",
            vec!["shared pagination helpers usually keep request bounds consistent".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| sort_whitelist_line(&body_lines(function)).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| sort_whitelist_line(&body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "sort_or_filter_whitelist_logic_duplicated_across_handlers",
            Severity::Info,
            line,
            "sort or filter whitelist logic is duplicated across handlers",
            vec!["shared contract helpers often reduce drift in allowed client fields".to_string()],
        ));
    }

    let error_shapes = handler_functions
        .iter()
        .filter_map(|function| handler_error_shape_key(&body_lines(function)))
        .collect::<BTreeSet<_>>();
    if error_shapes.len() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| handler_error_shape_key(&body_lines(function)).and(Some(function.fingerprint.start_line)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "multiple_error_envelope_shapes_same_api_module",
            Severity::Info,
            line,
            "handlers in the same module use several error envelope shapes",
            vec!["consistent error contracts are usually easier for clients to depend on".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| error_mapping_line(file, &body_lines(function)).is_some()).count() >= 2 {
        let line = handler_functions
            .iter()
            .find_map(|function| error_mapping_line(file, &body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "inline_error_to_status_mapping_duplicated",
            Severity::Info,
            line,
            "error-to-status translation is duplicated across handlers",
            vec!["centralizing transport error translation usually reduces drift".to_string()],
        ));
    }

    if let Some((line, code)) = is_transport_file(file)
        .then(|| {
            repeated_named_literals(file, |literal| {
                literal.name.to_ascii_lowercase().contains("code")
                    && literal
                        .value
                        .chars()
                        .all(|ch: char| ch.is_ascii_uppercase() || ch == '_' || ch == '-')
            })
        })
        .flatten()
    {
        findings.push(file_finding(
            file,
            "error_code_literals_duplicated_across_handlers",
            Severity::Info,
            line,
            format!("error code literal {code} is repeated across transport helpers"),
            vec!["shared typed error code catalogs usually make API errors easier to govern".to_string()],
        ));
    }

    if is_transport_file(file)
        && file
            .functions
            .iter()
            .flat_map(body_lines)
            .filter(|line| line.text.contains("\"code\"") && line.text.contains("USER_"))
            .count()
            >= 2
    {
        let line = file
            .functions
            .iter()
            .flat_map(body_lines)
            .find(|line| line.text.contains("\"code\"") && line.text.contains("USER_"))
            .map(|line| line.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "error_code_literals_duplicated_across_handlers",
            Severity::Info,
            line,
            "string error codes are duplicated across transport handlers",
            vec!["shared typed error code catalogs usually make API errors easier to govern".to_string()],
        ));
    }

    if let Some((line, value)) = (!is_repository_file(file))
        .then(|| {
            repeated_named_literals(file, |literal| {
                let value = literal.value.as_str();
                table_or_column_literal(value) && !(value.ends_with("_id") || value.ends_with("_at"))
            })
        })
        .flatten()
    {
        findings.push(file_finding(
            file,
            "table_name_literals_duplicated_outside_repository",
            Severity::Info,
            line,
            format!("SQL-oriented literal {value} is duplicated outside repository packages"),
            vec!["query-shape literals usually stay easier to govern when owned by repository code".to_string()],
        ));
    }

    if let Some((line, value)) = (!is_repository_file(file))
        .then(|| {
            repeated_named_literals(file, |literal| {
                let value = literal.value.as_str();
                table_or_column_literal(value) && (value.ends_with("_id") || value.ends_with("_at"))
            })
        })
        .flatten()
    {
        findings.push(file_finding(
            file,
            "column_name_literals_duplicated_outside_repository",
            Severity::Info,
            line,
            format!("SQL column literal {value} is duplicated outside repository packages"),
            vec!["column literals usually stay easier to govern when owned by repository code".to_string()],
        ));
    }

    if handler_functions.iter().filter(|function| {
        let lines = body_lines(function);
        let has_mapping = lines.iter().any(|line| line.text.contains("Response{") || line.text.contains("gin.H{"));
        let has_model_field = lines.iter().any(|line| line.text.contains(".ID") || line.text.contains(".Name") || line.text.contains(".Email"));
        has_mapping && has_model_field
    }).count() >= 2
    {
        let line = handler_functions
            .iter()
            .find_map(|function| {
                let lines = body_lines(function);
                let has_mapping = lines.iter().any(|line| line.text.contains("Response{") || line.text.contains("gin.H{"));
                let has_model_field = lines.iter().any(|line| line.text.contains(".ID") || line.text.contains(".Name") || line.text.contains(".Email"));
                if has_mapping && has_model_field {
                    Some(function.fingerprint.start_line)
                } else {
                    None
                }
            })
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "mapping_logic_duplicated_across_handlers",
            Severity::Info,
            line,
            "similar transport mapping logic is duplicated across handlers",
            vec!["shared mappers or render helpers usually reduce repeated field-by-field response shaping".to_string()],
        ));
    }

    if is_transport_file(file)
        && file.go_structs().iter().any(|go_struct| go_struct.name.ends_with("Error"))
    {
        let line = file
            .go_structs()
            .iter()
            .find(|go_struct| go_struct.name.ends_with("Error"))
            .map(|go_struct| go_struct.line)
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "domain_errors_declared_in_handler_package",
            Severity::Info,
            line,
            "handler package declares domain-like error types",
            vec!["reusable business errors are usually clearer in domain or service-facing packages".to_string()],
        ));
    }

    if middleware_functions.iter().any(|function| error_mapping_line(file, &body_lines(function)).is_some())
        && handler_functions.iter().any(|function| error_mapping_line(file, &body_lines(function)).is_some())
    {
        let line = middleware_functions
            .iter()
            .find_map(|function| error_mapping_line(file, &body_lines(function)))
            .unwrap_or(1);
        findings.push(file_finding(
            file,
            "middleware_and_handler_translate_same_error_domain",
            Severity::Info,
            line,
            "middleware and handlers both translate the same error family",
            vec!["keeping transport error translation in one layer usually reduces double-mapping drift".to_string()],
        ));
    }

    let mut error_statuses = BTreeMap::<String, BTreeSet<String>>::new();
    for function in &handler_functions {
        let lines = body_lines(function);
        let statuses = lines
            .iter()
            .filter_map(|line| extract_status_constant(&line.text))
            .collect::<BTreeSet<_>>();
        for error_name in lines.iter().filter_map(|line| extract_domain_error_name(&line.text)) {
            error_statuses.entry(error_name).or_default().extend(statuses.clone());
        }
    }
    if let Some((error_name, statuses)) = error_statuses.iter().find(|(_, statuses)| statuses.len() >= 2) {
        let line = handler_functions.first().map(|function| function.fingerprint.start_line).unwrap_or(1);
        findings.push(file_finding(
            file,
            "same_domain_error_mapped_to_multiple_statuses",
            Severity::Info,
            line,
            format!("domain error {error_name} is translated to multiple statuses"),
            vec![format!("observed statuses: {}", statuses.iter().cloned().collect::<Vec<_>>().join(", "))],
        ));
    }

    let trace_spans = handler_functions
        .iter()
        .flat_map(|function| body_lines(function))
        .filter_map(|line| tracing_span_name_literal(&line.text))
        .collect::<Vec<_>>();
    if has_duplicate_string(&trace_spans) {
        let line = handler_functions.first().map(|function| function.fingerprint.start_line).unwrap_or(1);
        findings.push(file_finding(
            file,
            "tracing_span_names_duplicated_as_raw_strings",
            Severity::Info,
            line,
            "raw tracing span names are duplicated across handlers",
            vec!["shared span helpers or constants usually reduce string drift in tracing".to_string()],
        ));
    }

    let logging_keys = handler_functions
        .iter()
        .flat_map(|function| body_lines(function))
        .filter_map(|line| request_logging_field_key(&line.text))
        .collect::<BTreeSet<_>>();
    if logging_keys.contains("request_id") && logging_keys.contains("requestId") {
        let line = handler_functions.first().map(|function| function.fingerprint.start_line).unwrap_or(1);
        findings.push(file_finding(
            file,
            "request_logging_fields_assembled_differently_across_handlers",
            Severity::Info,
            line,
            "request log field names differ across handlers",
            vec!["shared request logging helpers usually reduce field-name drift across an API module".to_string()],
        ));
    }

    if logging_keys.contains("user_id") && logging_keys.contains("userId") {
        let line = handler_functions.first().map(|function| function.fingerprint.start_line).unwrap_or(1);
        findings.push(file_finding(
            file,
            "domain_identifiers_logged_under_inconsistent_field_keys",
            Severity::Info,
            line,
            "same domain identifier is logged under inconsistent key names",
            vec!["stable logging keys usually make operational queries easier than mixed snake_case and camelCase identifiers".to_string()],
        ));
    }

    findings
}

fn function_architecture_findings(file: &ParsedFile, function: &ParsedFunction) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines = body_lines(function);
    let go = function.go_evidence();
    let tag_map = struct_tag_map(file);
    let gorm_structs = gorm_struct_names(file, &tag_map);
    let returns_text = signature_returns_text(&function.signature_text);
    let params_text = signature_params_text(&function.signature_text);

    let is_handler = is_request_path_function(file, function);
    let is_service = is_service_file(file);
    let is_repository = is_repository_file(file);

    if is_handler && references_repository_directly(&lines) && !references_service_directly(&lines) {
        findings.push(function_finding(
            file,
            function,
            "handler_calls_repository_directly_without_service",
            Severity::Warning,
            function.fingerprint.start_line,
            "request handler reaches repository-like dependencies directly without an obvious service seam",
            vec!["direct repository orchestration from handlers often signals missing service-layer ownership".to_string()],
        ));
    }

    if is_handler && !go.gorm_query_chains.is_empty() {
        let line = go.gorm_query_chains[0].line;
        findings.push(function_finding(
            file,
            function,
            "handler_calls_gorm_directly_outside_repository",
            Severity::Warning,
            line,
            "request handler builds GORM query chains directly",
            vec!["handlers usually read more clearly when GORM query shape lives in repository methods".to_string()],
        ));
        findings.push(function_finding(
            file,
            function,
            "gorm_chain_built_in_handler",
            Severity::Warning,
            line,
            "request handler owns GORM query construction directly",
            vec!["query-shape ownership is usually easier to test and reuse inside repositories".to_string()],
        ));
    }

    if is_handler && !go.db_query_calls.is_empty() && has_sql_like_import(file) {
        let line = go.db_query_calls[0].line;
        findings.push(function_finding(
            file,
            function,
            "handler_calls_database_sql_directly_outside_repository",
            Severity::Warning,
            line,
            "request handler executes SQL-oriented calls directly",
            vec!["handlers usually should not own driver-level query execution".to_string()],
        ));
    }

    if is_gin_handler(file, function) {
        let body_bind_count = go
            .gin_calls
            .iter()
            .filter(|call| is_body_bind_operation(&call.operation))
            .count();
        if body_bind_count >= 2 {
            let line = go
                .gin_calls
                .iter()
                .filter(|call| is_body_bind_operation(&call.operation))
                .nth(1)
                .map(|call| call.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "gin_handler_accepts_more_than_one_body_contract",
                Severity::Info,
                line,
                "Gin handler appears to bind more than one body contract",
                vec!["one request path usually benefits from one clear body contract".to_string()],
            ));
        }

        for gin_call in go.gin_calls {
            let bind_targets_model = gin_call
                .assigned_binding
                .as_deref()
                .is_some_and(|binding| binding_looks_like_model(file, &lines, &gorm_structs, binding))
                || gin_call.argument_texts.iter().any(|argument| {
                    argument
                        .trim()
                        .strip_prefix('&')
                        .is_some_and(|binding| binding_looks_like_model(file, &lines, &gorm_structs, binding))
                        || expression_looks_like_model(file, &lines, &gorm_structs, argument)
                });

            if is_body_bind_operation(&gin_call.operation) && bind_targets_model {
                findings.push(function_finding(
                    file,
                    function,
                    "gin_handler_binds_directly_into_model",
                    Severity::Warning,
                    gin_call.line,
                    "Gin handler binds request input directly into a persistence model",
                    vec!["binding into GORM-backed structs couples transport validation to persistence shape".to_string()],
                ));
                findings.push(function_finding(
                    file,
                    function,
                    "dto_to_model_mapping_in_handler",
                    Severity::Info,
                    gin_call.line,
                    "handler maps request input straight into persistence model state",
                    vec!["transport-to-model mapping is usually easier to evolve behind mappers or service inputs".to_string()],
                ));
            }

            if matches!(gin_call.operation.as_str(), "json" | "pure_json")
                && gin_call
                    .argument_texts
                    .last()
                    .is_some_and(|arg| expression_looks_like_model(file, &lines, &gorm_structs, arg))
            {
                findings.push(function_finding(
                    file,
                    function,
                    "gin_handler_returns_persistence_model_directly",
                    Severity::Warning,
                    gin_call.line,
                    "Gin handler serializes a persistence model directly",
                    vec!["response DTOs are usually easier to evolve than exposing database model shape".to_string()],
                ));
                findings.push(function_finding(
                    file,
                    function,
                    "model_to_dto_mapping_in_handler",
                    Severity::Info,
                    gin_call.line,
                    "handler shapes client response directly from persistence model data",
                    vec!["model-to-response mapping is usually easier to reuse in mapper or renderer helpers".to_string()],
                ));
            }
        }

        if multiple_response_shapes(go.gin_calls) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_returns_multiple_response_shapes",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler mixes several response rendering styles",
                vec!["handlers that return multiple unrelated response shapes are often trying to cover too many behaviors".to_string()],
            ));
        }
        if !multiple_response_shapes(go.gin_calls)
            && lines.iter().any(|line| line.text.contains(".JSON("))
            && (lines.iter().any(|line| line.text.contains(".HTML("))
                || lines.iter().any(|line| line.text.contains(".File("))
                || lines.iter().any(|line| line.text.contains(".Data(")))
        {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_returns_multiple_response_shapes",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler mixes several response rendering styles",
                vec!["handlers that return multiple unrelated response shapes are often trying to cover too many behaviors".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "gin_handler_mixes_html_json_and_file_responses",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler mixes HTML, JSON, or file-style responses",
                vec!["combining several transport modes in one handler often hurts clarity".to_string()],
            ));
        }

        if let Some(query_call) = go.db_query_calls.iter().find(|call| call.query_text.is_some()) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_executes_raw_sql_inline",
                Severity::Warning,
                query_call.line,
                "Gin handler executes literal SQL inline",
                vec!["request handlers usually should not own raw SQL strings".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "raw_sql_literal_in_handler",
                Severity::Warning,
                query_call.line,
                "request handler owns a literal SQL statement",
                query_call
                    .query_text
                    .as_ref()
                    .map(|query| vec![format!("literal query: {query}")])
                    .unwrap_or_default(),
            ));
        }

        if let Some(line) = required_validation_line(&lines)
            && (go.gin_calls.iter().any(|call| {
                is_body_bind_operation(&call.operation)
                    && call.assigned_binding.as_deref().is_some_and(|binding| {
                        file.go_structs().iter().any(|go_struct| {
                            go_struct.name == binding
                                && repeated_tagged_struct_use(&tag_map, go_struct, &["binding:", "validate:"])
                        })
                    })
            }) || file.go_structs().iter().any(|go_struct| {
                request_struct_name(&go_struct.name)
                    && repeated_tagged_struct_use(&tag_map, go_struct, &["binding:", "validate:"])
            }))
        {
            findings.push(function_finding(
                file,
                function,
                "manual_required_checks_after_validate_tags_available",
                Severity::Info,
                line,
                "handler manually re-checks required fields after binding into a validated DTO",
                vec!["boundary validation is usually clearer when one validation mechanism owns required-field checks".to_string()],
            ));
        }

        if lines.iter().any(|line| line.text.contains("validator.New(") || line.text.contains("RegisterValidation(")) {
            let line = lines
                .iter()
                .find(|line| line.text.contains("validator.New(") || line.text.contains("RegisterValidation("))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "custom_validator_registration_inside_handler",
                Severity::Info,
                line,
                "handler registers validators inline",
                vec!["validator registration is usually easier to govern in startup wiring than per-handler code".to_string()],
            ));
        }

        if repository_receiver_count(&lines) >= 2 {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_calls_multiple_repositories_directly",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler coordinates multiple repository-like dependencies directly",
                vec!["this usually signals missing service orchestration or an overgrown handler".to_string()],
            ));
        }

        if let Some(line) = first_config_lookup_line(file, &lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_parses_config_or_feature_flags_inline",
                Severity::Info,
                line,
                "Gin handler reads configuration or feature flags inline",
                vec!["boundary handlers are usually easier to test when configuration arrives through dependencies instead of direct lookups".to_string()],
            ));
        }

        if lines.iter().any(|line| transaction_start_line(&line.text)) {
            let line = lines
                .iter()
                .find(|line| transaction_start_line(&line.text))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "gin_handler_starts_transaction_inline",
                Severity::Warning,
                line,
                "Gin handler starts a transaction inline",
                vec!["transport handlers usually should not own transaction lifetime".to_string()],
            ));
        }

        if let Some(line) = authorization_business_logic_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_runs_authorization_business_rules_inline",
                Severity::Info,
                line,
                "Gin handler appears to mix authorization policy with request orchestration",
                vec!["permission matrix logic is often easier to reuse in dedicated policy or service code".to_string()],
            ));
        }

        if let Some(line) = action_switch_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_uses_action_param_switch_for_many_use_cases",
                Severity::Info,
                line,
                "Gin handler branches on action or mode parameters for multiple workflows",
                vec!["one endpoint function usually reads better when it serves one core use case".to_string()],
            ));
        }

        if mixes_html_json_and_file(go.gin_calls) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_mixes_html_json_and_file_responses",
                Severity::Info,
                function.fingerprint.start_line,
                "Gin handler mixes HTML, JSON, or file-style responses",
                vec!["combining several transport modes in one handler often hurts clarity".to_string()],
            ));
        }

        if let Some(line) = passes_gin_context_beyond_boundary(&lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_context_passed_beyond_request_boundary",
                Severity::Warning,
                line,
                "Gin handler passes *gin.Context into deeper layers or goroutines",
                vec!["transport-specific request context is usually better translated into context.Context and typed values".to_string()],
            ));
        }

        if let Some(line) = global_singleton_reference_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_uses_global_singletons",
                Severity::Info,
                line,
                "Gin handler reaches for singleton-like global dependencies",
                vec!["dependency injection usually keeps handlers easier to test than global runtime state".to_string()],
            ));
        }

        if let Some(line) = retry_or_backoff_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "gin_handler_contains_retry_or_backoff_orchestration",
                Severity::Info,
                line,
                "Gin handler contains retry or backoff orchestration",
                vec!["retry policy is often easier to own in services or outbound client layers".to_string()],
            ));
        }

        if lines.iter().any(|line| line.text.contains(".Group(\"/")) {
            let line = lines
                .iter()
                .find(|line| line.text.contains(".Group(\"/"))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "route_groups_created_inside_handlers",
                Severity::Info,
                line,
                "handler creates route groups instead of only serving requests",
                vec!["route-group ownership is usually clearer in router setup code".to_string()],
            ));
        }

        if let Some(line) = route_param_merge_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "multiple_bind_sources_into_same_struct_without_precedence_contract",
                Severity::Info,
                line,
                "handler merges several request sources into one DTO inline",
                vec!["normalizing request precedence before service calls usually reduces ambiguity".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "query_path_and_body_merge_performed_in_handler",
                Severity::Info,
                line,
                "handler merges path, query, or body values inline",
                vec!["shared bind helpers or boundary mappers usually keep merged request inputs clearer".to_string()],
            ));
        }

        if let Some(line) = required_validation_line(&lines)
            && (!go.db_query_calls.is_empty() || !go.gorm_query_chains.is_empty() || references_repository_directly(&lines))
        {
            findings.push(function_finding(
                file,
                function,
                "business_validation_mixed_with_persistence_calls_in_handler",
                Severity::Info,
                line,
                "handler interleaves validation logic with persistence or repository work",
                vec!["handlers usually read more clearly when request validation finishes before persistence begins".to_string()],
            ));
        }

        if let Some(line) = upload_write_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "file_upload_validation_mixed_with_storage_write",
                Severity::Info,
                line,
                "handler validates upload input and writes storage in the same step",
                vec!["splitting upload validation from storage writes usually makes the boundary easier to test".to_string()],
            ));
        }

        if let Some(line) = route_param_drift_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "route_param_name_and_dto_field_name_drift_without_mapping_helper",
                Severity::Info,
                line,
                "handler manually remaps route parameter names into DTO fields",
                vec!["shared mappers usually reduce repeated renaming between route params and request contracts".to_string()],
            ));
        }

        if let Some(line) = error_string_switch_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "handler_switches_on_error_strings",
                Severity::Warning,
                line,
                "handler branches on err.Error() text",
                vec!["typed errors are usually more stable than string-matching on error messages".to_string()],
            ));
        }

        if let Some(line) = raw_db_error_response_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "raw_db_error_exposed_to_client",
                Severity::Warning,
                line,
                "handler appears to serialize raw error text to clients",
                vec!["raw SQL or ORM error messages often leak implementation details into public contracts".to_string()],
            ));
        }

        if lines.iter().any(|line| line.text.contains("\"code\"")) && !import_path_has_any_role(file, &["code", "codes"]) {
            let line = lines
                .iter()
                .find(|line| line.text.contains("\"code\""))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "transport_layer_uses_untyped_string_codes_without_catalog",
                Severity::Info,
                line,
                "transport layer builds string error codes inline",
                vec!["typed or cataloged error codes are usually easier to govern than free-form strings".to_string()],
            ));
        }

        if let Some(line) = success_payload_with_error_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "success_response_contains_error_field_or_mixed_contract",
                Severity::Info,
                line,
                "success payload mixes data and error fields",
                vec!["response contracts are usually clearer when success and error shapes remain distinct".to_string()],
            ));
        }

        if let Some(line) = health_handler_repo_line(function, &lines) {
            findings.push(function_finding(
                file,
                function,
                "health_or_readiness_handlers_reach_into_business_repositories_directly",
                Severity::Info,
                line,
                "health or readiness handler queries business repositories directly",
                vec!["operational probes usually stay more stable when they depend on focused health checks instead of full business repositories".to_string()],
            ));
        }

        if let Some(line) = migration_or_seed_handler_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "migration_or_seed_logic_callable_from_request_handlers",
                Severity::Warning,
                line,
                "handler can trigger migration or seed logic",
                vec!["schema or seed workflows are usually safer when kept out of request handlers".to_string()],
            ));
        }

        if let Some(line) = api_example_literal_line(function) {
            findings.push(function_finding(
                file,
                function,
                "api_examples_embedded_in_handlers_instead_of_transport_docs_helpers",
                Severity::Info,
                line,
                "handler embeds example JSON payload text inline",
                vec!["transport docs helpers usually keep examples easier to reuse than hardcoded handler examples".to_string()],
            ));
        }
    }

    if is_service {
        if signature_has_request_dto(params_text) {
            findings.push(function_finding(
                file,
                function,
                "service_depends_on_transport_request_type",
                Severity::Warning,
                function.fingerprint.start_line,
                "service method accepts request-shaped transport DTO input",
                vec!["service APIs usually stay clearer when transport DTOs are translated before crossing the boundary".to_string()],
            ));
        }

        if function.signature_text.contains("*gin.Context") {
            findings.push(function_finding(
                file,
                function,
                "service_method_accepts_gin_context",
                Severity::Warning,
                function.fingerprint.start_line,
                "service method accepts *gin.Context directly",
                vec!["service methods usually prefer context.Context plus typed inputs".to_string()],
            ));
        }

        if has_http_status_usage(file, &lines) && returns_text.contains("int") {
            let line = first_http_status_line(file, &lines).unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "service_method_returns_http_status",
                Severity::Warning,
                line,
                "service method appears to own HTTP status return semantics",
                vec!["transport status translation usually belongs at the boundary layer".to_string()],
            ));
        }

        if returns_text.contains("gin.H") || returns_text.contains("http.ResponseWriter") {
            findings.push(function_finding(
                file,
                function,
                "service_method_returns_transport_writer_or_gin_h",
                Severity::Warning,
                function.fingerprint.start_line,
                "service method returns transport-facing response types",
                vec!["services usually should return domain results or DTO-ready values, not transport writers".to_string()],
            ));
        }

        if returns_framework_builder(returns_text) {
            findings.push(function_finding(
                file,
                function,
                "service_method_returns_gorm_db_or_sql_rows",
                Severity::Warning,
                function.fingerprint.start_line,
                "service method leaks ORM or SQL primitives to callers",
                vec!["service callers usually should not need *gorm.DB or row handles".to_string()],
            ));
        }

        if params_text.contains("map[string]any") || params_text.contains("map[string]interface{}") {
            findings.push(function_finding(
                file,
                function,
                "service_method_accepts_map_string_any_input",
                Severity::Info,
                function.fingerprint.start_line,
                "service method accepts map[string]any input",
                vec!["typed inputs usually age better than open-ended map payloads".to_string()],
            ));
        }

        if returns_text.contains("map[string]any") || returns_text.contains("map[string]interface{}") {
            findings.push(function_finding(
                file,
                function,
                "service_method_returns_map_string_any_output",
                Severity::Info,
                function.fingerprint.start_line,
                "service method returns map[string]any output",
                vec!["typed results are usually easier to validate and document".to_string()],
            ));
        }

        if let Some(line) = first_http_or_abort_semantics_line(file, &lines) {
            findings.push(function_finding(
                file,
                function,
                "service_method_imports_http_status_or_abort_semantics",
                Severity::Warning,
                line,
                "service method owns transport abort or HTTP status semantics",
                vec!["services usually should return typed outcomes while handlers translate them into HTTP responses".to_string()],
            ));
        }

        if let Some(query_call) = go.db_query_calls.iter().find(|call| call.query_text.is_some()) {
            findings.push(function_finding(
                file,
                function,
                "raw_sql_literal_in_service",
                Severity::Warning,
                query_call.line,
                "service method executes literal SQL directly",
                query_call
                    .query_text
                    .as_ref()
                    .map(|query| vec![format!("literal query: {query}")])
                    .unwrap_or_default(),
            ));
        }

        if let Some(chain) = go.gorm_query_chains.first() {
            findings.push(function_finding(
                file,
                function,
                "gorm_chain_built_in_service",
                Severity::Warning,
                chain.line,
                "service method owns GORM query construction directly",
                vec!["query-chain ownership often reads better inside repositories".to_string()],
            ));
        }

        if constructor_like(function) && constructor_instantiates_dependencies(&lines) {
            let line = lines
                .iter()
                .find(|line| constructor_dependency_line(&line.text))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "service_constructor_instantiates_dependencies_internally",
                Severity::Info,
                line,
                "service constructor instantiates concrete dependencies internally",
                vec!["accepting dependencies explicitly usually gives clearer wiring and easier tests".to_string()],
            ));
        }

        if let Some(service_struct) = file
            .go_structs()
            .iter()
            .find(|s| s.name.ends_with("Service") && s.fields.len() >= 5)
        {
            findings.push(file_finding(
                file,
                "service_struct_has_excessive_dependency_count",
                Severity::Info,
                service_struct.line,
                format!("service struct {} has many dependencies", service_struct.name),
                vec![format!("{} fields were observed on the service struct", service_struct.fields.len())],
            ));
        }

        if let Some(line) = pagination_or_query_parsing_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "service_method_handles_pagination_or_query_parsing",
                Severity::Info,
                line,
                "service method appears to parse pagination or query string details",
                vec!["query and page normalization is often cleaner at the transport boundary".to_string()],
            ));
        }

        if let Some(line) = request_binding_or_header_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "service_method_handles_request_binding_or_header_extraction",
                Severity::Info,
                line,
                "service method extracts request-boundary values directly",
                vec!["headers, params, and forms usually belong at the transport boundary".to_string()],
            ));
        }

        if let Some(line) = action_switch_line(&lines)
            && lines.iter().any(|line| {
                line.text.contains("create")
                    || line.text.contains("update")
                    || line.text.contains("delete")
            })
        {
            findings.push(function_finding(
                file,
                function,
                "service_method_combines_unrelated_write_paths_via_action_string",
                Severity::Info,
                line,
                "service method uses action or mode switching to implement several write paths",
                vec!["splitting use cases usually reads better than one action-driven service method".to_string()],
            ));
        }

        if let Some(line) = mutates_request_binding_in_place(params_text, &lines) {
            findings.push(function_finding(
                file,
                function,
                "service_method_mutates_transport_dto_in_place",
                Severity::Info,
                line,
                "service method mutates a request-shaped DTO in place",
                vec!["mapping into a service-layer input struct is usually clearer than rewriting transport DTOs".to_string()],
            ));
        }

        if signature_has_request_dto(params_text)
            && (params_text.contains("models.") || gorm_structs.iter().any(|name| params_text.contains(name)))
        {
            findings.push(function_finding(
                file,
                function,
                "service_method_accepts_dto_and_persistence_model_together",
                Severity::Warning,
                function.fingerprint.start_line,
                "service method signature mixes request DTOs and persistence model types",
                vec!["transport and persistence concerns are usually easier to evolve when they stay separate".to_string()],
            ));
        }

        if is_generic_name(&function.fingerprint.name)
            || matches!(function.fingerprint.name.as_str(), "Handle" | "Execute" | "Process")
        {
            findings.push(function_finding(
                file,
                function,
                "generic_process_execute_handle_service_name_without_domain_noun",
                Severity::Info,
                function.fingerprint.start_line,
                "service method uses a very generic name without an obvious domain noun",
                vec!["use-case services are usually easier to read when method names describe the domain action".to_string()],
            ));
        }

        if returns_text.contains("string") && lines.iter().any(|line| line.text.contains("fmt.Sprintf(") || line.text.contains("fmt.Errorf(")) {
            let line = lines
                .iter()
                .find(|line| line.text.contains("fmt.Sprintf(") || line.text.contains("fmt.Errorf("))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "service_returns_preformatted_client_message",
                Severity::Info,
                line,
                "service method appears to return preformatted client-facing messages",
                vec!["typed domain errors usually travel better across layers than final client text".to_string()],
            ));
        }

        if let Some(line) = not_found_nil_nil_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "not_found_semantics_implemented_by_nil_nil_return",
                Severity::Info,
                line,
                "service method encodes a missing result as nil, nil",
                vec!["explicit result contracts usually communicate absence more clearly than nil, nil".to_string()],
            ));
        }

        if function.signature_text.contains("*gin.Context")
            && let Some(line) = first_http_or_abort_semantics_line(file, &lines)
        {
            findings.push(function_finding(
                file,
                function,
                "service_depends_on_gin_abort_or_context_error_response",
                Severity::Warning,
                line,
                "service method depends on Gin abort or response behavior",
                vec!["transport response ownership usually belongs at the boundary layer".to_string()],
            ));
        }

        if constructor_like(function)
            && (params_text.contains("map[string]any") || params_text.contains("map[string]interface{}"))
        {
            findings.push(function_finding(
                file,
                function,
                "service_constructor_accepts_untyped_config_map",
                Severity::Info,
                function.fingerprint.start_line,
                "service constructor accepts an untyped config map",
                vec!["typed config structs usually make service wiring easier to validate than generic maps".to_string()],
            ));
        }

        if let Some(line) = cross_repo_write_without_tx_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "cross_repository_write_flow_without_shared_uow_boundary",
                Severity::Warning,
                line,
                "service coordinates several repository writes without an obvious transaction boundary",
                vec!["shared unit-of-work boundaries usually make multi-repository write flows easier to reason about".to_string()],
            ));
        }

        if lines.iter().any(|line| (line.text.contains("logger.") || line.text.contains("log.")) && line.text.contains("err")) {
            let line = lines
                .iter()
                .find(|line| (line.text.contains("logger.") || line.text.contains("log.")) && line.text.contains("err"))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "handler_and_service_both_log_same_error_chain",
                Severity::Info,
                line,
                "service path logs an error that is likely also logged at the boundary",
                vec!["logging ownership is usually clearer when one layer owns the final request-path error log".to_string()],
            ));
        }
    }

    if is_repository {
        if constructor_like(function)
            && (function.signature_text.contains("*gin.Engine")
                || function.signature_text.contains("gin.IRouter")
                || function.signature_text.contains("gin.RouterGroup"))
        {
            findings.push(function_finding(
                file,
                function,
                "repository_constructor_accepts_gin_engine_or_router",
                Severity::Warning,
                function.fingerprint.start_line,
                "repository constructor accepts Gin router types",
                vec!["repositories usually should not depend on transport router objects".to_string()],
            ));
        }

        if function.signature_text.contains("*gin.Context") {
            findings.push(function_finding(
                file,
                function,
                "repository_accepts_gin_context",
                Severity::Warning,
                function.fingerprint.start_line,
                "repository method accepts *gin.Context directly",
                vec!["repositories usually should accept context.Context rather than transport-specific context objects".to_string()],
            ));
        }

        if signature_has_request_dto(params_text) {
            findings.push(function_finding(
                file,
                function,
                "repository_accepts_http_request_dto",
                Severity::Warning,
                function.fingerprint.start_line,
                "repository method accepts request-shaped DTO input",
                vec!["repositories usually should receive domain values or persistence filters instead of HTTP-facing request contracts".to_string()],
            ));
        }

        if returns_transport_dto(returns_text) {
            findings.push(function_finding(
                file,
                function,
                "repository_returns_transport_dto",
                Severity::Warning,
                function.fingerprint.start_line,
                "repository method returns transport-facing DTO types",
                vec!["repository APIs usually should not know about response envelopes or view models".to_string()],
            ));
        }

        if returns_framework_builder(returns_text) {
            findings.push(function_finding(
                file,
                function,
                "repository_returns_framework_builder_to_upper_layer",
                Severity::Warning,
                function.fingerprint.start_line,
                "repository method returns framework query-builder or row-handle primitives",
                vec!["upper layers usually should not finish repositories by mutating leaked driver or ORM handles".to_string()],
            ));
        }

        if returns_text.contains("*gorm.DB") {
            findings.push(function_finding(
                file,
                function,
                "repository_returns_gorm_query_builder",
                Severity::Warning,
                function.fingerprint.start_line,
                "repository method returns *gorm.DB for callers to finish",
                vec!["repository methods usually read more clearly when they return results instead of partially built chains".to_string()],
            ));
        }

        if has_http_status_usage(file, &lines)
            && returns_text.contains("int")
            && returns_text.contains("error")
        {
            let line = first_http_status_line(file, &lines).unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "repository_returns_http_status_errors",
                Severity::Warning,
                line,
                "repository method appears to translate persistence failures into HTTP status returns",
                vec!["HTTP status mapping usually belongs in handlers or transport adapters, not repositories".to_string()],
            ));
        }

        if signature_mentions_transaction(&function.signature_text)
            && lines.iter().any(|line| line.text.contains("tx == nil"))
        {
            let line = lines
                .iter()
                .find(|line| line.text.contains("tx == nil"))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "optional_tx_nil_parameter_on_repository_api",
                Severity::Info,
                line,
                "repository method accepts a transaction parameter that is treated as optional",
                vec!["unit-of-work boundaries are usually easier to reason about than nil-able transaction parameters".to_string()],
            ));
        }

        if lines.iter().any(|line| transaction_start_line(&line.text)) {
            let line = lines
                .iter()
                .find(|line| transaction_start_line(&line.text))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "repository_begins_transaction_without_uow_or_callback",
                Severity::Info,
                line,
                "repository method begins its own transaction inline",
                vec!["repositories usually compose better when transaction ownership is explicit at a higher layer or through unit-of-work helpers".to_string()],
            ));
        }

        if !go.db_query_calls.is_empty() && !go.gorm_query_chains.is_empty() {
            let line = go.db_query_calls[0].line.min(go.gorm_query_chains[0].line);
            findings.push(function_finding(
                file,
                function,
                "repository_mixes_raw_sql_and_gorm_same_method_without_adapter_boundary",
                Severity::Info,
                line,
                "repository method mixes raw SQL calls and GORM chains in one path",
                vec!["explicit adapter boundaries are usually clearer than one method that spans several persistence styles".to_string()],
            ));
        }

        if returns_text.contains("sql.Null") {
            findings.push(function_finding(
                file,
                function,
                "sql_null_types_escape_repository_boundary",
                Severity::Info,
                function.fingerprint.start_line,
                "repository return type exposes sql.Null* wrappers",
                vec!["boundary-friendly DTO or domain types usually communicate optional values more clearly".to_string()],
            ));
        }

        if returns_text.contains("*gorm.DB")
            && (!go.gorm_query_chains.is_empty()
                || lines.iter().any(|line| line.text.contains(".Where(") || line.text.contains(".Scopes(")))
        {
            findings.push(function_finding(
                file,
                function,
                "repository_method_returns_partially_built_scopes_for_caller_chaining",
                Severity::Info,
                function.fingerprint.start_line,
                "repository method returns a partially built GORM chain",
                vec!["callers usually should not need to finish repository-owned scopes".to_string()],
            ));
        }

        let where_templates = go
            .gorm_query_chains
            .iter()
            .flat_map(|chain| chain.steps.iter())
            .filter(|step| step.method_name == "Where")
            .filter_map(|step| step.first_string_arg.clone())
            .collect::<Vec<_>>();
        if has_duplicate_string(&where_templates) {
            let line = go
                .gorm_query_chains
                .iter()
                .flat_map(|chain| chain.steps.iter())
                .find(|step| step.method_name == "Where" && step.first_string_arg.is_some())
                .map(|step| step.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "where_clause_templates_duplicated_across_repositories",
                Severity::Info,
                line,
                "repository repeats the same WHERE template in several places",
                vec!["shared query helpers or scopes usually reduce duplication in filter templates".to_string()],
            ));
        }
        if !has_duplicate_string(&where_templates)
            && lines
                .iter()
                .map(|line| line.text.match_indices("Where(\"status = ?\"").count())
                .sum::<usize>()
                >= 2
        {
            let line = lines
                .iter()
                .find(|line| line.text.contains("Where(\"status = ?\""))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "where_clause_templates_duplicated_across_repositories",
                Severity::Info,
                line,
                "repository repeats the same WHERE template in several places",
                vec!["shared query helpers or scopes usually reduce duplication in filter templates".to_string()],
            ));
        }

        if let Some(line) = base_repository_reflection_line(function, &lines) {
            findings.push(function_finding(
                file,
                function,
                "generic_base_repository_with_reflection_dispatch",
                Severity::Info,
                line,
                "generic repository helper depends on reflection-driven dispatch",
                vec!["bounded repository APIs are usually easier to reason about than reflection-heavy base repositories".to_string()],
            ));
        }
    }

    if is_handler
        && lines
            .iter()
            .any(|line| line.text.contains(".Begin()") || line.text.contains(".BeginTx(") || line.text.contains(".Transaction("))
    {
        let line = lines
            .iter()
            .find(|line| {
                line.text.contains(".Begin()")
                    || line.text.contains(".BeginTx(")
                    || line.text.contains(".Transaction(")
            })
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "handler_opens_transaction",
            Severity::Warning,
            line,
            "request handler opens a transaction directly",
            vec!["transaction ownership is usually clearer in services or unit-of-work helpers".to_string()],
        ));
    }

    if !is_repository
        && lines.iter().any(|line| line.text.contains(".Scan(") && line.text.to_ascii_lowercase().contains("rows"))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Scan(") && line.text.to_ascii_lowercase().contains("rows"))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "sql_rows_scan_logic_outside_repository",
            Severity::Info,
            line,
            "SQL rows scanning happens outside repository code",
            vec!["row scanning is usually clearer when kept inside persistence adapters".to_string()],
        ));
    }

    if is_middleware_file(file) {
        if references_repository_directly(&lines) {
            findings.push(function_finding(
                file,
                function,
                "middleware_calls_repository_directly",
                Severity::Info,
                function.fingerprint.start_line,
                "middleware reaches repository-like dependencies directly",
                vec!["cross-cutting middleware often stays simpler when it delegates business queries through dedicated services".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "middleware_contains_business_orchestration",
                Severity::Info,
                function.fingerprint.start_line,
                "middleware appears to own business orchestration",
                vec!["middleware is usually cleaner when limited to cross-cutting concerns".to_string()],
            ));
        }

        if let Some(line) = lines.iter().find(|line| transaction_start_line(&line.text)).map(|line| line.line) {
            findings.push(function_finding(
                file,
                function,
                "middleware_starts_db_transaction",
                Severity::Warning,
                line,
                "middleware starts a database transaction",
                vec!["request-wide transaction ownership in middleware can hide expensive coupling".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "middleware_opens_transaction",
                Severity::Warning,
                line,
                "middleware opens a transaction for downstream business logic",
                vec!["transaction lifetime is usually clearer in services or unit-of-work helpers".to_string()],
            ));
        }

        if go.gin_calls.iter().any(|call| matches!(call.operation.as_str(), "json" | "pure_json")) {
            let line = go
                .gin_calls
                .iter()
                .find(|call| matches!(call.operation.as_str(), "json" | "pure_json"))
                .map(|call| call.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "middleware_writes_business_response_payloads",
                Severity::Info,
                line,
                "middleware writes response payloads directly",
                vec!["middleware is usually easier to reason about when handlers own success-response shaping".to_string()],
            ));
        }

        if lines.iter().any(|line| {
            line.text.contains(".Create(")
                || line.text.contains(".Save(")
                || line.text.contains(".Updates(")
                || line.text.contains(".Delete(")
        }) {
            let line = lines
                .iter()
                .find(|line| {
                    line.text.contains(".Create(")
                        || line.text.contains(".Save(")
                        || line.text.contains(".Updates(")
                        || line.text.contains(".Delete(")
                })
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "middleware_mutates_domain_model_state",
                Severity::Warning,
                line,
                "middleware mutates persistence or domain state directly",
                vec!["cross-cutting middleware is usually safer when it avoids business writes".to_string()],
            ));
        }

        if let Some(line) = global_singleton_reference_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "middleware_uses_global_logger_or_config_singleton",
                Severity::Info,
                line,
                "middleware depends on global logger or config state",
                vec!["middleware is usually easier to test when logger and config are injected".to_string()],
            ));
        }
    }

    if constructor_like(function) {
        if let Some(line) = first_env_lookup_line(file, &lines) {
            findings.push(function_finding(
                file,
                function,
                "constructor_reads_env_directly",
                Severity::Info,
                line,
                "constructor reads environment or config directly",
                vec!["constructors are easier to test when normalized config is injected instead of looked up inline".to_string()],
            ));
        }
    }

    if is_router_file(file)
        && (function.fingerprint.name.contains("Route")
            || function.fingerprint.name.contains("Router")
            || function.fingerprint.name.contains("Register"))
    {
        if let Some(line) = lines.iter().find(|line| migration_line(&line.text)).map(|line| line.line)
        {
            findings.push(function_finding(
                file,
                function,
                "router_setup_runs_migrations",
                Severity::Warning,
                line,
                "router setup performs migration or schema work",
                vec!["route wiring is usually easier to reason about when bootstrap, migrations, and transport registration stay separate".to_string()],
            ));
        }

        let inline_handler_count = lines.iter().filter(|line| line.text.contains("func(c *gin.Context)") || line.text.contains("func(c * gin.Context)")).count();
        if inline_handler_count >= 2 {
            let line = lines
                .iter()
                .find(|line| line.text.contains("func(c *gin.Context)") || line.text.contains("func(c * gin.Context)"))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "gin_route_registration_anonymous_function_overuse",
                Severity::Info,
                line,
                "route registration uses several inline anonymous Gin handlers",
                vec!["named handlers or handler methods are usually easier to test and reuse".to_string()],
            ));
        }

        if let Some((line, path)) = repeated_route_path(lines.as_slice()) {
            findings.push(function_finding(
                file,
                function,
                "gin_route_paths_repeated_as_raw_strings",
                Severity::Info,
                line,
                format!("route path {path} is repeated as a raw string"),
                vec!["centralizing route path segments usually reduces drift".to_string()],
            ));
        }

        if let Some(line) = route_registration_contains_business_logic_line(file, function, &lines)
        {
            findings.push(function_finding(
                file,
                function,
                "route_registration_contains_business_logic",
                Severity::Info,
                line,
                "route registration function also performs business or persistence work",
                vec!["router setup usually reads better when it only wires handlers and middleware".to_string()],
            ));
        }

        if let Some(line) = router_constructs_dependencies_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "router_constructor_builds_concrete_dependencies",
                Severity::Info,
                line,
                "router setup constructs concrete dependencies inline",
                vec!["router constructors are usually simpler when dependencies are assembled earlier in bootstrap".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "bootstrap_builds_clients_inside_route_registration",
                Severity::Info,
                line,
                "route registration hides client or repository construction",
                vec!["building clients and repositories inside route registration makes startup ownership harder to see".to_string()],
            ));
        }

        if let Some(line) = background_worker_registration_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "background_worker_started_from_http_handler_registration",
                Severity::Info,
                line,
                "route registration starts background work as a side effect",
                vec!["HTTP route setup and worker startup are usually easier to manage as separate bootstrap concerns".to_string()],
            ));
            findings.push(function_finding(
                file,
                function,
                "background_jobs_registered_from_gin_packages_instead_of_bootstrap",
                Severity::Info,
                line,
                "Gin package registers background jobs directly",
                vec!["schedulers and workers usually belong in application bootstrap, not transport packages".to_string()],
            ));
        }

        if let Some(line) = admin_debug_route_line(&lines) {
            findings.push(function_finding(
                file,
                function,
                "admin_or_debug_endpoint_registration_mixed_into_public_router_setup",
                Severity::Info,
                line,
                "admin or debug routes are registered in the same public router setup",
                vec!["operational endpoints are easier to govern when clearly separated from public routing".to_string()],
            ));
        }

        let constructor_names = lines
            .iter()
            .filter_map(|line| extract_constructor_name(&line.text))
            .collect::<Vec<_>>();
        if has_duplicate_string(&constructor_names) {
            let line = lines.first().map(|line| line.line).unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "same_dependency_wired_in_multiple_bootstrap_locations",
                Severity::Info,
                line,
                "same dependency appears to be constructed repeatedly in one bootstrap path",
                vec!["shared bootstrap composition usually avoids repeated construction of the same dependency".to_string()],
            ));
        }

        if lines.iter().any(|line| migration_line(&line.text))
            && lines.iter().any(|line| line.text.contains(".Run(") || line.text.contains(".ListenAndServe("))
        {
            let line = lines
                .iter()
                .find(|line| migration_line(&line.text))
                .map(|line| line.line)
                .unwrap_or(function.fingerprint.start_line);
            findings.push(function_finding(
                file,
                function,
                "migration_runner_and_api_server_bootstrap_coupled",
                Severity::Info,
                line,
                "bootstrap path runs migrations and serves traffic together",
                vec!["separating migration and serve lifecycles often makes deployments safer".to_string()],
            ));
        }
    }

    if function.fingerprint.name == "init"
        && let Some(line) = lines
            .iter()
            .find(|line| init_registers_dependencies_or_routes(&line.text))
            .map(|line| line.line)
    {
        findings.push(function_finding(
            file,
            function,
            "init_registers_routes_or_dependencies",
            Severity::Warning,
            line,
            "init() registers routes or constructs runtime dependencies",
            vec!["side-effectful startup registration is usually clearer in explicit bootstrap code".to_string()],
        ));
    }

    if is_gorm_hook(file, function, &gorm_structs) {
        if let Some(line) = first_external_io_line(file, &lines) {
            findings.push(function_finding(
                file,
                function,
                "gorm_model_hook_contains_external_io",
                Severity::Warning,
                line,
                "GORM model hook performs external I/O",
                vec!["model hooks are usually safer when they stay local to persistence state changes".to_string()],
            ));
        }

        if import_path_has_any_role(file, SERVICE_ROLE_HINTS)
            || import_path_has_any_role(file, REPOSITORY_ROLE_HINTS)
        {
            findings.push(function_finding(
                file,
                function,
                "gorm_model_hook_calls_service_or_repository",
                Severity::Warning,
                function.fingerprint.start_line,
                "GORM model hook depends on service or repository packages",
                vec!["model hooks usually should not reach upward into higher application layers".to_string()],
            ));
        }
    }

    if is_gorm_hook(file, function, &gorm_structs)
        && (go
            .gorm_query_chains
            .iter()
            .any(|chain| chain.root_text.contains("Other") || chain.root_text.contains("Audit"))
            || lines.iter().any(|line| line.text.contains("Audit")))
    {
        let line = go
            .gorm_query_chains
            .iter()
            .find(|chain| chain.root_text.contains("Other") || chain.root_text.contains("Audit"))
            .map(|chain| chain.line)
            .or_else(|| lines.iter().find(|line| line.text.contains("Audit")).map(|line| line.line))
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "gorm_hook_mutates_unrelated_tables",
            Severity::Info,
            line,
            "GORM hook appears to touch unrelated models or tables",
            vec!["hooks are usually safer when limited to the owning aggregate".to_string()],
        ));
    }

    if !is_repository && !is_handler && !is_middleware_file(file)
        && lines.iter().any(|line| line.text.contains(".Session(") || line.text.contains(".Clauses("))
        && has_import_path(file, "gorm.io/gorm")
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Session(") || line.text.contains(".Clauses("))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "gorm_session_options_configured_outside_repository",
            Severity::Info,
            line,
            "GORM session or clause options are configured outside a repository boundary",
            vec!["query configuration usually belongs with repository-owned query shape".to_string()],
        ));
    }

    if !is_repository
        && lines.iter().any(|line| line.text.contains(".Session("))
        && has_import_path(file, "gorm.io/gorm")
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Session("))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "gorm_session_options_configured_outside_repository",
            Severity::Info,
            line,
            "GORM session options are configured outside repository code",
            vec!["query configuration usually belongs with repository-owned query shape".to_string()],
        ));
    }

    if go
        .gorm_query_chains
        .iter()
        .any(|chain| chain.steps.iter().any(|step| step.method_name == "Unscoped"))
        && !danger_named(&function.fingerprint.name)
    {
        let line = go
            .gorm_query_chains
            .iter()
            .find(|chain| chain.steps.iter().any(|step| step.method_name == "Unscoped"))
            .map(|chain| chain.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "unscoped_query_without_explicit_danger_naming",
            Severity::Info,
            line,
            "Unscoped query appears in a generically named function",
            vec!["hard-delete or unscoped access is usually easier to review when the function name makes the danger explicit".to_string()],
        ));
    }
    if !danger_named(&function.fingerprint.name)
        && lines.iter().any(|line| line.text.contains(".Unscoped("))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Unscoped("))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "unscoped_query_without_explicit_danger_naming",
            Severity::Info,
            line,
            "Unscoped query appears in a generically named function",
            vec!["hard-delete or unscoped access is usually easier to review when the function name makes the danger explicit".to_string()],
        ));
    }

    if is_service
        && returns_text.contains("*gorm.DB")
        && (function.fingerprint.name.contains("Tx") || function.fingerprint.name.contains("Transaction"))
    {
        findings.push(function_finding(
            file,
            function,
            "service_returns_tx_to_caller",
            Severity::Warning,
            function.fingerprint.start_line,
            "service method returns transaction-like state to callers",
            vec!["services usually should own transactional work instead of leaking transaction handles".to_string()],
        ));
    }

    if is_service && signature_mentions_transaction(&function.signature_text) && lines.iter().any(|line| transaction_start_line(&line.text)) {
        let line = lines
            .iter()
            .find(|line| transaction_start_line(&line.text))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "same_service_method_accepts_tx_and_begins_tx",
            Severity::Info,
            line,
            "service method both accepts a transaction and begins one",
            vec!["mixing caller-owned and self-owned transactions usually hides ownership".to_string()],
        ));
    }

    if let Some(line) = external_http_inside_transaction_line(file, &lines) {
        findings.push(function_finding(
            file,
            function,
            "external_http_call_inside_transaction_scope",
            Severity::Warning,
            line,
            "network I/O occurs while a transaction appears to be open",
            vec!["holding open transactions across network calls usually increases contention and ambiguity".to_string()],
        ));
    }

    if let Some(line) = action_inside_transaction_line(&lines, &["Publish(", "publish("]) {
        findings.push(function_finding(
            file,
            function,
            "event_publish_before_transaction_commit",
            Severity::Warning,
            line,
            "event publication appears before transaction commit",
            vec!["durable commit usually should happen before publishing dependent events".to_string()],
        ));
    }

    if let Some(line) = action_inside_transaction_line(&lines, &["Invalidate(", ".Del(", ".Delete("]) {
        findings.push(function_finding(
            file,
            function,
            "cache_invalidation_before_transaction_commit",
            Severity::Warning,
            line,
            "cache invalidation appears before transaction commit",
            vec!["cache mutation before commit can leave the cache ahead of durable state".to_string()],
        ));
    }

    if let Some(line) = action_inside_transaction_line(&lines, &["go func", "go "]) {
        findings.push(function_finding(
            file,
            function,
            "background_goroutine_started_inside_transaction_scope",
            Severity::Warning,
            line,
            "goroutine is started while transaction state appears active",
            vec!["background work that depends on uncommitted state is usually fragile".to_string()],
        ));
    }

    if !is_repository && gorm_scope_count(function) >= 2
    {
        findings.push(function_finding(
            file,
            function,
            "gorm_scopes_defined_inline_repeatedly",
            Severity::Info,
            function.fingerprint.start_line,
            "function defines several inline GORM scopes",
            vec!["named shared scopes are usually easier to govern than repeated inline closures".to_string()],
        ));
    }

    if !is_repository
        && (preload_count(function) >= 2
            || lines.iter().filter(|line| line.text.contains(".Preload(")).count() >= 2)
    {
        findings.push(function_finding(
            file,
            function,
            "preload_rules_scattered_across_layers",
            Severity::Info,
            function.fingerprint.start_line,
            "preload rules are built outside a repository boundary",
            vec!["preload ownership is usually clearer when repositories own query shape".to_string()],
        ));
    }

    if soft_delete_line_count(&lines) >= 2 {
        let line = lines
            .iter()
            .find(|line| line.text.contains("deleted_at") && line.text.contains("IS NULL"))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "soft_delete_filters_written_manually_in_many_queries",
            Severity::Info,
            line,
            "function repeats manual soft-delete filters",
            vec!["shared scopes or repository helpers usually keep soft-delete policy more consistent".to_string()],
        ));
    }

    if !is_repository && let Some(line) = locking_clause_line(&lines) {
        findings.push(function_finding(
            file,
            function,
            "gorm_locking_clauses_built_outside_repository",
            Severity::Warning,
            line,
            "locking clause is built outside repository code",
            vec!["locking behavior is usually easier to review when owned by repositories".to_string()],
        ));
    }

    if (function.fingerprint.name.contains("Patch") || function.fingerprint.name.contains("Update"))
        && let Some(line) = update_struct_without_field_intent_line(&lines)
    {
        findings.push(function_finding(
            file,
            function,
            "updates_with_struct_used_for_patch_without_field_intent_helper",
            Severity::Info,
            line,
            "patch-like update uses a struct write without explicit field intent",
            vec!["patch semantics are usually safer when selected fields are made explicit".to_string()],
        ));
    }

    if !is_repository && let Some(line) = map_update_flow_line(&lines) {
        findings.push(function_finding(
            file,
            function,
            "map_based_updates_passed_from_handler_to_repository",
            Severity::Info,
            line,
            "map-based update payload flows through upper layers",
            vec!["typed update contracts usually age better than ad hoc update maps".to_string()],
        ));
    }

    if !is_repository && let Some(line) = transaction_cross_layer_line(function, &lines) {
        findings.push(function_finding(
            file,
            function,
            "transaction_object_crosses_more_than_one_layer_boundary",
            Severity::Info,
            line,
            "transaction handle crosses layer boundaries as a regular argument",
            vec!["transaction ownership is usually clearer when hidden behind unit-of-work boundaries".to_string()],
        ));
    }

    if let Some(line) = split_commit_rollback_line(&lines) {
        findings.push(function_finding(
            file,
            function,
            "commit_or_rollback_split_across_functions_without_owner",
            Severity::Info,
            line,
            "transaction completion paths look split without one clear owner",
            vec!["one explicit transaction owner usually makes commit and rollback behavior easier to follow".to_string()],
        ));
    }

    if let Some(line) = nested_tx_line(&lines) {
        findings.push(function_finding(
            file,
            function,
            "savepoint_or_nested_tx_logic_scattered_without_dedicated_helper",
            Severity::Info,
            line,
            "nested transaction or savepoint logic appears inline",
            vec!["savepoint and nested transaction flow is usually easier to audit behind a dedicated helper".to_string()],
        ));
    }

    if !(constructor_like(function) || is_router_file(file))
        && let Some(line) = feature_flag_lookup_line(file, &lines)
    {
        findings.push(function_finding(
            file,
            function,
            "feature_flag_lookup_without_config_abstraction",
            Severity::Info,
            line,
            "feature flag or config lookup happens inline",
            vec!["focused config interfaces usually make feature gating easier to test than direct lookups".to_string()],
        ));
    }

    if is_main_or_cmd_file(file)
        && lines.iter().any(|line| line.text.contains("flag.Parse("))
        && lines.iter().any(|line| line.text.contains("gorm.Open(") || line.text.contains("NewService(") || line.text.contains("NewRepository("))
        && lines.iter().any(|line| line.text.contains(".Run(") || line.text.contains("ListenAndServe("))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains("flag.Parse("))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "main_or_cmd_mixes_cli_parsing_server_bootstrap_and_business_config",
            Severity::Info,
            line,
            "main or cmd path mixes CLI parsing, server startup, and business wiring",
            vec!["startup composition helpers usually make entrypoints easier to maintain than one large bootstrap path".to_string()],
        ));
    }

    if is_main_or_cmd_file(file) && let Some(line) = lifecycle_start_without_shutdown_line(&lines)
    {
        findings.push(function_finding(
            file,
            function,
            "application_lifecycle_missing_shutdown_owner",
            Severity::Info,
            line,
            "startup path launches long-lived resources without an obvious shutdown owner",
            vec!["application lifecycle wrappers usually make shutdown behavior more explicit".to_string()],
        ));
    }

    if is_main_or_cmd_file(file)
        && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS)
        && lines.iter().any(|line| line.text.contains("NewHandlerService(") || line.text.contains("gin."))
    {
        findings.push(function_finding(
            file,
            function,
            "operational_command_handlers_reuse_http_services_without_adapter",
            Severity::Info,
            function.fingerprint.start_line,
            "operational command path appears to depend on HTTP-oriented service contracts",
            vec!["CLI and worker entrypoints usually compose better against transport-neutral application services".to_string()],
        ));
    }

    if is_handler && let Some(line) = metrics_label_line(&lines) {
        findings.push(function_finding(
            file,
            function,
            "metrics_labels_built_inline_in_handlers",
            Severity::Info,
            line,
            "handler builds metric labels inline",
            vec!["focused observation helpers usually keep metric label sets more consistent than ad hoc handler code".to_string()],
        ));
    }

    if is_repository && let Some(line) = repository_log_http_metadata_line(&lines) {
        findings.push(function_finding(
            file,
            function,
            "repository_logs_with_http_status_or_route_labels",
            Severity::Info,
            line,
            "repository log call includes HTTP route or status metadata",
            vec!["persistence logs are usually clearer when they stick to persistence-domain fields rather than transport labels".to_string()],
        ));
    }

    if is_handler && let Some(line) = audit_before_service_line(&lines) {
        findings.push(function_finding(
            file,
            function,
            "audit_logging_executed_in_handler_before_service_success",
            Severity::Info,
            line,
            "handler performs audit logging before service success is known",
            vec!["audit logging often belongs after the business outcome is confirmed".to_string()],
        ));
    }

    if !is_repository
        && lines.iter().any(|line| line.text.contains(".Order(\""))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains(".Order(\""))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "order_by_clause_literals_scattered_across_layers",
            Severity::Info,
            line,
            "raw ORDER BY clause is built outside repository code",
            vec!["shared query helpers usually reduce scattered sort expressions across layers".to_string()],
        ));
    }

    if is_repository
        && lines.iter().any(|line| line.text.contains("tx.") && line.text.contains("Status"))
        && lines.iter().any(|line| line.text.contains("err"))
    {
        let line = lines
            .iter()
            .find(|line| line.text.contains("tx.") && line.text.contains("Status"))
            .map(|line| line.line)
            .unwrap_or(function.fingerprint.start_line);
        findings.push(function_finding(
            file,
            function,
            "transaction_error_translation_done_in_repository_and_handler",
            Severity::Info,
            line,
            "repository translates transaction failures into transport status semantics",
            vec!["keeping transaction error translation in one layer usually reduces double handling".to_string()],
        ));
    }

    if file_matches_any_role(file, VALIDATION_ROLE_HINTS)
        && references_repository_directly(&lines)
    {
        findings.push(function_finding(
            file,
            function,
            "validator_depends_on_repository_directly",
            Severity::Info,
            function.fingerprint.start_line,
            "validator code reaches repository-like dependencies directly",
            vec!["validators usually compose better from precomputed facts or focused services than direct repository calls".to_string()],
        ));
    }

    findings
}

fn test_architecture_findings(file: &ParsedFile) -> Vec<Finding> {
    let import_line = file.imports.first().map(|import| import.line).unwrap_or(1);
    let mut findings = Vec::new();

    if is_service_file(file) && has_import_path(file, "github.com/gin-gonic/gin") {
        findings.push(file_finding(
            file,
            "service_tests_import_gin",
            Severity::Info,
            import_line,
            "service-layer tests import Gin directly",
            vec!["transport-neutral services usually can be tested without a Gin dependency".to_string()],
        ));
    }

    if is_repository_file(file)
        && (has_import_path(file, "github.com/gin-gonic/gin") || has_import_path(file, "net/http"))
    {
        findings.push(file_finding(
            file,
            "repository_tests_depend_on_http_transport_types",
            Severity::Info,
            import_line,
            "repository tests depend on HTTP transport packages directly",
            vec!["repository tests usually read more clearly when they focus on persistence contracts instead of transport types".to_string()],
        ));
    }

    if is_transport_file(file) && (has_import_path(file, "gorm.io/gorm") || has_sql_like_import(file)) {
        findings.push(file_finding(
            file,
            "handler_tests_use_real_database_without_seam",
            Severity::Info,
            import_line,
            "transport test imports database dependencies directly",
            vec!["handler tests usually benefit from seams above the repository layer".to_string()],
        ));
    }

    if file
        .imports
        .iter()
        .any(|import| import.path.contains("models") || import.path.contains("model"))
        && is_transport_file(file)
    {
        findings.push(file_finding(
            file,
            "tests_couple_to_gorm_model_for_api_contract_assertions",
            Severity::Info,
            import_line,
            "transport test imports persistence models directly",
            vec!["API tests usually read more clearly against response contracts than persistence shape".to_string()],
        ));
    }

    if is_transport_file(file) {
        let test_functions = file
            .functions
            .iter()
            .filter(|function| function.is_test_function)
            .collect::<Vec<_>>();

        if test_functions.len() >= 2
            && test_functions
                .iter()
                .filter(|function| body_lines(function).iter().any(|line| line.text.contains("gin.New(") || line.text.contains("RegisterRoutes(")))
                .count()
                >= 2
        {
            findings.push(file_finding(
                file,
                "route_registration_tests_duplicate_full_bootstrap_per_file",
                Severity::Info,
                test_functions[0].fingerprint.start_line,
                "multiple transport tests rebuild full route bootstrap inline",
                vec!["shared route test setup usually reduces repeated bootstrap noise".to_string()],
            ));
        }

        if test_functions.iter().any(|function| raw_json_assertion_line(&body_lines(function)).is_some()) {
            let line = test_functions
                .iter()
                .find_map(|function| raw_json_assertion_line(&body_lines(function)))
                .unwrap_or(import_line);
            findings.push(file_finding(
                file,
                "tests_assert_raw_json_strings_without_response_dto",
                Severity::Info,
                line,
                "transport test asserts raw JSON strings directly",
                vec!["typed response DTO assertions usually age better than brittle raw JSON comparisons".to_string()],
            ));
        }

        if test_functions.iter().any(|function| gin_context_stub_line(&body_lines(function)).is_some()) {
            let line = test_functions
                .iter()
                .find_map(|function| gin_context_stub_line(&body_lines(function)))
                .unwrap_or(import_line);
            findings.push(file_finding(
                file,
                "tests_stub_gin_context_instead_of_httptest_boundary",
                Severity::Info,
                line,
                "transport test stubs Gin context directly",
                vec!["httptest boundary tests usually capture transport behavior more realistically than mocking Gin internals".to_string()],
            ));
        }

        if test_functions.iter().any(|function| transport_test_repo_touch_line(&body_lines(function)).is_some()) {
            let line = test_functions
                .iter()
                .find_map(|function| transport_test_repo_touch_line(&body_lines(function)))
                .unwrap_or(import_line);
            findings.push(file_finding(
                file,
                "transport_tests_bypass_service_interface_and_touch_repo_directly",
                Severity::Info,
                line,
                "transport test reaches repository construction directly",
                vec!["handler tests usually stay clearer when they depend on the service seam rather than repository details".to_string()],
            ));
        }

        if test_functions.iter().any(|function| sql_query_assertion_line(&body_lines(function)).is_some()) {
            let line = test_functions
                .iter()
                .find_map(|function| sql_query_assertion_line(&body_lines(function)))
                .unwrap_or(import_line);
            findings.push(file_finding(
                file,
                "sql_query_text_asserted_in_handler_tests",
                Severity::Info,
                line,
                "transport test asserts raw SQL text",
                vec!["SQL shape assertions usually belong closer to repository tests than handler tests".to_string()],
            ));
        }

        if file.path.to_string_lossy().to_ascii_lowercase().contains("/handler/")
            && test_functions.iter().any(|function| body_lines(function).iter().any(|line| migration_line(&line.text)))
        {
            let line = test_functions
                .iter()
                .find_map(|function| {
                    body_lines(function)
                        .iter()
                        .find(|line| migration_line(&line.text))
                        .map(|line| line.line)
                })
                .unwrap_or(import_line);
            findings.push(file_finding(
                file,
                "migration_tests_live_under_handler_packages",
                Severity::Info,
                line,
                "migration-oriented test lives under a handler package",
                vec!["migration tests are usually easier to govern outside transport packages".to_string()],
            ));
        }

        if test_functions.iter().any(|function| table_driven_multi_domain_line(&body_lines(function)).is_some()) {
            let line = test_functions
                .iter()
                .find_map(|function| table_driven_multi_domain_line(&body_lines(function)))
                .unwrap_or(import_line);
            findings.push(file_finding(
                file,
                "table_driven_tests_mix_multiple_domains_in_one_cases_slice",
                Severity::Info,
                line,
                "table-driven test mixes several domain concerns in one case set",
                vec!["splitting large mixed-domain case tables usually keeps test intent clearer".to_string()],
            ));
        }

        if test_functions.iter().filter(|function| body_lines(function).iter().any(|line| line.text.contains("gin.New(") || line.text.contains("gorm.Open("))).count() >= 2 {
            findings.push(file_finding(
                file,
                "shared_integration_test_setup_not_centralized_under_test_support",
                Severity::Info,
                test_functions[0].fingerprint.start_line,
                "integration-like test setup is repeated in the same file",
                vec!["shared test support usually keeps router and DB bootstrap from repeating across tests".to_string()],
            ));
        }
    }

    if file.is_test_file {
        let helper_functions = file
            .functions
            .iter()
            .filter(|function| !function.is_test_function && (function.fingerprint.name.starts_with("build") || function.fingerprint.name.starts_with("new") || function.fingerprint.name.starts_with("make")))
            .collect::<Vec<_>>();
        if helper_functions.len() >= 2 {
            findings.push(file_finding(
                file,
                "test_helpers_duplicated_across_packages",
                Severity::Info,
                helper_functions[0].fingerprint.start_line,
                "test file defines several bespoke setup helpers",
                vec!["shared test helpers usually reduce repeated setup builders across packages".to_string()],
            ));
        }

        if file.go_structs().iter().filter(|go_struct| go_struct.name.contains("Mock") || go_struct.name.contains("Fake")).count() >= 2 {
            let line = file
                .go_structs()
                .iter()
                .find(|go_struct| go_struct.name.contains("Mock") || go_struct.name.contains("Fake"))
                .map(|go_struct| go_struct.line)
                .unwrap_or(import_line);
            findings.push(file_finding(
                file,
                "mock_repository_types_duplicated_across_tests",
                Severity::Info,
                line,
                "test file declares several bespoke fake repository types",
                vec!["shared mocks or focused stubs usually reduce repeated mock types across tests".to_string()],
            ));
        }
    }

    findings
}

fn struct_tag_map<'a>(file: &'a ParsedFile) -> BTreeMap<&'a str, Vec<&'a StructTag>> {
    let mut map = BTreeMap::<&str, Vec<&StructTag>>::new();
    for tag in file.struct_tags() {
        map.entry(tag.struct_name.as_str()).or_default().push(tag);
    }
    map
}

fn gorm_struct_names(file: &ParsedFile, tag_map: &BTreeMap<&str, Vec<&StructTag>>) -> BTreeSet<String> {
    file.go_structs()
        .iter()
        .filter(|go_struct| has_any_tag_for_struct(tag_map, &go_struct.name, &["gorm:"]))
        .map(|go_struct| go_struct.name.clone())
        .collect()
}

fn has_any_tag_for_struct(
    tag_map: &BTreeMap<&str, Vec<&StructTag>>,
    struct_name: &str,
    markers: &[&str],
) -> bool {
    tag_map.get(struct_name).is_some_and(|tags| {
        tags.iter()
            .any(|tag| markers.iter().any(|marker| tag.raw_tag.contains(marker)))
    })
}

fn file_finding(
    file: &ParsedFile,
    rule_id: &str,
    severity: Severity,
    line: usize,
    message: impl Into<String>,
    evidence: Vec<String>,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: None,
        start_line: line,
        end_line: line,
        message: message.into(),
        evidence,
    }
}

fn function_finding(
    file: &ParsedFile,
    function: &ParsedFunction,
    rule_id: &str,
    severity: Severity,
    line: usize,
    message: impl Into<String>,
    evidence: Vec<String>,
) -> Finding {
    Finding {
        rule_id: rule_id.to_string(),
        severity,
        path: file.path.clone(),
        function_name: Some(function.fingerprint.name.clone()),
        start_line: line,
        end_line: line,
        message: message.into(),
        evidence,
    }
}

fn is_service_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, SERVICE_ROLE_HINTS)
}

fn is_repository_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, REPOSITORY_ROLE_HINTS)
}

fn is_model_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, MODEL_ROLE_HINTS)
}

fn is_transport_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, TRANSPORT_ROLE_HINTS)
}

fn is_router_file(file: &ParsedFile) -> bool {
    file_has_dedicated_role_home(file, ROUTER_ROLE_HINTS)
}

fn is_middleware_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, MIDDLEWARE_ROLE_HINTS)
}

fn file_matches_any_role(file: &ParsedFile, hints: &[&str]) -> bool {
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    let package = file.package_name.as_deref().unwrap_or_default().to_ascii_lowercase();

    hints.iter().any(|hint| {
        path.contains(&format!("/{hint}/"))
            || path.contains(&format!("/{hint}_"))
            || path.contains(&format!("_{hint}."))
            || path.ends_with(&format!("/{hint}.go"))
            || path.contains(hint)
            || package == *hint
    })
}

fn file_has_dedicated_role_home(file: &ParsedFile, hints: &[&str]) -> bool {
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    let package = file.package_name.as_deref().unwrap_or_default().to_ascii_lowercase();

    hints.iter().any(|hint| {
        package == *hint
            || path.contains(&format!("/{hint}/"))
            || path.ends_with(&format!("/{hint}.go"))
    })
}

fn import_path_has_any_role(file: &ParsedFile, hints: &[&str]) -> bool {
    file.imports.iter().any(|import| {
        let path = import.path.to_ascii_lowercase();
        hints.iter().any(|hint| {
            path.contains(&format!("/{hint}/"))
                || path.ends_with(&format!("/{hint}"))
                || path.contains(&format!("/{hint}_"))
                || path.contains(hint)
        })
    })
}

fn repository_type_name(name: &str) -> bool {
    name.ends_with("Repository") || name.ends_with("Repo") || name.ends_with("Store")
}

fn request_struct_name(name: &str) -> bool {
    REQUEST_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn response_struct_name(name: &str) -> bool {
    RESPONSE_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn api_error_struct_name(name: &str) -> bool {
    API_ERROR_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn signature_returns_text(signature: &str) -> &str {
    let Some((_, close_idx)) = signature_group_bounds(signature, params_group_index(signature)) else {
        return "";
    };
    signature[close_idx + 1..].trim()
}

fn signature_params_text(signature: &str) -> &str {
    let Some((open_idx, close_idx)) = signature_group_bounds(signature, params_group_index(signature)) else {
        return "";
    };
    &signature[open_idx + 1..close_idx]
}

fn params_group_index(signature: &str) -> usize {
    if signature.trim_start().starts_with("func (") {
        1
    } else {
        0
    }
}

fn signature_group_bounds(signature: &str, group_index: usize) -> Option<(usize, usize)> {
    let mut depth = 0usize;
    let mut current_open = None;
    let mut groups = Vec::new();

    for (idx, ch) in signature.char_indices() {
        match ch {
            '(' => {
                if depth == 0 {
                    current_open = Some(idx);
                }
                depth += 1;
            }
            ')' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    groups.push((current_open?, idx));
                }
            }
            _ => {}
        }
    }

    groups.get(group_index).copied()
}

fn has_sql_like_import(file: &ParsedFile) -> bool {
    [
        "database/sql",
        "github.com/jmoiron/sqlx",
        "github.com/jackc/pgx/v5",
        "github.com/jackc/pgx/v4",
    ]
    .iter()
    .any(|path| has_import_path(file, path))
}

fn is_body_bind_operation(operation: &str) -> bool {
    matches!(
        operation,
        "bind_json"
            | "should_bind_json"
            | "bind"
            | "should_bind"
            | "should_bind_body_with"
    )
}

fn model_import_aliases(file: &ParsedFile) -> Vec<String> {
    file.imports
        .iter()
        .filter(|import| {
            let path = import.path.to_ascii_lowercase();
            MODEL_ROLE_HINTS.iter().any(|hint| {
                path.contains(&format!("/{hint}/")) || path.ends_with(&format!("/{hint}"))
            })
        })
        .map(|import| import.alias.clone())
        .collect()
}

fn binding_looks_like_model(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
    gorm_structs: &BTreeSet<String>,
    binding: &str,
) -> bool {
    let aliases = model_import_aliases(file);

    lines.iter().any(|line| {
        (line.text.contains(&format!("var {binding} "))
            || line.text.contains(&format!("{binding} :="))
            || line.text.contains(&format!("{binding}=")))
            && (aliases
                .iter()
                .any(|alias| line.text.contains(&format!("{alias}.")))
                || gorm_structs
                    .iter()
                    .any(|struct_name| line.text.contains(struct_name)))
    })
}

fn expression_looks_like_model(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
    gorm_structs: &BTreeSet<String>,
    expression: &str,
) -> bool {
    let trimmed = expression.trim().trim_start_matches('&');
    if binding_looks_like_model(file, lines, gorm_structs, trimmed) {
        return true;
    }

    let aliases = model_import_aliases(file);
    aliases
        .iter()
        .any(|alias| expression.contains(&format!("{alias}.")))
        || gorm_structs
            .iter()
            .any(|struct_name| expression.contains(struct_name))
}

fn references_repository_directly(lines: &[super::framework_patterns::BodyLine]) -> bool {
    repository_receiver_count(lines) > 0
}

fn references_service_directly(lines: &[super::framework_patterns::BodyLine]) -> bool {
    receiver_count_with_suffix(lines, &["Service"]) > 0
}

fn repository_receiver_count(lines: &[super::framework_patterns::BodyLine]) -> usize {
    receiver_count_with_suffix(lines, &["Repo", "Repository", "Store"])
}

fn receiver_count_with_suffix(
    lines: &[super::framework_patterns::BodyLine],
    suffixes: &[&str],
) -> usize {
    let mut names = BTreeSet::new();

    for line in lines {
        for token in line
            .text
            .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        {
            if suffixes.iter().any(|suffix| token.ends_with(suffix)) {
                names.insert(token.to_string());
            }
        }
    }

    names.len()
}

fn first_config_lookup_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    first_matching_line(
        lines,
        &config_lookup_patterns(file)
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )
}

fn first_env_lookup_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    first_matching_line(
        lines,
        &env_lookup_patterns(file)
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )
}

fn first_http_status_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    let http_aliases = import_aliases_for(file, "net/http");
    let mut patterns = http_aliases
        .iter()
        .map(|alias| format!("{alias}.Status"))
        .collect::<Vec<_>>();
    patterns.push("AbortWithStatus".to_string());
    patterns.push("AbortWithStatusJSON".to_string());

    first_matching_line(lines, &patterns.iter().map(String::as_str).collect::<Vec<_>>())
}

fn has_http_status_usage(file: &ParsedFile, lines: &[super::framework_patterns::BodyLine]) -> bool {
    first_http_status_line(file, lines).is_some()
}

fn first_http_or_abort_semantics_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    let http_aliases = import_aliases_for(file, "net/http");
    let mut patterns = http_aliases
        .iter()
        .map(|alias| format!("{alias}.Status"))
        .collect::<Vec<_>>();
    patterns.extend([
        "Abort(".to_string(),
        "AbortWithStatus".to_string(),
        "AbortWithStatusJSON".to_string(),
    ]);

    first_matching_line(lines, &patterns.iter().map(String::as_str).collect::<Vec<_>>())
}

fn config_lookup_patterns(file: &ParsedFile) -> Vec<String> {
    let mut patterns = env_lookup_patterns(file);
    patterns.extend([
        "viper.Get".to_string(),
        "featureFlag".to_string(),
        "FeatureFlag".to_string(),
        "flag.Lookup".to_string(),
    ]);
    patterns
}

fn env_lookup_patterns(file: &ParsedFile) -> Vec<String> {
    let mut patterns = import_aliases_for(file, "os")
        .iter()
        .flat_map(|alias| {
            [
                format!("{alias}.Getenv("),
                format!("{alias}.LookupEnv("),
            ]
        })
        .collect::<Vec<_>>();

    if patterns.is_empty() {
        patterns.extend(["os.Getenv(".to_string(), "os.LookupEnv(".to_string()]);
    }

    patterns
}

fn returns_framework_builder(returns_text: &str) -> bool {
    [
        "*gorm.DB",
        "*sql.Rows",
        "*sql.Row",
        "*sqlx.Rows",
        "pgx.Rows",
        "sql.Rows",
    ]
    .iter()
    .any(|pattern| returns_text.contains(pattern))
}

fn returns_transport_dto(returns_text: &str) -> bool {
    RESPONSE_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| returns_text.contains(suffix))
        || returns_text.contains("Envelope")
        || returns_text.contains("View")
}

fn signature_has_request_dto(params_text: &str) -> bool {
    REQUEST_STRUCT_SUFFIXES
        .iter()
        .any(|suffix| params_text.contains(suffix))
}

fn signature_mentions_transaction(signature: &str) -> bool {
    signature.contains("tx *gorm.DB")
        || signature.contains("tx *sql.Tx")
        || signature.contains("tx *sqlx.Tx")
}

fn constructor_like(function: &ParsedFunction) -> bool {
    function.fingerprint.name.starts_with("New") || function.fingerprint.name.starts_with("Build")
}

fn migration_line(text: &str) -> bool {
    text.contains("AutoMigrate(") || text.contains("Migrate(") || text.contains("Migration")
}

fn init_registers_dependencies_or_routes(text: &str) -> bool {
    text.contains(".GET(")
        || text.contains(".POST(")
        || text.contains(".PUT(")
        || text.contains(".DELETE(")
        || text.contains(".Use(")
        || text.contains("NewService(")
        || text.contains("NewRepository(")
        || text.contains("gin.Default(")
        || text.contains("gin.New(")
}

fn is_gorm_hook(
    file: &ParsedFile,
    function: &ParsedFunction,
    gorm_structs: &BTreeSet<String>,
) -> bool {
    has_import_path(file, "gorm.io/gorm")
        && function
            .fingerprint
            .receiver_type
            .as_ref()
            .is_some_and(|receiver| gorm_structs.contains(receiver))
        && GORM_HOOK_METHODS.contains(&function.fingerprint.name.as_str())
}

fn first_external_io_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    let mut patterns = Vec::new();
    for alias in import_aliases_for(file, "net/http") {
        patterns.push(format!("{alias}.Get("));
        patterns.push(format!("{alias}.Post("));
        patterns.push(format!("{alias}.Do("));
    }
    for alias in import_aliases_for(file, "os") {
        patterns.push(format!("{alias}.WriteFile("));
        patterns.push(format!("{alias}.Create("));
        patterns.push(format!("{alias}.Open("));
    }
    for alias in import_aliases_for(file, "os/exec") {
        patterns.push(format!("{alias}.Command("));
    }
    patterns.push("publish(".to_string());
    patterns.push("Publish(".to_string());

    first_matching_line(lines, &patterns.iter().map(String::as_str).collect::<Vec<_>>())
}

fn first_matching_line(
    lines: &[super::framework_patterns::BodyLine],
    patterns: &[&str],
) -> Option<usize> {
    lines
        .iter()
        .find(|line| patterns.iter().any(|pattern| line.text.contains(pattern)))
        .map(|line| line.line)
}

fn global_singleton_name(name: &str) -> bool {
    matches!(name, "DB" | "Client" | "Logger" | "Config" | "Settings" | "Engine" | "Repo")
}

fn type_looks_like_global_singleton(type_text: &str) -> bool {
    ["*gorm.DB", "*sql.DB", "*gin.Engine", "*http.Client", "Logger", "Config"]
        .iter()
        .any(|pattern| type_text.contains(pattern))
}

fn looks_like_sql_literal(value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    ["SELECT ", "INSERT ", "UPDATE ", "DELETE ", " FROM ", " WHERE "]
        .iter()
        .any(|pattern| upper.contains(pattern))
}

fn transaction_helper_name(name: &str) -> bool {
    name.contains("Transaction") || name.contains("Transactional") || name.ends_with("Tx") || name.starts_with("WithTx")
}

fn mixed_role_symbol_count(file: &ParsedFile) -> usize {
    let mut roles = BTreeSet::new();
    for s in file.go_structs() {
        let name = s.name.as_str();
        if name.ends_with("Service") {
            roles.insert("service");
        }
        if repository_type_name(name) {
            roles.insert("repository");
        }
        if request_struct_name(name) || response_struct_name(name) {
            roles.insert("transport");
        }
        if name.ends_with("Validator") {
            roles.insert("validation");
        }
    }
    roles.len()
}

fn cross_layer_import_violation(file: &ParsedFile) -> bool {
    (is_repository_file(file) && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS))
        || (is_service_file(file) && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS))
        || (is_model_file(file) && import_path_has_any_role(file, TRANSPORT_ROLE_HINTS))
}

fn role_drift(file: &ParsedFile) -> Option<(usize, &'static str, &'static str)> {
    if is_service_file(file)
        && file.go_structs().iter().filter(|s| repository_type_name(&s.name)).count() > file.go_structs().iter().filter(|s| s.name.ends_with("Service")).count()
    {
        let line = file.go_structs().first().map(|s| s.line).unwrap_or(1);
        return Some((line, "service", "repository"));
    }
    if is_model_file(file)
        && file.go_structs().iter().filter(|s| request_struct_name(&s.name) || response_struct_name(&s.name)).count() > file.go_structs().iter().filter(|s| !request_struct_name(&s.name) && !response_struct_name(&s.name)).count()
    {
        let line = file.go_structs().first().map(|s| s.line).unwrap_or(1);
        return Some((line, "model", "transport"));
    }
    None
}

fn multiple_response_shapes(calls: &[crate::analysis::GinCallSummary]) -> bool {
    let kinds = calls
        .iter()
        .filter_map(|call| match call.operation.as_str() {
            "json" | "pure_json" | "indented_json" => Some("json"),
            "html" => Some("html"),
            "data" | "file" => Some("file"),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    kinds.len() >= 2
}

fn authorization_business_logic_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            (lower.contains("role") || lower.contains("permission") || lower.contains("authorize"))
                && (lower.contains("if ") || lower.contains("switch "))
        })
        .map(|line| line.line)
}

fn action_switch_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("action")
                && (lower.contains("switch") || lower.contains("if "))
                || lower.contains("mode")
                    && (lower.contains("switch") || lower.contains("if "))
        })
        .map(|line| line.line)
}

fn mixes_html_json_and_file(calls: &[crate::analysis::GinCallSummary]) -> bool {
    multiple_response_shapes(calls)
}

fn passes_gin_context_beyond_boundary(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("(c)") || line.text.contains(", c)") || line.text.contains("go func(c")
        })
        .map(|line| line.line)
}

fn global_singleton_reference_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            ["DB.", "Client.", "Logger.", "Config.", "Settings.", "Global"]
                .iter()
                .any(|pattern| line.text.contains(pattern))
        })
        .map(|line| line.line)
}

fn retry_or_backoff_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("retry") || lower.contains("backoff") || lower.contains("sleep(")
        })
        .map(|line| line.line)
}

fn constructor_instantiates_dependencies(lines: &[super::framework_patterns::BodyLine]) -> bool {
    lines.iter().any(|line| constructor_dependency_line(&line.text))
}

fn constructor_dependency_line(text: &str) -> bool {
    text.contains("NewRepo(")
        || text.contains("NewRepository(")
        || text.contains("NewClient(")
        || text.contains("NewStore(")
        || text.contains("log.New(")
        || text.contains("gorm.Open(")
}

fn pagination_or_query_parsing_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("pagesize")
                || lower.contains("page_size")
                || lower.contains("page")
                    && (lower.contains("atoi(") || lower.contains("parseint(") || lower.contains("query("))
                || lower.contains("sort=")
        })
        .map(|line| line.line)
}

fn request_binding_or_header_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains(".Header(")
                || line.text.contains(".GetHeader(")
                || line.text.contains(".FormValue(")
                || line.text.contains(".Param(")
                || line.text.contains(".Query(")
        })
        .map(|line| line.line)
}

fn mutates_request_binding_in_place(
    params_text: &str,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    let requestish = REQUEST_STRUCT_SUFFIXES
        .iter()
        .find(|suffix| params_text.contains(**suffix))?;
    lines
        .iter()
        .find(|line| line.text.contains('=') && line.text.contains('.'))
        .filter(|_line| params_text.contains(*requestish))
        .map(|line| line.line)
}

fn not_found_nil_nil_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("return nil, nil"))
        .map(|line| line.line)
}

fn transaction_start_line(text: &str) -> bool {
    text.contains(".Begin()") || text.contains(".BeginTx(") || text.contains(".Transaction(")
}

fn repeated_route_path(lines: &[super::framework_patterns::BodyLine]) -> Option<(usize, String)> {
    let mut seen = BTreeMap::<String, usize>::new();
    for line in lines {
        if let Some(path) = quoted_route_path(&line.text) {
            if let Some(first_line) = seen.get(&path) {
                return Some((*first_line, path));
            }
            seen.insert(path, line.line);
        }
    }
    None
}

fn quoted_route_path(text: &str) -> Option<String> {
    let start = text.find("\"/")?;
    let rest = &text[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn route_registration_contains_business_logic_line(
    file: &ParsedFile,
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    if !is_router_file(file) {
        return None;
    }
    if function.go_evidence().gorm_query_chains.is_empty() && function.go_evidence().db_query_calls.is_empty() {
        return None;
    }
    Some(
        function
            .go_evidence()
            .gorm_query_chains
            .first()
            .map(|chain| chain.line)
            .or_else(|| function.go_evidence().db_query_calls.first().map(|call| call.line))
            .or_else(|| lines.first().map(|line| line.line))
            .unwrap_or(function.fingerprint.start_line),
    )
}

fn router_constructs_dependencies_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| constructor_dependency_line(&line.text) || line.text.contains("http.Client{"))
        .map(|line| line.line)
}

fn background_worker_registration_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("startworker")
                || lower.contains("schedule")
                || lower.contains("cron")
                || lower.contains("go ")
        })
        .map(|line| line.line)
}

fn admin_debug_route_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("\"/admin") || line.text.contains("\"/debug"))
        .map(|line| line.line)
}

fn danger_named(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("hard") || lower.contains("purge") || lower.contains("admin") || lower.contains("danger")
}

fn external_http_inside_transaction_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    action_inside_transaction_line(
        lines,
        &import_aliases_for(file, "net/http")
            .iter()
            .flat_map(|alias| [format!("{alias}.Get("), format!("{alias}.Post("), format!("{alias}.Do(")])
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )
}

fn action_inside_transaction_line(
    lines: &[super::framework_patterns::BodyLine],
    markers: &[&str],
) -> Option<usize> {
    let start = lines.iter().find(|line| transaction_start_line(&line.text))?.line;
    let end = lines
        .iter()
        .find(|line| line.line > start && (line.text.contains(".Commit(") || line.text.contains(".Rollback(")))
        .map(|line| line.line)
        .unwrap_or(usize::MAX);
    lines
        .iter()
        .find(|line| line.line > start && line.line < end && markers.iter().any(|marker| line.text.contains(marker)))
        .map(|line| line.line)
}

fn is_main_or_cmd_file(file: &ParsedFile) -> bool {
    file_matches_any_role(file, &["cmd", "main"])
}

fn route_registration_line_in_file(file: &ParsedFile) -> Option<usize> {
    file.module_scope_calls
        .iter()
        .find(|call| is_route_registration_name(&call.name))
        .map(|call| call.line)
        .or_else(|| {
            file.functions.iter().find_map(|function| {
                body_lines(function)
                    .iter()
                    .find(|line| {
                        [".GET(", ".POST(", ".PUT(", ".PATCH(", ".DELETE(", ".Any(", ".Use("]
                            .iter()
                            .any(|marker| line.text.contains(marker))
                    })
                    .map(|line| line.line)
            })
        })
}

fn is_route_registration_name(name: &str) -> bool {
    matches!(name, "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "Any" | "Use" | "Group")
}

fn first_domain_constant_line(file: &ParsedFile) -> Option<usize> {
    file.top_level_bindings
        .iter()
        .find(|binding| {
            let name = binding.name.as_str();
            let value = binding.value_text.to_ascii_lowercase();
            (name.ends_with("Status")
                || name.ends_with("State")
                || name.ends_with("Type")
                || name.ends_with("Role"))
                && (value.contains('"') || value.contains("iota"))
        })
        .map(|binding| binding.line)
        .or_else(|| {
            file.pkg_strings
                .iter()
                .find(|literal| {
                    let name = literal.name.as_str();
                    name.ends_with("Status")
                        || name.ends_with("State")
                        || name.ends_with("Type")
                        || name.ends_with("Role")
                })
                .map(|literal| literal.line)
        })
}

fn first_mapper_symbol_line(file: &ParsedFile) -> Option<usize> {
    file.go_structs()
        .iter()
        .find(|go_struct| go_struct.name.ends_with("Mapper"))
        .map(|go_struct| go_struct.line)
        .or_else(|| {
            file.functions
                .iter()
                .find(|function| {
                    let name = function.fingerprint.name.as_str();
                    name.starts_with("Map")
                        || name.contains("ToModel")
                        || name.contains("ToDTO")
                        || name.contains("ToResponse")
                })
                .map(|function| function.fingerprint.start_line)
        })
}

fn struct_has_sql_null_field(go_struct: &crate::analysis::GoStructSummary) -> bool {
    go_struct
        .fields
        .iter()
        .any(|field| field.type_text.contains("sql.Null"))
}

fn first_non_pointer_patch_field(go_struct: &crate::analysis::GoStructSummary) -> Option<usize> {
    if !(go_struct.name.contains("Patch") || go_struct.name.contains("Update")) {
        return None;
    }

    go_struct
        .fields
        .iter()
        .find(|field| {
            let ty = field.type_text.trim();
            !ty.starts_with('*')
                && !ty.starts_with("[]")
                && !ty.contains("map[")
                && !ty.contains("sql.Null")
                && matches!(
                    ty,
                    "string"
                        | "bool"
                        | "int"
                        | "int32"
                        | "int64"
                        | "uint"
                        | "uint32"
                        | "uint64"
                        | "float32"
                        | "float64"
                        | "time.Time"
                )
        })
        .map(|field| field.line)
}

fn model_has_calculated_field(go_struct: &crate::analysis::GoStructSummary) -> Option<usize> {
    go_struct
        .fields
        .iter()
        .find(|field| {
            let name = field.name.as_str();
            name.ends_with("Display")
                || name.ends_with("Label")
                || name.ends_with("URL")
                || name.ends_with("Count")
                || name.ends_with("Summary")
        })
        .map(|field| field.line)
}

fn model_spans_multiple_subdomains(go_struct: &crate::analysis::GoStructSummary) -> bool {
    if go_struct.fields.len() < 10 {
        return false;
    }

    let mut domains = BTreeSet::new();
    for field in &go_struct.fields {
        let lower = field.name.to_ascii_lowercase();
        if ["billing", "invoice", "card"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("billing");
        }
        if ["shipping", "address", "delivery"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("shipping");
        }
        if ["profile", "avatar", "bio"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("profile");
        }
        if ["inventory", "stock", "sku"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("inventory");
        }
        if ["tenant", "org", "workspace"].iter().any(|marker| lower.contains(marker)) {
            domains.insert("tenant");
        }
    }

    domains.len() >= 3
}

fn repeated_tagged_struct_use(
    tag_map: &BTreeMap<&str, Vec<&StructTag>>,
    go_struct: &crate::analysis::GoStructSummary,
    markers: &[&str],
) -> bool {
    has_any_tag_for_struct(tag_map, &go_struct.name, markers)
}

fn handler_recovery_line(function: &ParsedFunction) -> Option<usize> {
    body_lines(function)
        .iter()
        .find(|line| line.text.contains("recover()") || line.text.contains("recover("))
        .map(|line| line.line)
}

fn request_identity_extraction_line(
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("MustGet(\"user")
                || line.text.contains("MustGet(\"tenant")
                || line.text.contains("Get(\"user")
                || line.text.contains("Get(\"tenant")
                || line.text.contains("GetString(\"request_id")
        })
        .map(|line| line.line)
}

fn request_id_generation_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("uuid.new") || lower.contains("request-id") || lower.contains("request_id")
        })
        .map(|line| line.line)
}

fn pagination_binding_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("Query(\"page")
                || line.text.contains("DefaultQuery(\"page")
                || line.text.contains("Query(\"page_size")
                || line.text.contains("DefaultQuery(\"page_size")
        })
        .map(|line| line.line)
}

fn response_envelope_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("gin.H{\"data\"")
                || line.text.contains("gin.H{\"meta\"")
                || (line.text.contains("Response{") && line.text.contains("Data:"))
        })
        .map(|line| line.line)
}

fn required_validation_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("== \"\"")
                || lower.contains("== 0")
                || lower.contains("len(") && lower.contains("== 0")
        })
        .map(|line| line.line)
}

fn validation_error_shape(lines: &[super::framework_patterns::BodyLine]) -> Option<&'static str> {
    lines.iter().find_map(|line| {
        if line.text.contains("\"errors\"") {
            Some("errors")
        } else if line.text.contains("\"error\"") {
            Some("error")
        } else if line.text.contains("\"message\"") {
            Some("message")
        } else {
            None
        }
    })
}

fn default_injection_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            lower.contains("pagesize =")
                || lower.contains("page_size =")
                || lower.contains("page = 1")
                || lower.contains("limit =")
        })
        .map(|line| line.line)
}

fn path_param_parse_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains("strconv.Atoi(") || line.text.contains("uuid.Parse("))
                && line.text.contains(".Param(")
        })
        .map(|line| line.line)
}

fn pagination_bounds_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            (lower.contains("pagesize") || lower.contains("page_size"))
                && (lower.contains(">") || lower.contains("<"))
        })
        .map(|line| line.line)
}

fn sort_whitelist_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            let lower = line.text.to_ascii_lowercase();
            (lower.contains("sort") || lower.contains("filter"))
                && (lower.contains("switch") || lower.contains("allowed") || lower.contains("whitelist"))
        })
        .map(|line| line.line)
}

fn route_param_merge_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let has_bind = lines.iter().any(|line| {
        let lower = line.text.to_ascii_lowercase();
        lower.contains("bind") || lower.contains("shouldbind")
    });
    if !has_bind {
        return None;
    }
    lines
        .iter()
        .find(|line| line.text.contains(".Param(") || line.text.contains(".Query("))
        .map(|line| line.line)
}

fn upload_write_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let has_upload_validation = lines.iter().any(|line| line.text.contains("FormFile(") || line.text.contains("MultipartForm("));
    if !has_upload_validation {
        return None;
    }
    lines
        .iter()
        .find(|line| line.text.contains("SaveUploadedFile(") || line.text.contains("io.Copy("))
        .map(|line| line.line)
}

fn route_param_drift_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains(".Param(\"id\")") && line.text.contains("UserID"))
                || (line.text.contains(".Param(\"user_id\")") && line.text.contains("ID ="))
        })
        .map(|line| line.line)
}

fn error_mapping_line(
    file: &ParsedFile,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    if !lines.iter().any(|line| line.text.contains("errors.Is(") || line.text.contains("errors.As(")) {
        return None;
    }
    first_http_status_line(file, lines)
}

fn handler_error_shape_key(lines: &[super::framework_patterns::BodyLine]) -> Option<&'static str> {
    lines.iter().find_map(|line| {
        if line.text.contains("\"error\"") && line.text.contains("\"code\"") {
            Some("error+code")
        } else if line.text.contains("\"errors\"") {
            Some("errors")
        } else if line.text.contains("\"message\"") && line.text.contains("\"status\"") {
            Some("message+status")
        } else {
            None
        }
    })
}

fn error_string_switch_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("err.Error()"))
        .map(|line| line.line)
}

fn raw_db_error_response_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("err.Error()")
                && (line.text.contains("c.JSON(") || line.text.contains("gin.H") || line.text.contains("http.Error("))
        })
        .map(|line| line.line)
}

fn success_payload_with_error_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("\"data\"") && line.text.contains("\"error\""))
        .map(|line| line.line)
}

fn repeated_named_literals(file: &ParsedFile, matcher: impl Fn(&NamedLiteral) -> bool) -> Option<(usize, String)> {
    let mut counts = BTreeMap::<String, usize>::new();
    let mut lines = BTreeMap::<String, usize>::new();
    for literal in &file.pkg_strings {
        if matcher(literal) {
            *counts.entry(literal.value.clone()).or_default() += 1;
            lines.entry(literal.value.clone()).or_insert(literal.line);
        }
    }
    counts
        .into_iter()
        .find(|(_, count)| *count >= 2)
        .and_then(|(value, _)| lines.get(&value).copied().map(|line| (line, value)))
}

fn table_or_column_literal(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    !lower.contains(' ')
        && (lower.ends_with("s") || lower.ends_with("_id") || lower.ends_with("_at"))
        && lower.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_' || ch.is_ascii_digit())
}

fn gorm_scope_count(function: &ParsedFunction) -> usize {
    body_lines(function)
        .iter()
        .map(|line| line.text.match_indices("Scopes(func").count())
        .sum()
}

fn preload_count(function: &ParsedFunction) -> usize {
    let parsed = function
        .go_evidence()
        .gorm_query_chains
        .iter()
        .flat_map(|chain| chain.steps.iter())
        .filter(|step| step.method_name == "Preload")
        .count();
    if parsed > 0 {
        parsed
    } else {
        body_lines(function)
            .iter()
            .map(|line| line.text.match_indices(".Preload(").count())
            .sum()
    }
}

fn soft_delete_line_count(lines: &[super::framework_patterns::BodyLine]) -> usize {
    lines
        .iter()
        .map(|line| {
            let lower = line.text.to_ascii_lowercase();
            if lower.contains("deleted_at") && lower.contains("is null") {
                lower.match_indices("deleted_at").count()
            } else {
                0
            }
        })
        .sum()
}

fn locking_clause_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("Locking") || line.text.contains("FOR UPDATE"))
        .map(|line| line.line)
}

fn update_struct_without_field_intent_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains(".Updates(") || line.text.contains(".Save("))
                && !line.text.contains("Select(")
                && !line.text.contains("Omit(")
        })
        .map(|line| line.line)
}

fn map_update_flow_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let has_map = lines
        .iter()
        .any(|line| line.text.contains("map[string]any{") || line.text.contains("map[string]interface{}{"));
    if !has_map {
        return None;
    }
    lines
        .iter()
        .find(|line| line.text.contains("Update(") || line.text.contains("Updates("))
        .map(|line| line.line)
}

fn base_repository_reflection_line(
    function: &ParsedFunction,
    lines: &[super::framework_patterns::BodyLine],
) -> Option<usize> {
    if function.fingerprint.name.contains("Base")
        || function.fingerprint.name.contains("Generic")
        || function.fingerprint.receiver_type.as_ref().is_some_and(|receiver| receiver.contains("Base") || receiver.contains("Generic"))
    {
        return lines
            .iter()
            .find(|line| line.text.contains("reflect."))
            .map(|line| line.line);
    }
    None
}

fn transaction_cross_layer_line(function: &ParsedFunction, lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    if !signature_mentions_transaction(&function.signature_text) {
        return None;
    }
    lines
        .iter()
        .find(|line| line.text.contains("svc.") || line.text.contains("service.") || line.text.contains("repo."))
        .map(|line| line.line)
}

fn cross_repo_write_without_tx_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let repo_refs = repository_receiver_count(lines);
    let has_write = lines.iter().any(|line| {
        line.text.contains(".Create(")
            || line.text.contains(".Save(")
            || line.text.contains(".Delete(")
            || line.text.contains(".Updates(")
    });
    if repo_refs >= 2 && has_write && !lines.iter().any(|line| transaction_start_line(&line.text)) {
        return lines.first().map(|line| line.line);
    }
    None
}

fn split_commit_rollback_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let has_commit = lines.iter().any(|line| line.text.contains(".Commit("));
    let has_rollback = lines.iter().any(|line| line.text.contains(".Rollback("));
    if has_commit && has_rollback && !lines.iter().any(|line| line.text.contains("defer")) {
        lines.iter()
            .find(|line| line.text.contains(".Commit(") || line.text.contains(".Rollback("))
            .map(|line| line.line)
    } else {
        None
    }
}

fn nested_tx_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("SavePoint(") || line.text.contains("RollbackTo("))
        .map(|line| line.line)
        .or_else(|| {
            let tx_begins = lines
                .iter()
                .filter(|line| transaction_start_line(&line.text))
                .count();
            if tx_begins >= 2 {
                lines.iter().find(|line| transaction_start_line(&line.text)).map(|line| line.line)
            } else {
                None
            }
        })
}

fn mutable_config_package_var(file: &ParsedFile) -> Option<usize> {
    file.package_vars()
        .iter()
        .find(|var| {
            let lower = var.name.to_ascii_lowercase();
            (lower.contains("config") || lower.contains("setting"))
                && !var.value_text.as_deref().unwrap_or_default().contains("const")
        })
        .map(|var| var.line)
}

fn feature_flag_lookup_line(file: &ParsedFile, lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    first_matching_line(
        lines,
        &config_lookup_patterns(file)
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )
}

fn lifecycle_start_without_shutdown_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let start_line = lines
        .iter()
        .find(|line| line.text.contains(".Run(") || line.text.contains("ListenAndServe("))
        .map(|line| line.line)?;
    let has_shutdown = lines.iter().any(|line| {
        line.text.contains("Shutdown(") || line.text.contains("Close(") || line.text.contains("Stop(")
    });
    if has_shutdown {
        None
    } else {
        Some(start_line)
    }
}

fn test_fixture_builder_line(function: &ParsedFunction) -> Option<usize> {
    let name = function.fingerprint.name.as_str();
    if name.starts_with("NewTest") || name.contains("Fixture") || name.starts_with("BuildTest") {
        Some(function.fingerprint.start_line)
    } else {
        None
    }
}

fn raw_json_assertion_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains("{\"") || line.text.contains("assert.JSONEq("))
                && (line.text.contains("assert.") || line.text.contains("require.") || line.text.contains("cmp."))
        })
        .map(|line| line.line)
}

fn gin_context_stub_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("gin.CreateTestContext("))
        .map(|line| line.line)
}

fn transport_test_repo_touch_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("NewRepo(") || line.text.contains("Repository{"))
        .map(|line| line.line)
}

fn sql_query_assertion_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains("SELECT ") || line.text.contains("UPDATE ") || line.text.contains("DELETE "))
                && (line.text.contains("assert.") || line.text.contains("require.") || line.text.contains("Expect("))
        })
        .map(|line| line.line)
}

fn table_driven_multi_domain_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            line.text.contains("[]struct")
                && (line.text.contains("user") || line.text.contains("order") || line.text.contains("invoice"))
        })
        .map(|line| line.line)
}

fn metrics_label_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| line.text.contains("prometheus.Labels{") || line.text.contains("WithLabelValues("))
        .map(|line| line.line)
}

fn tracing_span_name_literal(line: &str) -> Option<String> {
    let marker = "Start(";
    let idx = line.find(marker)?;
    let rest = &line[idx + marker.len()..];
    let first_quote = rest.find('"')?;
    let rest = &rest[first_quote + 1..];
    let end_quote = rest.find('"')?;
    Some(rest[..end_quote].to_string())
}

fn repository_log_http_metadata_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| {
            (line.text.contains("logger.") || line.text.contains("log."))
                && (line.text.contains("status") || line.text.contains("route") || line.text.contains("path"))
        })
        .map(|line| line.line)
}

fn audit_before_service_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let audit_line = lines
        .iter()
        .find(|line| line.text.to_ascii_lowercase().contains("audit"))
        .map(|line| line.line)?;
    let service_line = lines
        .iter()
        .find(|line| line.text.contains("svc.") || line.text.contains("service."))
        .map(|line| line.line)?;
    if audit_line < service_line {
        Some(audit_line)
    } else {
        None
    }
}

fn request_logging_field_key(line: &str) -> Option<&'static str> {
    if line.contains("\"request_id\"") {
        Some("request_id")
    } else if line.contains("\"requestId\"") {
        Some("requestId")
    } else if line.contains("\"user_id\"") {
        Some("user_id")
    } else if line.contains("\"userId\"") {
        Some("userId")
    } else {
        None
    }
}

fn health_handler_repo_line(function: &ParsedFunction, lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    let lower = function.fingerprint.name.to_ascii_lowercase();
    if !(lower.contains("health") || lower.contains("ready")) {
        return None;
    }
    if references_repository_directly(lines) {
        Some(function.fingerprint.start_line)
    } else {
        None
    }
}

fn migration_or_seed_handler_line(lines: &[super::framework_patterns::BodyLine]) -> Option<usize> {
    lines
        .iter()
        .find(|line| migration_line(&line.text) || line.text.contains("Seed("))
        .map(|line| line.line)
}

fn api_example_literal_line(function: &ParsedFunction) -> Option<usize> {
    function
        .local_strings
        .iter()
        .find(|literal| literal.value.contains("{\"") && literal.value.contains("example"))
        .map(|literal| literal.line)
        .or_else(|| {
            body_lines(function)
                .iter()
                .find(|line| line.text.contains("{\\\"example\\\"") || line.text.contains("{\"example\""))
                .map(|line| line.line)
        })
}

fn extract_constructor_name(text: &str) -> Option<String> {
    let start = text.find("New")?;
    let rest = &text[start..];
    let end = rest.find('(')?;
    let name = &rest[..end];
    if name.len() > 3
        && name.chars().next().is_some_and(|ch| ch == 'N')
        && name.chars().all(|ch| ch.is_ascii_alphanumeric())
    {
        Some(name.to_string())
    } else {
        None
    }
}

fn has_duplicate_string(values: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    values.iter().any(|value| !seen.insert(value.as_str()))
}

fn extract_status_constant(text: &str) -> Option<String> {
    for part in text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '.')) {
        if part.contains("Status") {
            return Some(part.to_string());
        }
    }
    None
}

fn extract_domain_error_name(text: &str) -> Option<String> {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .find(|token| token.starts_with("Err") && token.len() > 3)
        .map(ToString::to_string)
}
