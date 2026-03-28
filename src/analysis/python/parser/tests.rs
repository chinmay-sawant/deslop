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
    assert!(fetch.exception_handlers.is_empty());
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

#[test]
fn test_python_exception_handler_evidence() {
    let source = r#"
def load_config():
    try:
        return read_config()
    except Exception:
        pass

def recover_config():
    try:
        return read_config()
    except ValueError:
        return default_config()
"#;

    let parsed = parse_file(Path::new("config.py"), source).expect("python parsing should succeed");

    let load_config = &parsed.functions[0];
    assert_eq!(load_config.exception_handlers.len(), 1);
    assert!(load_config.exception_handlers[0].is_broad);
    assert!(load_config.exception_handlers[0].suppresses);
    assert_eq!(
        load_config.exception_handlers[0].action.as_deref(),
        Some("pass")
    );

    let recover_config = &parsed.functions[1];
    assert_eq!(recover_config.exception_handlers.len(), 1);
    assert!(!recover_config.exception_handlers[0].is_broad);
}

#[test]
fn test_python_phase4_parser_evidence() {
    let source = r#"
def process_items(items, *args, **kwargs):
    if items == None:
        return None

    [emit(item) for item in items]
    first = list(items)[0]
    queue = [first]
    queue.pop(0)
    total = 0
    blocked = [first]
    for item in items:
        scratch = []
        scratch.append(item)
        if item in blocked:
            continue
        if len(blocked) > 0 and len(blocked) < 3:
            total += len(item)
    return first

def scan_tree(node):
    for child in node.children:
        scan_tree(child)

def read_config(path):
    handle = open(path)
    return handle.read()

class BaseManager:
    pass

class PayloadManager(BaseManager):
    def __init__(self):
        self.alpha = 1
        self.beta = 2
        self.gamma = 3
        self.delta = 4
        self.epsilon = 5
        self.zeta = 6
        self.eta = 7
        self.theta = 8
        self.iota = 9
        self.kappa = 10
        self.client = Session()
        self.cache = CacheClient()
        self.reporter = Reporter()

    def render(self):
        return self.alpha

    def persist(self):
        return self.beta
"#;

    let parsed =
        parse_file(Path::new("pkg/service.py"), source).expect("python parsing should succeed");

    let process_items = &parsed.functions[0];
    assert_eq!(process_items.none_comparison_lines, vec![3]);
    assert_eq!(process_items.redundant_return_none_lines, vec![4]);
    assert_eq!(process_items.side_effect_comprehension_lines, vec![6]);
    assert_eq!(process_items.list_materialization_lines, vec![7]);
    assert_eq!(process_items.deque_operation_lines, vec![9]);
    assert_eq!(process_items.temp_collection_lines, vec![13]);
    assert_eq!(process_items.list_membership_loop_lines, vec![15]);
    assert_eq!(process_items.repeated_len_loop_lines, vec![12]);
    assert_eq!(process_items.builtin_candidate_lines, vec![12]);
    assert!(process_items.has_varargs);
    assert!(process_items.has_kwargs);
    assert!(!process_items.has_complete_type_hints);
    assert!(process_items.validation_signature.is_some());
    assert!(!process_items.normalized_body.is_empty());

    let scan_tree = &parsed.functions[1];
    assert_eq!(scan_tree.recursive_call_lines, vec![23]);

    let read_config = &parsed.functions[2];
    assert_eq!(read_config.missing_context_manager_lines, vec![26]);

    assert_eq!(parsed.class_summaries.len(), 2);
    let payload_manager = &parsed.class_summaries[1];
    assert_eq!(payload_manager.name, "PayloadManager");
    assert_eq!(payload_manager.method_count, 3);
    assert_eq!(payload_manager.instance_attribute_count, 13);
    assert_eq!(payload_manager.public_method_count, 2);
    assert_eq!(payload_manager.base_classes, vec!["BaseManager"]);
    assert_eq!(payload_manager.constructor_collaborator_count, 3);
}

#[test]
fn test_python_init_reexports_are_indexed_as_symbols() {
    let source = r#"
from .types import WidgetTemplate, LayoutConfig, Heading
from .generator import render_widget
"#;

    let parsed =
        parse_file(Path::new("pkg/widgets/__init__.py"), source)
            .expect("python parsing should succeed");

    assert!(parsed.imports.iter().all(|import| import.is_public));
    assert!(parsed.symbols.iter().any(|symbol| symbol.name == "WidgetTemplate"));
    assert!(parsed.symbols.iter().any(|symbol| symbol.name == "LayoutConfig"));
    assert!(parsed.symbols.iter().any(|symbol| symbol.name == "Heading"));
    assert!(parsed.symbols.iter().any(|symbol| symbol.name == "render_widget"));
}
