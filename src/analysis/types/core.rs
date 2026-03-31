use std::path::PathBuf;

use crate::analysis::Language;
use crate::model::FunctionFingerprint;

use super::common::{
    BlockFingerprint, CallSite, CommentSummary, DeclaredSymbol, FormattedErrorCall, ImportSpec,
    NamedLiteral, TestFunctionSummary, TopLevelBindingSummary, TopLevelCallSummary,
};
use super::go::{
    ContextFactoryCall, DbQueryCall, GinCallSummary, GoFunctionEvidenceView, GoStructSummary,
    GormQueryChain, InterfaceSummary, PackageVarSummary, ParseInputCall, StructTag,
};
use super::python::{
    ClassSummary, ExceptionHandler, PythonFunctionEvidenceView, PythonModelSummary,
};
use super::rust::{
    MacroCall, RuntimeCall, RustEnumSummary, RustFunctionEvidenceView, RustStaticSummary,
    StructSummary, UnsafePattern,
};

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
    pub package_vars: Vec<PackageVarSummary>,
    pub interfaces: Vec<InterfaceSummary>,
    pub go_structs: Vec<GoStructSummary>,
    pub module_scope_calls: Vec<TopLevelCallSummary>,
    pub top_level_bindings: Vec<TopLevelBindingSummary>,
    pub python_models: Vec<PythonModelSummary>,
    pub rust_statics: Vec<RustStaticSummary>,
    pub rust_enums: Vec<RustEnumSummary>,
    // Rust heuristics consume conservative struct summaries so they can stay syntax-driven.
    pub structs: Vec<StructSummary>,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedFunction {
    pub fingerprint: FunctionFingerprint,
    pub signature_text: String,
    pub body_start_line: usize,
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
    pub gorm_query_chains: Vec<GormQueryChain>,
    pub parse_input_calls: Vec<ParseInputCall>,
    pub gin_calls: Vec<GinCallSummary>,
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
    // Unsafe-pattern summaries keep the rule pack local to parser evidence rather than ad-hoc text scans.
    pub unsafe_soundness: Vec<UnsafePattern>,
}

impl ParsedFunction {
    pub(crate) fn go_evidence(&self) -> GoFunctionEvidenceView<'_> {
        GoFunctionEvidenceView {
            context_factory_calls: &self.context_factory_calls,
            dropped_errors: &self.dropped_errors,
            panic_errors: &self.panic_errors,
            errorf_calls: &self.errorf_calls,
            goroutines: &self.goroutines,
            loop_goroutines: &self.loop_goroutines,
            unmanaged_goroutines: &self.unmanaged_goroutines,
            sleep_loops: &self.sleep_loops,
            busy_wait_lines: &self.busy_wait_lines,
            mutex_loops: &self.mutex_loops,
            alloc_loops: &self.alloc_loops,
            fmt_loops: &self.fmt_loops,
            reflect_loops: &self.reflect_loops,
            concat_loops: &self.concat_loops,
            json_loops: &self.json_loops,
            db_query_calls: &self.db_query_calls,
            gorm_query_chains: &self.gorm_query_chains,
            parse_input_calls: &self.parse_input_calls,
            gin_calls: &self.gin_calls,
        }
    }

    pub(crate) fn python_evidence(&self) -> PythonFunctionEvidenceView<'_> {
        PythonFunctionEvidenceView {
            exception_handlers: &self.exception_handlers,
            validation_signature: self.validation_signature.as_ref(),
            exception_block_signatures: &self.exception_block_signatures,
            concat_loops: &self.concat_loops,
            none_comparison_lines: &self.none_comparison_lines,
            side_effect_comprehension_lines: &self.side_effect_comprehension_lines,
            redundant_return_none_lines: &self.redundant_return_none_lines,
            list_materialization_lines: &self.list_materialization_lines,
            deque_operation_lines: &self.deque_operation_lines,
            temp_collection_lines: &self.temp_collection_lines,
            recursive_call_lines: &self.recursive_call_lines,
            list_membership_loop_lines: &self.list_membership_loop_lines,
            repeated_len_loop_lines: &self.repeated_len_loop_lines,
            builtin_candidate_lines: &self.builtin_candidate_lines,
            missing_context_manager_lines: &self.missing_context_manager_lines,
            has_complete_type_hints: self.has_complete_type_hints,
            has_varargs: self.has_varargs,
            has_kwargs: self.has_kwargs,
            is_async: self.is_async,
            await_points: &self.await_points,
        }
    }

    pub(crate) fn rust_evidence(&self) -> RustFunctionEvidenceView<'_> {
        RustFunctionEvidenceView {
            safety_comment_lines: &self.safety_comment_lines,
            unsafe_lines: &self.unsafe_lines,
            is_async: self.is_async,
            await_points: &self.await_points,
            macro_calls: &self.macro_calls,
            spawn_calls: &self.spawn_calls,
            lock_calls: &self.lock_calls,
            permit_acquires: &self.permit_acquires,
            futures_created: &self.futures_created,
            blocking_calls: &self.blocking_calls,
            select_macro_lines: &self.select_macro_lines,
            drop_impl: self.drop_impl,
            write_loops: &self.write_loops,
            line_iteration_loops: &self.line_iteration_loops,
            default_hasher_lines: &self.default_hasher_lines,
            boxed_container_lines: &self.boxed_container_lines,
            unsafe_soundness: &self.unsafe_soundness,
        }
    }
}
