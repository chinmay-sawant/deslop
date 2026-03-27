use std::collections::{BTreeMap, BTreeSet};

use crate::analysis::{CallSite, ImportSpec};

const SUSPICIOUS_GENERIC_NAMES: &[&str] = &[
    "processdata",
    "handlerequest",
    "executetask",
    "convertvalue",
    "validateinput",
    "transformdata",
    "parsedata",
    "formatresponse",
    "processrequest",
];

const GO_BUILTINS: &[&str] = &[
    "append", "cap", "clear", "close", "complex", "copy", "delete", "imag", "len", "make", "max",
    "min", "new", "panic", "print", "println", "real", "recover",
];
const PYTHON_BUILTINS: &[&str] = &[
    "abs",
    "all",
    "any",
    "ascii",
    "bin",
    "bool",
    "breakpoint",
    "bytearray",
    "bytes",
    "callable",
    "chr",
    "classmethod",
    "compile",
    "complex",
    "delattr",
    "dict",
    "dir",
    "divmod",
    "enumerate",
    "eval",
    "exec",
    "filter",
    "float",
    "format",
    "frozenset",
    "getattr",
    "globals",
    "hasattr",
    "hash",
    "help",
    "hex",
    "id",
    "input",
    "int",
    "isinstance",
    "issubclass",
    "iter",
    "len",
    "list",
    "locals",
    "map",
    "max",
    "memoryview",
    "min",
    "next",
    "object",
    "oct",
    "open",
    "ord",
    "pow",
    "print",
    "property",
    "range",
    "repr",
    "reversed",
    "round",
    "set",
    "setattr",
    "slice",
    "sorted",
    "staticmethod",
    "str",
    "sum",
    "super",
    "tuple",
    "type",
    "vars",
    "zip",
];
const PYTHON_BUILTIN_EXCEPTIONS: &[&str] = &[
    "ArithmeticError",
    "AssertionError",
    "AttributeError",
    "BaseException",
    "BlockingIOError",
    "BrokenPipeError",
    "BufferError",
    "BytesWarning",
    "ChildProcessError",
    "ConnectionAbortedError",
    "ConnectionError",
    "ConnectionRefusedError",
    "ConnectionResetError",
    "DeprecationWarning",
    "EOFError",
    "EnvironmentError",
    "Exception",
    "FileExistsError",
    "FileNotFoundError",
    "FloatingPointError",
    "FutureWarning",
    "GeneratorExit",
    "IOError",
    "ImportError",
    "ImportWarning",
    "IndentationError",
    "IndexError",
    "InterruptedError",
    "IsADirectoryError",
    "KeyError",
    "KeyboardInterrupt",
    "LookupError",
    "MemoryError",
    "ModuleNotFoundError",
    "NameError",
    "NotADirectoryError",
    "NotImplementedError",
    "OSError",
    "OverflowError",
    "PendingDeprecationWarning",
    "PermissionError",
    "ProcessLookupError",
    "RecursionError",
    "ReferenceError",
    "ResourceWarning",
    "RuntimeError",
    "RuntimeWarning",
    "StopAsyncIteration",
    "StopIteration",
    "SyntaxError",
    "SyntaxWarning",
    "SystemError",
    "SystemExit",
    "TabError",
    "TimeoutError",
    "TypeError",
    "UnboundLocalError",
    "UnicodeDecodeError",
    "UnicodeEncodeError",
    "UnicodeError",
    "UnicodeTranslateError",
    "UnicodeWarning",
    "UserWarning",
    "ValueError",
    "Warning",
    "WindowsError",
    "ZeroDivisionError",
];

pub(super) fn import_alias_lookup(imports: &[ImportSpec]) -> BTreeMap<String, String> {
    let mut lookup = BTreeMap::new();
    for import in imports {
        lookup.insert(import.alias.clone(), import.path.clone());
        lookup.insert(import.path.clone(), import.path.clone());
    }
    lookup
}

pub(super) fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

pub(super) fn is_generic_name(name: &str) -> bool {
    if SUSPICIOUS_GENERIC_NAMES.contains(&name) {
        return true;
    }

    let generic_tokens = BTreeSet::from([
        "process",
        "handle",
        "execute",
        "convert",
        "validate",
        "transform",
        "parse",
        "format",
        "request",
        "response",
        "data",
        "input",
        "output",
        "task",
        "value",
    ]);
    generic_tokens
        .iter()
        .filter(|token| name.contains(*token))
        .count()
        >= 2
}

pub(super) fn is_builtin(name: &str) -> bool {
    GO_BUILTINS.contains(&name)
        || PYTHON_BUILTINS.contains(&name)
        || PYTHON_BUILTIN_EXCEPTIONS.contains(&name)
}

pub(super) fn is_global_sym(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

pub(super) fn identifier_token_count(name: &str) -> usize {
    identifier_tokens(name).len()
}

pub(super) fn identifier_tokens(name: &str) -> Vec<String> {
    let mut count = 0usize;
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut previous_was_separator = true;
    let mut previous_is_lower = false;

    for character in name.chars() {
        if character == '_' || character == '-' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            previous_was_separator = true;
            previous_is_lower = false;
            continue;
        }

        if !character.is_ascii_alphanumeric() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            previous_was_separator = true;
            previous_is_lower = false;
            continue;
        }

        if count == 0
            || previous_was_separator
            || character.is_ascii_uppercase() && previous_is_lower
        {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            count += 1;
        }

        current.push(character.to_ascii_lowercase());
        previous_was_separator = false;
        previous_is_lower = character.is_ascii_lowercase();
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

pub(super) fn is_title_doc(line: &str) -> bool {
    let words = line
        .split_whitespace()
        .map(|word| word.trim_matches(|character: char| !character.is_ascii_alphanumeric()))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();

    words.len() >= 3
        && !line.ends_with('.')
        && words.iter().all(|word| {
            word.chars().next().is_some_and(|character| {
                !character.is_ascii_alphabetic() || character.is_ascii_uppercase()
            })
        })
}

pub(super) fn is_tutorial_doc(comment: &str) -> bool {
    let normalized = comment.to_ascii_lowercase();
    comment.lines().count() >= 2
        && (normalized.contains("this function")
            || normalized.contains("this method")
            || normalized.contains("by doing")
            || normalized.contains("because"))
}

pub(super) fn is_blocking_call(call: &CallSite, import_aliases: &BTreeMap<String, String>) -> bool {
    if is_db_query(&call.name) {
        return true;
    }

    let Some(receiver) = &call.receiver else {
        return false;
    };
    let Some(import_path) = import_aliases.get(receiver) else {
        return false;
    };

    matches!(import_path.as_str(), "time") && call.name == "Sleep"
        || matches!(import_path.as_str(), "net/http")
            && matches!(
                call.name.as_str(),
                "Get" | "Head" | "Post" | "PostForm" | "Do"
            )
        || matches!(import_path.as_str(), "net")
            && matches!(call.name.as_str(), "Dial" | "DialTimeout" | "Listen")
        || matches!(import_path.as_str(), "os")
            && matches!(
                call.name.as_str(),
                "ReadFile" | "WriteFile" | "Open" | "OpenFile" | "Create"
            )
        || matches!(import_path.as_str(), "io") && call.name == "ReadAll"
}

pub(super) fn is_db_query(name: &str) -> bool {
    matches!(
        name,
        "Query"
            | "QueryContext"
            | "QueryRow"
            | "QueryRowContext"
            | "Exec"
            | "ExecContext"
            | "Get"
            | "Select"
            | "Raw"
            | "First"
            | "Find"
            | "Take"
            | "Preload"
    )
}
