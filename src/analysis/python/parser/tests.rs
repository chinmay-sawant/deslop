use std::path::Path;

use super::parse_file;

#[test]
fn test_python_parser_extracts_functions_imports_and_strings() {
    let source = r#"
import requests as req
from pathlib import Path

API_TOKEN = "super-secret-value"

async def fetch_profile(user_id: str) -> str:
    """Fetch Profile Data.

    This docstring explains every implementation step.
    """
    password = "top-secret-token"
    result = ""
    for piece in ["a", "b"]:
        result += piece
    return req.get(user_id).text

class Renderer:
    def render(self, path: str) -> str:
        return Path(path).read_text()

def helper():
    def nested():
        return 1

    return print("ready")
"#;

    let parsed =
        parse_file(Path::new("pkg/service.py"), source).expect("python parsing should succeed");

    assert_eq!(parsed.package_name.as_deref(), Some("service"));
    assert_eq!(parsed.imports.len(), 2);
    assert_eq!(parsed.pkg_strings.len(), 1);
    assert_eq!(parsed.pkg_strings[0].name, "API_TOKEN");
    assert_eq!(parsed.functions.len(), 3);

    let fetch = &parsed.functions[0];
    assert_eq!(fetch.fingerprint.name, "fetch_profile");
    assert_eq!(fetch.fingerprint.kind, "async_function");
    assert_eq!(fetch.local_strings.len(), 2);
    assert_eq!(fetch.concat_loops, vec![15]);
    assert!(fetch.doc_comment.is_some());
    assert!(
        fetch
            .calls
            .iter()
            .any(|call| call.receiver.as_deref() == Some("req") && call.name == "get")
    );

    let render = &parsed.functions[1];
    assert_eq!(render.fingerprint.kind, "method");
    assert_eq!(
        render.fingerprint.receiver_type.as_deref(),
        Some("Renderer")
    );

    let helper = &parsed.functions[2];
    assert_eq!(helper.fingerprint.name, "helper");
    assert!(helper.calls.iter().any(|call| call.name == "print"));
}

#[test]
fn test_python_parser_marks_syntax_errors() {
    let source = r#"
def broken(
    return 1
"#;

    let parsed = parse_file(Path::new("broken.py"), source)
        .expect("python parsing should still return a parsed file");

    assert!(parsed.syntax_error);
}

#[test]
fn test_python_test_detection() {
    let source = r#"
class TestClient:
    def test_fetch(self):
        self.assertEqual(fetch(), 1)
"#;

    let parsed = parse_file(Path::new("tests/test_client.py"), source)
        .expect("python parsing should succeed");

    assert!(parsed.is_test_file);
    assert!(parsed.functions[0].is_test_function);
    assert!(parsed.functions[0].test_summary.is_some());
}
