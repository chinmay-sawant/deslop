use super::common::BlockFingerprint;

// ── Owned evidence storage ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct PythonFunctionEvidence {
    pub exception_handlers: Vec<ExceptionHandler>,
    pub validation_signature: Option<BlockFingerprint>,
    pub exception_block_signatures: Vec<BlockFingerprint>,
    pub normalized_body: String,
    pub concat_loops: Vec<usize>,
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
}

impl PythonFunctionEvidence {
    pub(crate) fn as_view(&self) -> PythonFunctionEvidenceView<'_> {
        PythonFunctionEvidenceView {
            exception_handlers: &self.exception_handlers,
            validation_signature: self.validation_signature.as_ref(),
            exception_block_signatures: &self.exception_block_signatures,
            normalized_body: &self.normalized_body,
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
}

// ── Borrowed evidence view (read-only, zero-copy) ─────────────────────────────

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
pub(crate) struct ExceptionHandler {
    pub line: usize,
    pub clause: String,
    pub action: Option<String>,
    pub is_broad: bool,
    pub suppresses: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PythonFieldSummary {
    pub name: String,
    pub line: usize,
    pub annotation_text: Option<String>,
    pub default_text: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PythonModelSummary {
    pub name: String,
    pub line: usize,
    pub base_classes: Vec<String>,
    pub decorators: Vec<String>,
    pub is_dataclass: bool,
    pub is_typed_dict: bool,
    pub fields: Vec<PythonFieldSummary>,
    pub method_names: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PythonFunctionEvidenceView<'a> {
    pub exception_handlers: &'a [ExceptionHandler],
    pub validation_signature: Option<&'a BlockFingerprint>,
    pub exception_block_signatures: &'a [BlockFingerprint],
    pub normalized_body: &'a str,
    pub concat_loops: &'a [usize],
    pub none_comparison_lines: &'a [usize],
    pub side_effect_comprehension_lines: &'a [usize],
    pub redundant_return_none_lines: &'a [usize],
    pub list_materialization_lines: &'a [usize],
    pub deque_operation_lines: &'a [usize],
    pub temp_collection_lines: &'a [usize],
    pub recursive_call_lines: &'a [usize],
    pub list_membership_loop_lines: &'a [usize],
    pub repeated_len_loop_lines: &'a [usize],
    pub builtin_candidate_lines: &'a [usize],
    pub missing_context_manager_lines: &'a [usize],
    pub has_complete_type_hints: bool,
    pub has_varargs: bool,
    pub has_kwargs: bool,
    pub is_async: bool,
    pub await_points: &'a [usize],
}

impl<'a> PythonFunctionEvidenceView<'a> {
    pub(crate) fn empty() -> Self {
        PythonFunctionEvidenceView {
            exception_handlers: &[],
            validation_signature: None,
            exception_block_signatures: &[],
            normalized_body: "",
            concat_loops: &[],
            none_comparison_lines: &[],
            side_effect_comprehension_lines: &[],
            redundant_return_none_lines: &[],
            list_materialization_lines: &[],
            deque_operation_lines: &[],
            temp_collection_lines: &[],
            recursive_call_lines: &[],
            list_membership_loop_lines: &[],
            repeated_len_loop_lines: &[],
            builtin_candidate_lines: &[],
            missing_context_manager_lines: &[],
            has_complete_type_hints: false,
            has_varargs: false,
            has_kwargs: false,
            is_async: false,
            await_points: &[],
        }
    }
}
