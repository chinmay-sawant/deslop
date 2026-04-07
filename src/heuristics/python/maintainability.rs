#[path = "maintainability/file_rules.rs"]
mod file_rules;
#[path = "maintainability/function_rules.rs"]
mod function_rules;
#[path = "maintainability/helpers.rs"]
mod helpers;

pub(crate) const BINDING_LOCATION: &str = file!();

pub(super) use file_rules::{commented_out_code_findings, sync_async_module_findings};
pub(super) use function_rules::{
    api_type_hint_findings, broad_exception_handler_findings, builtin_reduction_findings,
    env_fallback_findings, eval_exec_findings, exception_swallowed_findings,
    hardcoded_business_rule_findings, hardcoded_path_findings, input_validation_findings,
    magic_value_branching_findings, missing_context_manager_findings, network_timeout_findings,
    none_comparison_findings, print_debugging_findings, redundant_return_none_findings,
    reinvented_utility_findings, side_effect_comprehension_findings, variadic_public_api_findings,
};
