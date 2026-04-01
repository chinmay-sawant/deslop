use crate::model::SymbolKind;

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
pub(crate) struct NamedLiteral {
    pub line: usize,
    pub name: String,
    pub value: String,
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
    pub line: usize,
    pub group_line: usize,
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
pub(crate) struct TopLevelCallSummary {
    pub line: usize,
    pub receiver: Option<String>,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TopLevelBindingSummary {
    pub name: String,
    pub line: usize,
    pub value_text: String,
}
