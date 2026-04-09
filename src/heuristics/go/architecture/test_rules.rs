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
