pub(crate) mod comments;
mod common;
mod engine;
pub(crate) mod go;
pub(crate) mod hallucination;
pub(crate) mod naming;
pub(crate) mod python;
mod registry;
pub(crate) mod rust;
pub(crate) mod security;
pub(crate) mod test_quality;
#[cfg(test)]
mod tests;

pub(crate) use self::engine::{
    evaluate_go_file, evaluate_go_repo, evaluate_python_file, evaluate_python_repo,
    evaluate_shared_file, extend_file_rules, extend_function_rules,
};
