// ── Owned evidence storage ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct RustFunctionEvidence {
    pub safety_comment_lines: Vec<usize>,
    pub unsafe_lines: Vec<usize>,
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

impl RustFunctionEvidence {
    pub(crate) fn as_view(&self) -> RustFunctionEvidenceView<'_> {
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

// ── Borrowed evidence view (read-only, zero-copy) ─────────────────────────────

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
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RustAttributeSummary {
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RustModuleDeclaration {
    pub line: usize,
    pub name: String,
    pub path_override: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RustIncludeDeclaration {
    pub line: usize,
    pub path: String,
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

impl<'a> RustFunctionEvidenceView<'a> {
    pub(crate) fn empty() -> Self {
        RustFunctionEvidenceView {
            safety_comment_lines: &[],
            unsafe_lines: &[],
            is_async: false,
            await_points: &[],
            macro_calls: &[],
            spawn_calls: &[],
            lock_calls: &[],
            permit_acquires: &[],
            futures_created: &[],
            blocking_calls: &[],
            select_macro_lines: &[],
            drop_impl: false,
            write_loops: &[],
            line_iteration_loops: &[],
            default_hasher_lines: &[],
            boxed_container_lines: &[],
            unsafe_soundness: &[],
        }
    }
}
