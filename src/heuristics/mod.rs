pub(crate) mod comments;
mod common;
pub(crate) mod engine;
pub(crate) mod go;
pub(crate) mod hallucination;
pub(crate) mod naming;
pub(crate) mod python;
pub(crate) mod registry;
pub(crate) mod rust;
pub(crate) mod security;
pub(crate) mod test_quality;
#[cfg(test)]
mod tests;

pub(crate) use self::engine::{evaluate_file, evaluate_repo};
