#[derive(Debug, Clone)]
pub(crate) struct FieldSummary {
    pub line: usize,
    pub name: String,
    pub type_text: String,
    pub attributes: Vec<String>,
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
    pub attributes: Vec<String>,
    pub has_debug_derive: bool,
    pub has_default_derive: bool,
    pub has_serialize_derive: bool,
    pub has_deserialize_derive: bool,
    pub visibility_pub: bool,
    pub impl_default: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RustStaticSummary {
    pub line: usize,
    pub name: String,
    pub type_text: String,
    pub value_text: Option<String>,
    pub visibility_pub: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RustEnumSummary {
    pub line: usize,
    pub name: String,
    pub variant_count: usize,
    pub derives: Vec<String>,
    pub attributes: Vec<String>,
    pub has_serialize_derive: bool,
    pub has_deserialize_derive: bool,
    pub visibility_pub: bool,
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct RustFunctionEvidenceView<'a> {
    pub safety_comment_lines: &'a [usize],
    pub unsafe_lines: &'a [usize],
    pub is_async: bool,
    pub await_points: &'a [usize],
    pub macro_calls: &'a [MacroCall],
    pub spawn_calls: &'a [RuntimeCall],
    pub lock_calls: &'a [RuntimeCall],
    pub permit_acquires: &'a [RuntimeCall],
    pub futures_created: &'a [RuntimeCall],
    pub blocking_calls: &'a [RuntimeCall],
    pub select_macro_lines: &'a [usize],
    pub drop_impl: bool,
    pub write_loops: &'a [usize],
    pub line_iteration_loops: &'a [usize],
    pub default_hasher_lines: &'a [usize],
    pub boxed_container_lines: &'a [usize],
    pub unsafe_soundness: &'a [UnsafePattern],
}
