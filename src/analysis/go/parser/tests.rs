use std::path::Path;

use crate::model::SymbolKind;

use super::{comments::extract_doc_comment, general::alias_from_path, parse_file};

#[test]
fn test_import_alias() {
    assert_eq!(alias_from_path("github.com/acme/utils"), "utils");
}

#[test]
fn test_alias_symbols() {
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
    assert!(
        !parsed
            .symbols
            .iter()
            .any(|symbol| symbol.name == "PlainValue")
    );
}

#[test]
fn test_doc_comment() {
    let source =
        "// Run Processes The Input\n// This function does X by doing Y because Z\nfunc Run() {}\n";

    let comment = extract_doc_comment(source, 2).expect("doc comment should exist");
    assert_eq!(
        comment,
        "Run Processes The Input\nThis function does X by doing Y because Z"
    );
}

#[test]
fn test_error_handling() {
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

    assert_eq!(run.dropped_errors, vec![9]);
    assert_eq!(run.panic_errors, vec![10]);
    assert_eq!(run.errorf_calls.len(), 1);
    assert!(run.errorf_calls[0].mentions_err);
    assert!(!run.errorf_calls[0].uses_percent_w);
    assert_eq!(log_only.panic_errors, vec![17]);
}

#[test]
fn test_ctx_sleep() {
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
    assert_eq!(poll.sleep_loops, vec![10]);
}

#[test]
fn test_ctx_busy_json() {
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
    assert_eq!(run.json_loops, vec![21]);
}

#[test]
fn test_concurrency_db() {
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

    assert_eq!(run.unmanaged_goroutines, vec![11]);
    assert_eq!(run.mutex_loops, vec![18]);
    assert_eq!(run.alloc_loops, vec![23]);
    assert_eq!(run.fmt_loops, vec![21]);
    assert_eq!(run.reflect_loops, vec![22]);
    assert_eq!(run.db_query_calls.len(), 1);
    assert!(run.db_query_calls[0].in_loop);
    assert_eq!(
        run.db_query_calls[0].query_text.as_deref(),
        Some("SELECT * FROM widgets WHERE name LIKE '%foo%'")
    );
}

#[test]
fn test_concat_goroutine() {
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

    assert_eq!(build.concat_loops, vec![6]);
    assert_eq!(build.goroutines, vec![7]);
    assert_eq!(build.loop_goroutines, vec![7]);
}

#[test]
fn test_pkg_literals() {
    let source = r#"package sample

const apiToken = "sk_test_1234567890"

type User struct {
    Name string `json:"name" db:"users.name" db:"duplicate"`
}

func (u User) NameValue() string {
    return u.Name
}

func (u *User) SetName(value string) {
    u.Name = value
}

func TestUser(t *testing.T) {
    token := "super-secret-value"
    _ = token
    got := buildUser()
    if err := got.Validate(); err != nil {
        t.Fatal(err)
    }
}

func buildUser() *User { return &User{} }
"#;

    let parsed = parse_file(Path::new("sample_test.go"), source).expect("parse should work");
    let test_fn = parsed
        .functions
        .iter()
        .find(|function| function.fingerprint.name == "TestUser")
        .expect("TestUser should be parsed");

    assert!(parsed.is_test_file);
    assert_eq!(parsed.pkg_strings.len(), 1);
    assert_eq!(parsed.pkg_strings[0].name, "apiToken");
    assert_eq!(parsed.struct_tags.len(), 1);
    assert_eq!(parsed.struct_tags[0].field_name, "Name");
    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "NameValue"
            && symbol.receiver_type.as_deref() == Some("User")
            && symbol.receiver_is_pointer == Some(false)
    }));
    assert!(parsed.symbols.iter().any(|symbol| {
        symbol.name == "SetName"
            && symbol.receiver_type.as_deref() == Some("User")
            && symbol.receiver_is_pointer == Some(true)
    }));
    assert_eq!(test_fn.local_strings.len(), 1);
    assert_eq!(test_fn.local_strings[0].name, "token");
    assert!(test_fn.test_summary.is_some());
    assert_eq!(
        test_fn
            .test_summary
            .as_ref()
            .map(|summary| summary.production_calls),
        Some(2)
    );
}

#[test]
fn test_imports_preserve_source_order_and_lines() {
    let source = r#"package sample

import (
    "github.com/acme/widgets"
    "fmt"
    "strings"
)

func Run() {}
"#;

    let parsed = parse_file(Path::new("sample.go"), source).expect("parse should work");
    let imports = parsed
        .imports
        .iter()
        .map(|import| (import.path.as_str(), import.line, import.group_line))
        .collect::<Vec<_>>();

    assert_eq!(
        imports,
        vec![
            ("github.com/acme/widgets", 4, 3),
            ("fmt", 5, 3),
            ("strings", 6, 3),
        ]
    );
}
