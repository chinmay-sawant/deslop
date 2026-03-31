use super::common::BlockFingerprint;

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
