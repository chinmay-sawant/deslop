use std::fs;
use std::path::Path;

use deslop::{benchmark_repository, scan_repository, BenchmarkOptions, ScanOptions};

use super::{create_temp_workspace, write_fixture};

const GOMINDMAPPER_ROOT: &str = "/home/chinmay/ChinmayPersonalProjects/mindmapper/gomindmapper";
const GOPDFSUIT_ROOT: &str = "/home/chinmay/ChinmayPersonalProjects/gopdfsuit";

#[test]
#[ignore]
fn scans_real_projects_and_prints_reports() {
    let roots = [Path::new(GOMINDMAPPER_ROOT), Path::new(GOPDFSUIT_ROOT)];

    for root in roots {
        let report = scan_repository(&ScanOptions {
            root: root.to_path_buf(),
            respect_ignore: true,
        })
        .unwrap_or_else(|error| panic!("scan should succeed for {}: {error}", root.display()));

        assert!(report.files_discovered > 0, "{} should contain Go files", root.display());
        assert!(
            report.files_analyzed > 0,
            "{} should contain analyzable Go files",
            root.display()
        );

        println!("scan report for {}", root.display());
        println!("  files_discovered: {}", report.files_discovered);
        println!("  files_analyzed: {}", report.files_analyzed);
        println!("  functions_found: {}", report.functions_found);
        println!("  findings: {}", report.findings.len());
        println!("  parse_failures: {}", report.parse_failures.len());
        println!(
            "  index_summary: packages={} symbols={} imports={}",
            report.index_summary.package_count,
            report.index_summary.symbol_count,
            report.index_summary.import_count
        );
        println!(
            "  timings_ms: discover={} parse={} index={} heuristics={} total={}",
            report.timings.discover_ms,
            report.timings.parse_ms,
            report.timings.index_ms,
            report.timings.heuristics_ms,
            report.timings.total_ms
        );
    }
}

#[test]
fn benchmarks_a_real_scan_path() {
    let temp_dir = create_temp_workspace();
    write_fixture(
        &temp_dir,
        "main.go",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/go/simple.go")),
    );

    let report = benchmark_repository(&BenchmarkOptions {
        root: temp_dir.clone(),
        repeats: 2,
        warmups: 1,
        respect_ignore: true,
    })
    .expect("benchmark should succeed");

    assert_eq!(report.repeats, 2);
    assert_eq!(report.warmups, 1);
    assert_eq!(report.runs.len(), 2);

    fs::remove_dir_all(temp_dir).expect("temp dir cleanup should succeed");
}
