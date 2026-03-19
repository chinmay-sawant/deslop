use std::path::Path;

use crate::model::SymbolKind;

use super::{
    comments::extract_doc_comment,
    general::package_alias_from_import_path,
    parse_file,
};

#[test]
fn derives_import_alias_from_path() {
    assert_eq!(
        package_alias_from_import_path("github.com/acme/utils"),
        "utils"
    );
}

#[test]
fn collects_package_level_function_alias_vars_as_symbols() {
    let source = r#"package pdf

import font "example.com/font"

var (
    IsCustomFont = font.IsCustomFont
    PlainValue = 42
)

func collectAllStandardFontsInTemplate() {
    IsCustomFont("Helvetica")
}
"#;

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "IsCustomFont" && matches!(symbol.kind, SymbolKind::Function)
    }));
    assert!(!parsed.symbols.iter().any(|symbol| symbol.name == "PlainValue"));
}

#[test]
fn extracts_doc_comment_text() {
    let source = "// Run Processes The Input\n// This function does X by doing Y because Z\nfunc Run() {}\n";

    let comment = extract_doc_comment(source, 2).expect("doc comment should exist");
    assert_eq!(
        comment,
        "Run Processes The Input\nThis function does X by doing Y because Z"
    );
}

#[test]
fn collects_error_handling_signals() {
    let source = r#"package sample

import (
    "fmt"
    "log"
)

func Run(err error) error {
    _ = err
    if err != nil {
        panic(err)
    }
    return fmt.Errorf("wrap: %v", err)
}

func LogOnly(err error) {
    if err != nil {
        log.Fatal(err)
    }
}
"#;

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let run = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Run")
        .expect("Run should be parsed");
    let log_only = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "LogOnly")
        .expect("LogOnly should be parsed");

    assert_eq!(run.dropped_error_lines, vec![9]);
    assert_eq!(run.panic_on_error_lines, vec![10]);
    assert_eq!(run.errorf_calls.len(), 1);
    assert!(run.errorf_calls[0].mentions_err);
    assert!(!run.errorf_calls[0].uses_percent_w);
    assert_eq!(log_only.panic_on_error_lines, vec![17]);
}

#[test]
fn collects_context_and_sleep_signals() {
    let source = r#"package sample

import (
    "context"
    "time"
)

func Poll(ctx context.Context) {
    for {
        time.Sleep(time.Second)
        _ = ctx
    }
}
"#;

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let poll = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Poll")
        .expect("Poll should be parsed");

    assert!(poll.has_context_parameter);
    assert_eq!(poll.sleep_in_loop_lines, vec![10]);
}

#[test]
fn collects_context_factory_busy_wait_and_json_signals() {
    let source = r#"package sample

import (
    "context"
    "encoding/json"
    "time"
)

func Run(parent context.Context, items []string) {
    ctx, cancel := context.WithTimeout(parent, time.Second)
    _ = ctx

    for {
        select {
        default:
            return
        }
    }

    for _, item := range items {
        _, _ = json.Marshal(item)
    }

    _ = cancel
}
"#;

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let run = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Run")
        .expect("Run should be parsed");

    assert_eq!(run.context_factory_calls.len(), 1);
    assert_eq!(run.context_factory_calls[0].cancel_name, "cancel");
    assert_eq!(run.context_factory_calls[0].factory_name, "WithTimeout");
    assert_eq!(run.busy_wait_lines, vec![14]);
    assert_eq!(run.json_marshal_in_loop_lines, vec![21]);
}

#[test]
fn collects_concurrency_and_db_signals() {
    let source = r#"package sample

import (
    "context"
    "fmt"
    "reflect"
    "time"
)

func Run(ctx context.Context, db Queryer, items []string, mu MutexLike) {
    go func() {
        for {
            _ = ctx
        }
    }()

    for _, item := range items {
        mu.Lock()
        time.Sleep(time.Millisecond)
        _, _ = db.QueryContext(ctx, "SELECT * FROM widgets WHERE name LIKE '%foo%'")
        _ = fmt.Sprintf("%s", item)
        _ = reflect.TypeOf(item)
        _ = make([]byte, 16)
        mu.Unlock()
    }
}

type Queryer interface {
    QueryContext(context.Context, string) (any, error)
}

type MutexLike interface {
    Lock()
    Unlock()
}
"#;

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let run = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Run")
        .expect("Run should be parsed");

    assert_eq!(run.goroutine_without_shutdown_lines, vec![11]);
    assert_eq!(run.mutex_lock_in_loop_lines, vec![18]);
    assert_eq!(run.allocation_in_loop_lines, vec![23]);
    assert_eq!(run.fmt_in_loop_lines, vec![21]);
    assert_eq!(run.reflection_in_loop_lines, vec![22]);
    assert_eq!(run.db_query_calls.len(), 1);
    assert!(run.db_query_calls[0].in_loop);
    assert_eq!(
        run.db_query_calls[0].query_text.as_deref(),
        Some("SELECT * FROM widgets WHERE name LIKE '%foo%'")
    );
}

#[test]
fn collects_string_concat_and_goroutine_signals() {
    let source = r#"package sample

func Build(parts []string) string {
    out := ""
    for _, part := range parts {
        out += part
        go notify(part)
    }
    return out
}

func notify(value string) {}
"#;

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let build = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "Build")
        .expect("Build should be parsed");

    assert_eq!(build.string_concat_in_loop_lines, vec![6]);
    assert_eq!(build.goroutine_launch_lines, vec![7]);
}
