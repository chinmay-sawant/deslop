mod docs;
mod hygiene;
mod import_resolution;
mod local_calls;

pub(crate) const BINDING_LOCATION: &str = file!();

#[cfg(test)]
pub(crate) fn alias_lookup(
    imports: &[crate::analysis::ImportSpec],
) -> std::collections::BTreeMap<String, crate::analysis::ImportSpec> {
    import_resolution::alias_lookup(imports)
}

#[cfg(test)]
pub(crate) fn call_matches_import(
    index: &crate::index::RepositoryIndex,
    file_path: &std::path::Path,
    import_spec: &crate::analysis::ImportSpec,
) -> bool {
    import_resolution::call_matches_import(index, file_path, import_spec)
}

#[cfg(test)]
pub(crate) fn import_matches_item(
    index: &crate::index::RepositoryIndex,
    file_path: &std::path::Path,
    import_spec: &crate::analysis::ImportSpec,
) -> bool {
    import_resolution::import_matches_item(index, file_path, import_spec)
}

pub(crate) fn doc_marker_findings(
    file: &crate::analysis::ParsedFile,
    function: &crate::analysis::ParsedFunction,
) -> Vec<crate::model::Finding> {
    docs::doc_marker_findings(file, function)
}

pub(crate) fn non_test_call_findings(
    file: &crate::analysis::ParsedFile,
    function: &crate::analysis::ParsedFunction,
    call_name: &str,
    rule_id: &str,
    message_suffix: &str,
) -> Vec<crate::model::Finding> {
    hygiene::non_test_call_findings(file, function, call_name, rule_id, message_suffix)
}

pub(crate) fn non_test_macro_findings(
    file: &crate::analysis::ParsedFile,
    function: &crate::analysis::ParsedFunction,
    macro_name: &str,
    rule_id: &str,
    message_suffix: &str,
) -> Vec<crate::model::Finding> {
    hygiene::non_test_macro_findings(file, function, macro_name, rule_id, message_suffix)
}

pub(crate) fn unsafe_findings(
    file: &crate::analysis::ParsedFile,
    function: &crate::analysis::ParsedFunction,
) -> Vec<crate::model::Finding> {
    hygiene::unsafe_findings(file, function)
}

pub(crate) fn rust_import_findings(
    file: &crate::analysis::ParsedFile,
    function: &crate::analysis::ParsedFunction,
    index: &crate::index::RepositoryIndex,
    imports: &[crate::analysis::ImportSpec],
    current_package: &crate::index::PackageIndex,
) -> Vec<crate::model::Finding> {
    import_resolution::rust_import_findings(file, function, index, imports, current_package)
}

pub(crate) fn rust_call_findings(
    file: &crate::analysis::ParsedFile,
    function: &crate::analysis::ParsedFunction,
    index: &crate::index::RepositoryIndex,
    imports: &[crate::analysis::ImportSpec],
) -> Vec<crate::model::Finding> {
    local_calls::rust_call_findings(file, function, index, imports)
}

fn is_rust_import(import_path: &str) -> bool {
    import_path.starts_with("crate::")
        || import_path.starts_with("self::")
        || import_path.starts_with("super::")
}
