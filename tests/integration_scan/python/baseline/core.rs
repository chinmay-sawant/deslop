use super::FixtureWorkspace;

#[test]
fn test_python_fingerprints() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[("app.py", python_fixture!("simple.txt"))]);

    let report = workspace.scan();

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
}

#[test]
fn test_python_syntax() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[("broken.py", python_fixture!("broken.txt"))]);

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 1);
    assert_eq!(report.files_analyzed, 1);
    assert!(report.files[0].syntax_error);
    assert!(report.parse_failures.is_empty());
}

#[test]
fn test_python_mixed_repo() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        ("app.py", python_fixture!("simple.txt")),
        ("main.go", go_fixture!("simple.go")),
        ("src/main.rs", rust_fixture!("simple.txt")),
    ]);

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 3);
    assert_eq!(report.files_analyzed, 3);
    assert!(report.parse_failures.is_empty());

    let analyzed_paths = report
        .files
        .iter()
        .map(|file| {
            file.path
                .strip_prefix(workspace.root())
                .expect("report path should stay under the temp dir")
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(analyzed_paths, vec!["app.py", "main.go", "src/main.rs"]);
}

#[test]
fn test_python_rust_mixed_repo() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        ("pkg/render/service.py", python_fixture!("simple.txt")),
        ("pkg/render/lib.rs", rust_fixture!("simple.txt")),
    ]);

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 2);
    assert_eq!(report.files_analyzed, 2);
    assert!(report.parse_failures.is_empty());
}

#[test]
fn test_python_same_directory_mixed_repo() {
    let workspace = FixtureWorkspace::new();
    workspace.write_files(&[
        ("pkg/render/__init__.py", python_fixture!("simple.txt")),
        ("pkg/render/main.go", go_fixture!("simple.go")),
        ("pkg/render/lib.rs", rust_fixture!("simple.txt")),
    ]);

    let report = workspace.scan();

    assert_eq!(report.files_discovered, 3);
    assert_eq!(report.files_analyzed, 3);
    assert!(report.parse_failures.is_empty());
}
