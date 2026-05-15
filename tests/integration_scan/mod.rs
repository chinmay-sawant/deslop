// `support` lives one directory above; #[path] is required here because
// the module file is outside the integration_scan/ subtree.
#[path = "../support/mod.rs"]
mod support;

mod benchmarking;
mod go;
mod python;
mod rust;
