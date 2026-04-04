use super::FixtureWorkspace;

#[test]
fn test_unmanaged_goroutines() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("go_routine.go", go_fixture!("goroutine_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_without_coordination")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_spawn_in_loop")
    );
}

#[test]
fn test_coordination() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("go_routine.go", go_fixture!("goroutine_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_without_coordination")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_spawn_in_loop")
    );
}

#[test]
fn test_shutdown_mutex() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("concurrency.go", go_fixture!("concurrency_slop.txt"));

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_without_shutdown_path")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "mutex_in_loop")
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_call_while_locked")
    );
}

#[test]
fn test_no_slop() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file("concurrency.go", go_fixture!("concurrency_clean.txt"));

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_without_shutdown_path")
    );
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "blocking_call_while_locked")
    );
}

#[test]
fn test_deeper_goroutine_lifetime() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "goroutine_deeper.go",
        go_fixture!("goroutine_deeper_slop.txt"),
    );

    let report = workspace.scan();

    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_derived_context_unmanaged")
    );
}

#[test]
fn test_deeper_goroutine_lifetime_clean() {
    let workspace = FixtureWorkspace::new();
    workspace.write_file(
        "goroutine_deeper.go",
        go_fixture!("goroutine_deeper_clean.txt"),
    );

    let report = workspace.scan();

    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.rule_id == "goroutine_derived_context_unmanaged")
    );
}
