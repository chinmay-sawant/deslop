use super::common::FormattedErrorCall;

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
    #[allow(dead_code)]
    pub in_loop: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct GinCallSummary {
    pub line: usize,
    pub operation: String,
    #[allow(dead_code)]
    pub argument_texts: Vec<String>,
    #[allow(dead_code)]
    pub assigned_binding: Option<String>,
    #[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct GoFunctionEvidenceView<'a> {
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
