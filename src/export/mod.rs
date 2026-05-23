mod block;
mod chunk;
mod function;
mod triage;
mod writer;

use crate::Result;
use crate::model::{ScanOutput, ScanReport};

pub(crate) use writer::export_finding_context;
pub use writer::{ExportOptions, ExportSummary};

impl ScanOutput {
    pub fn export_context(
        &self,
        report: &ScanReport,
        options: &ExportOptions,
    ) -> Result<ExportSummary> {
        export_finding_context(report, &self.parsed_files, options)
    }
}
