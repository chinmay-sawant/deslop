mod common;
mod core;
mod go;
mod python;
mod rust;

use crate::model::FileReport;

pub(crate) use common::{
    BlockFingerprint, CallSite, CommentSummary, DeclaredSymbol, FormattedErrorCall, ImportSpec,
    NamedLiteral, TestFunctionSummary, TopLevelBindingSummary, TopLevelCallSummary,
};
pub(crate) use core::{ParsedFile, ParsedFunction};
pub(crate) use go::{
    ContextFactoryCall, DbQueryCall, GinCallSummary, GoFieldSummary, GoStructSummary,
    GormChainStep, GormQueryChain, InterfaceSummary, PackageVarSummary, ParseInputCall, StructTag,
};
pub(crate) use python::{ClassSummary, ExceptionHandler, PythonFieldSummary, PythonModelSummary};
pub(crate) use rust::{
    FieldSummary, MacroCall, RuntimeCall, RustEnumSummary, RustStaticSummary, StructSummary,
    UnsafePattern, UnsafePatternKind,
};

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
