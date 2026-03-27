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
    pub line_count: usize,
    pub byte_size: usize,
    pub pkg_strings: Vec<NamedLiteral>,
    pub comments: Vec<CommentSummary>,
    pub struct_tags: Vec<StructTag>,
    pub functions: Vec<ParsedFunction>,
    pub imports: Vec<ImportSpec>,
    pub symbols: Vec<DeclaredSymbol>,
    pub class_summaries: Vec<ClassSummary>,
    pub structs: Vec<StructSummary>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedFunction {
    pub fingerprint: FunctionFingerprint,
    pub calls: Vec<CallSite>,
    pub exception_handlers: Vec<ExceptionHandler>,
    pub has_context_parameter: bool,
    pub is_test_function: bool,
    pub local_binding_names: Vec<String>,
    pub doc_comment: Option<String>,
    pub body_text: String,
    pub local_strings: Vec<NamedLiteral>,
    pub normalized_body: String,
    pub validation_signature: Option<BlockFingerprint>,
    pub exception_block_signatures: Vec<BlockFingerprint>,
    pub test_summary: Option<TestFunctionSummary>,
    pub safety_comment_lines: Vec<usize>,
    pub unsafe_lines: Vec<usize>,
    pub dropped_errors: Vec<usize>,
    pub panic_errors: Vec<usize>,
    pub errorf_calls: Vec<FormattedErrorCall>,
    pub context_factory_calls: Vec<ContextFactoryCall>,
    pub goroutines: Vec<usize>,
    pub loop_goroutines: Vec<usize>,
    pub unmanaged_goroutines: Vec<usize>,
    pub sleep_loops: Vec<usize>,
    pub busy_wait_lines: Vec<usize>,
    pub mutex_loops: Vec<usize>,
    pub alloc_loops: Vec<usize>,
    pub fmt_loops: Vec<usize>,
    pub reflect_loops: Vec<usize>,
    pub concat_loops: Vec<usize>,
    pub json_loops: Vec<usize>,
    pub db_query_calls: Vec<DbQueryCall>,
    pub none_comparison_lines: Vec<usize>,
    pub side_effect_comprehension_lines: Vec<usize>,
    pub redundant_return_none_lines: Vec<usize>,
    pub list_materialization_lines: Vec<usize>,
    pub deque_operation_lines: Vec<usize>,
    pub temp_collection_lines: Vec<usize>,
    pub recursive_call_lines: Vec<usize>,
    pub list_membership_loop_lines: Vec<usize>,
    pub repeated_len_loop_lines: Vec<usize>,
    pub builtin_candidate_lines: Vec<usize>,
    pub missing_context_manager_lines: Vec<usize>,
    pub has_complete_type_hints: bool,
    pub has_varargs: bool,
    pub has_kwargs: bool,
    pub is_async: bool,
    pub await_points: Vec<usize>,
    pub macro_calls: Vec<MacroCall>,
    pub spawn_calls: Vec<RuntimeCall>,
    pub lock_calls: Vec<RuntimeCall>,
    pub permit_acquires: Vec<RuntimeCall>,
    pub futures_created: Vec<RuntimeCall>,
    pub blocking_calls: Vec<RuntimeCall>,
    pub select_macro_lines: Vec<usize>,
    pub drop_impl: bool,
    pub write_loops: Vec<usize>,
    pub line_iteration_loops: Vec<usize>,
    pub default_hasher_lines: Vec<usize>,
    pub boxed_container_lines: Vec<usize>,
    pub unsafe_soundness: Vec<UnsafePattern>,
}

#[derive(Debug, Clone)]
pub(crate) struct ClassSummary {
    pub name: String,
    pub line: usize,
    pub end_line: usize,
    pub method_count: usize,
    pub public_method_count: usize,
    pub instance_attribute_count: usize,
    pub base_classes: Vec<String>,
    pub constructor_collaborator_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct CommentSummary {
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct BlockFingerprint {
    pub line: usize,
    pub signature: String,
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
pub(crate) struct ExceptionHandler {
    pub line: usize,
    pub clause: String,
    pub action: Option<String>,
    pub is_broad: bool,
    pub suppresses: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ImportSpec {
    pub alias: String,
    pub path: String,
    pub namespace_path: Option<String>,
    pub imported_name: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct DeclaredSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub receiver_type: Option<String>,
    pub receiver_is_pointer: Option<bool>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct FieldSummary {
    pub line: usize,
    pub name: String,
    pub type_text: String,
    pub is_pub: bool,
    pub is_option: bool,
    pub is_primitive: bool,
    pub is_bool: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct StructSummary {
    pub line: usize,
    pub name: String,
    pub fields: Vec<FieldSummary>,
    pub derives: Vec<String>,
    pub has_debug_derive: bool,
    pub has_default_derive: bool,
    pub has_serialize_derive: bool,
    pub has_deserialize_derive: bool,
    pub visibility_pub: bool,
    pub impl_default: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct MacroCall {
    pub line: usize,
    pub name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeCall {
    pub line: usize,
    pub name: String,
    pub receiver: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct UnsafePattern {
    pub line: usize,
    pub kind: UnsafePatternKind,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnsafePatternKind {
    GetUnchecked,
    RawParts,
    SetLen,
    AssumeInit,
    Transmute,
    RawPointerCast,
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
