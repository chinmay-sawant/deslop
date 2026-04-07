use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::model::Finding;

pub(super) type FunctionEvaluator = fn(&ParsedFile, &ParsedFunction) -> Vec<Finding>;
pub(super) type FileEvaluator = fn(&ParsedFile) -> Vec<Finding>;
pub(super) type RepoEvaluator = fn(&[&ParsedFile], &RepositoryIndex) -> Vec<Finding>;

pub(crate) struct PythonFunctionRuleSpec {
    pub(super) family: &'static str,
    pub(super) rule_ids: &'static [&'static str],
    pub(super) evaluate: FunctionEvaluator,
}

pub(crate) struct PythonFileRuleSpec {
    pub(super) family: &'static str,
    pub(super) rule_ids: &'static [&'static str],
    pub(super) evaluate: FileEvaluator,
}

pub(crate) struct PythonRepoRuleSpec {
    pub(super) family: &'static str,
    pub(super) rule_ids: &'static [&'static str],
    pub(super) evaluate: RepoEvaluator,
}
