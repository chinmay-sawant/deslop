use super::FixtureWorkspace;

#[test]
fn test_go_fingerprints() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("main.go", go_fixture!("simple.go"));

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(report.functions_found, 2);
    assert!(report.parse_failures.is_empty());
    assert_eq!(report.files[0].package_name.as_deref(), Some("sample"));
    assert!(report.findings.is_empty());

    let names = report.files[0]
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["Add", "Run"]);
}

#[test]
fn respects_gitignore() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(".gitignore", "ignored.go\n");
    workspace.write_file("main.go", go_fixture!("simple.go"));
    workspace.write_file("ignored.go", go_fixture!("simple.go"));

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
}

#[test]
fn test_generated_syntax() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("generated.go", go_fixture!("generated.go"));
    workspace.write_file("broken.go", go_fixture!("malformed.txt"));

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 1);
    assert_eq!(
        report.files[0]
            .path
            .file_name()
            .and_then(|name| name.to_str()),
        Some("broken.go")
    );
    assert!(report.files[0].syntax_error);
}
