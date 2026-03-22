use std::path::PathBuf;

use crate::analysis::Language;
use crate::model::{FileReport, FunctionFingerprint, SymbolKind};

#[derive(Debug, Clone)]
pub(crate) struct ParsedFile {
    pub language: Language,
    pub path: PathBuf,
    pub package_name: Option<String>,
    pub is_test_file: bool,
    pub syntax_error: bool,
    pub byte_size: usize,
    pub package_string_literals: Vec<NamedLiteral>,
    pub struct_tags: Vec<StructTag>,
    pub functions: Vec<ParsedFunction>,
    pub imports: Vec<ImportSpec>,
    pub symbols: Vec<DeclaredSymbol>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedFunction {
    pub fingerprint: FunctionFingerprint,
    pub calls: Vec<CallSite>,
    pub has_context_parameter: bool,
    pub is_test_function: bool,
    pub local_binding_names: Vec<String>,
    pub doc_comment: Option<String>,
    pub local_string_literals: Vec<NamedLiteral>,
    pub test_summary: Option<TestFunctionSummary>,
    pub safety_comment_lines: Vec<usize>,
    pub unsafe_lines: Vec<usize>,
    pub dropped_error_lines: Vec<usize>,
    pub panic_on_error_lines: Vec<usize>,
    pub errorf_calls: Vec<FormattedErrorCall>,
    pub context_factory_calls: Vec<ContextFactoryCall>,
    pub goroutine_launch_lines: Vec<usize>,
    pub goroutine_in_loop_lines: Vec<usize>,
    pub goroutine_without_shutdown_lines: Vec<usize>,
    pub sleep_in_loop_lines: Vec<usize>,
    pub busy_wait_lines: Vec<usize>,
    pub mutex_lock_in_loop_lines: Vec<usize>,
    pub allocation_in_loop_lines: Vec<usize>,
    pub fmt_in_loop_lines: Vec<usize>,
    pub reflection_in_loop_lines: Vec<usize>,
    pub string_concat_in_loop_lines: Vec<usize>,
    pub json_marshal_in_loop_lines: Vec<usize>,
    pub db_query_calls: Vec<DbQueryCall>,
}

#[derive(Debug, Clone)]
pub(crate) struct ContextFactoryCall {
    pub line: usize,
    pub cancel_name: String,
    pub factory_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct DbQueryCall {
    pub line: usize,
    pub receiver: Option<String>,
    pub method_name: String,
    pub query_text: Option<String>,
    pub query_argument_text: Option<String>,
    pub query_uses_dynamic_construction: bool,
    pub in_loop: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct NamedLiteral {
    pub line: usize,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub(crate) struct StructTag {
    pub line: usize,
    pub struct_name: String,
    pub field_name: String,
    pub raw_tag: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TestFunctionSummary {
    pub assertion_like_calls: usize,
    pub error_assertion_calls: usize,
    pub skip_calls: usize,
    pub production_calls: usize,
    pub has_todo_marker: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct FormattedErrorCall {
    pub line: usize,
    pub format_string: Option<String>,
    pub mentions_err: bool,
    pub uses_percent_w: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct CallSite {
    pub receiver: Option<String>,
    pub name: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct ImportSpec {
    pub alias: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct DeclaredSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub receiver_type: Option<String>,
    pub receiver_is_pointer: Option<bool>,
    pub line: usize,
}

impl ParsedFile {
    pub fn to_report(&self) -> FileReport {
        FileReport {
            path: self.path.clone(),
            package_name: self.package_name.clone(),
            syntax_error: self.syntax_error,
            byte_size: self.byte_size,
            functions: self
                .functions
                .iter()
                .map(|function| function.fingerprint.clone())
                .collect(),
        }
    }
}
