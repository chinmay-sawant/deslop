use super::common::FormattedErrorCall;

// ── Owned evidence storage ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct GoFunctionEvidence {
    pub has_context_parameter: bool,
    pub context_factory_calls: Vec<ContextFactoryCall>,
    pub dropped_errors: Vec<usize>,
    pub panic_errors: Vec<usize>,
    pub errorf_calls: Vec<FormattedErrorCall>,
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
}

impl GoFunctionEvidence {
    pub(crate) fn as_view(&self) -> GoFunctionEvidenceView<'_> {
        GoFunctionEvidenceView {
            has_context_parameter: self.has_context_parameter,
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
}

// ── Borrowed evidence view (read-only, zero-copy) ─────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct StructTag {
    pub line: usize,
    pub struct_name: String,
    pub field_name: String,
    pub raw_tag: String,
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
pub(crate) struct GormChainStep {
    pub line: usize,
    pub method_name: String,
    pub argument_texts: Vec<String>,
    pub first_string_arg: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct GormQueryChain {
    pub line: usize,
    pub root_text: String,
    pub terminal_method: String,
    pub steps: Vec<GormChainStep>,
    pub in_loop: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ParseInputCall {
    pub line: usize,
    pub parser_family: String,
    pub input_text: String,
    pub input_binding: Option<String>,
    pub target_text: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct GinCallSummary {
    pub line: usize,
    pub operation: String,
    pub argument_texts: Vec<String>,
    pub assigned_binding: Option<String>,
    pub in_loop: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PackageVarSummary {
    pub name: String,
    pub line: usize,
    pub type_text: Option<String>,
    pub value_text: Option<String>,
    pub is_pub: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct InterfaceSummary {
    pub name: String,
    pub line: usize,
    pub is_pub: bool,
    pub methods: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct GoFieldSummary {
    pub name: String,
    pub line: usize,
    pub type_text: String,
    pub is_pub: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct GoStructSummary {
    pub name: String,
    pub line: usize,
    pub is_pub: bool,
    pub fields: Vec<GoFieldSummary>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GoFunctionEvidenceView<'a> {
    pub has_context_parameter: bool,
    pub context_factory_calls: &'a [ContextFactoryCall],
    pub dropped_errors: &'a [usize],
    pub panic_errors: &'a [usize],
    pub errorf_calls: &'a [FormattedErrorCall],
    pub goroutines: &'a [usize],
    pub loop_goroutines: &'a [usize],
    pub unmanaged_goroutines: &'a [usize],
    pub sleep_loops: &'a [usize],
    pub busy_wait_lines: &'a [usize],
    pub mutex_loops: &'a [usize],
    pub alloc_loops: &'a [usize],
    pub fmt_loops: &'a [usize],
    pub reflect_loops: &'a [usize],
    pub concat_loops: &'a [usize],
    pub json_loops: &'a [usize],
    pub db_query_calls: &'a [DbQueryCall],
    pub gorm_query_chains: &'a [GormQueryChain],
    pub parse_input_calls: &'a [ParseInputCall],
    pub gin_calls: &'a [GinCallSummary],
}

impl<'a> GoFunctionEvidenceView<'a> {
    pub(crate) fn empty() -> Self {
        GoFunctionEvidenceView {
            has_context_parameter: false,
            context_factory_calls: &[],
            dropped_errors: &[],
            panic_errors: &[],
            errorf_calls: &[],
            goroutines: &[],
            loop_goroutines: &[],
            unmanaged_goroutines: &[],
            sleep_loops: &[],
            busy_wait_lines: &[],
            mutex_loops: &[],
            alloc_loops: &[],
            fmt_loops: &[],
            reflect_loops: &[],
            concat_loops: &[],
            json_loops: &[],
            db_query_calls: &[],
            gorm_query_chains: &[],
            parse_input_calls: &[],
            gin_calls: &[],
        }
    }
}
