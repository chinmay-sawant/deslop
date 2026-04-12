use super::common::BlockFingerprint;

// ── Owned evidence storage ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
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
    pub sorted_first_lines: Vec<usize>,
    pub len_comprehension_lines: Vec<usize>,
    pub in_list_literal_lines: Vec<usize>,
    pub startswith_chain_lines: Vec<usize>,
    pub enumerate_range_len_lines: Vec<usize>,
    pub dict_materialization_in_loop_lines: Vec<usize>,
    pub readlines_then_iterate_lines: Vec<usize>,
    pub read_splitlines_lines: Vec<usize>,
    pub write_in_loop_lines: Vec<usize>,
    pub csv_flush_per_row_lines: Vec<usize>,
    pub regex_in_hotpath_lines: Vec<usize>,
    pub repeated_call_same_arg: Vec<(String, usize)>,
    pub repeated_open_same_file: Vec<(String, usize)>,
    pub copy_in_loop_lines: Vec<usize>,
    pub invariant_call_in_loop_lines: Vec<(String, usize)>,
    pub index_in_loop_lines: Vec<usize>,
    pub append_sort_in_loop_lines: Vec<usize>,
    pub join_list_comp_lines: Vec<usize>,
    pub repeated_subscript_lines: Vec<usize>,
}

impl PythonFunctionEvidence {
    pub(crate) fn as_view(&self) -> PythonFunctionEvidenceView<'_> {
        let view = PythonFunctionEvidenceView {
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
            sorted_first_lines: &self.sorted_first_lines,
            len_comprehension_lines: &self.len_comprehension_lines,
            in_list_literal_lines: &self.in_list_literal_lines,
            startswith_chain_lines: &self.startswith_chain_lines,
            enumerate_range_len_lines: &self.enumerate_range_len_lines,
            dict_materialization_in_loop_lines: &self.dict_materialization_in_loop_lines,
            readlines_then_iterate_lines: &self.readlines_then_iterate_lines,
            read_splitlines_lines: &self.read_splitlines_lines,
            write_in_loop_lines: &self.write_in_loop_lines,
            csv_flush_per_row_lines: &self.csv_flush_per_row_lines,
            regex_in_hotpath_lines: &self.regex_in_hotpath_lines,
            repeated_call_same_arg: &self.repeated_call_same_arg,
            repeated_open_same_file: &self.repeated_open_same_file,
            copy_in_loop_lines: &self.copy_in_loop_lines,
            invariant_call_in_loop_lines: &self.invariant_call_in_loop_lines,
            index_in_loop_lines: &self.index_in_loop_lines,
            append_sort_in_loop_lines: &self.append_sort_in_loop_lines,
            join_list_comp_lines: &self.join_list_comp_lines,
            repeated_subscript_lines: &self.repeated_subscript_lines,
        };
        let _ = view.await_points;
        view
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
    pub sorted_first_lines: &'a [usize],
    pub len_comprehension_lines: &'a [usize],
    pub in_list_literal_lines: &'a [usize],
    pub startswith_chain_lines: &'a [usize],
    pub enumerate_range_len_lines: &'a [usize],
    pub dict_materialization_in_loop_lines: &'a [usize],
    pub readlines_then_iterate_lines: &'a [usize],
    pub read_splitlines_lines: &'a [usize],
    pub write_in_loop_lines: &'a [usize],
    pub csv_flush_per_row_lines: &'a [usize],
    pub regex_in_hotpath_lines: &'a [usize],
    pub repeated_call_same_arg: &'a [(String, usize)],
    pub repeated_open_same_file: &'a [(String, usize)],
    pub copy_in_loop_lines: &'a [usize],
    pub invariant_call_in_loop_lines: &'a [(String, usize)],
    pub index_in_loop_lines: &'a [usize],
    pub append_sort_in_loop_lines: &'a [usize],
    pub join_list_comp_lines: &'a [usize],
    pub repeated_subscript_lines: &'a [usize],
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
            sorted_first_lines: &[],
            len_comprehension_lines: &[],
            in_list_literal_lines: &[],
            startswith_chain_lines: &[],
            enumerate_range_len_lines: &[],
            dict_materialization_in_loop_lines: &[],
            readlines_then_iterate_lines: &[],
            read_splitlines_lines: &[],
            write_in_loop_lines: &[],
            csv_flush_per_row_lines: &[],
            regex_in_hotpath_lines: &[],
            repeated_call_same_arg: &[],
            repeated_open_same_file: &[],
            copy_in_loop_lines: &[],
            invariant_call_in_loop_lines: &[],
            index_in_loop_lines: &[],
            append_sort_in_loop_lines: &[],
            join_list_comp_lines: &[],
            repeated_subscript_lines: &[],
        }
    }
}
