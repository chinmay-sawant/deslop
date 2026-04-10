use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

macro_rules! perf_rule {
    ($id:expr, $desc:expr) => {
        RuleDefinition {
            id: $id,
            language: RuleLanguage::Python,
            family: "performance",
            default_severity: RuleDefaultSeverity::Contextual,
            status: RuleStatus::Stable,
            configurability: &[
                RuleConfigurability::Disable,
                RuleConfigurability::Ignore,
                RuleConfigurability::SeverityOverride,
            ],
            description: $desc,
            binding_location: super::bindings::PYTHON_PERFORMANCE,
        }
    };
}

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "blocking_sync_io_in_async",
        language: RuleLanguage::Python,
        family: "performance",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Synchronous network, subprocess, sleep, or file I/O calls made from async def functions.",
        binding_location: super::bindings::PYTHON_PERFORMANCE,
    },
    RuleDefinition {
        id: "deque_candidate_queue",
        language: RuleLanguage::Python,
        family: "performance",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Queue-style list operations like pop(0) or insert(0, ...) that may want collections.deque.",
        binding_location: super::bindings::PYTHON_PERFORMANCE,
    },
    RuleDefinition {
        id: "full_dataset_load",
        language: RuleLanguage::Python,
        family: "performance",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Calls that load an entire payload into memory instead of streaming.",
        binding_location: super::bindings::PYTHON_PERFORMANCE,
    },
    RuleDefinition {
        id: "list_materialization_first_element",
        language: RuleLanguage::Python,
        family: "performance",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "list(...)[0] style access that materializes a whole list just to read the first element.",
        binding_location: super::bindings::PYTHON_PERFORMANCE,
    },
    RuleDefinition {
        id: "list_membership_in_loop",
        language: RuleLanguage::Python,
        family: "performance",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Repeated membership checks against obviously list-like containers inside loops.",
        binding_location: super::bindings::PYTHON_PERFORMANCE,
    },
    RuleDefinition {
        id: "recursive_traversal_risk",
        language: RuleLanguage::Python,
        family: "performance",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Direct recursion in traversal-style helpers that may be safer as iterative walks for deep inputs.",
        binding_location: super::bindings::PYTHON_PERFORMANCE,
    },
    RuleDefinition {
        id: "repeated_len_in_loop",
        language: RuleLanguage::Python,
        family: "performance",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Repeated len(...) checks inside loops when the receiver appears unchanged locally.",
        binding_location: super::bindings::PYTHON_PERFORMANCE,
    },
    RuleDefinition {
        id: "string_concat_in_loop",
        language: RuleLanguage::Python,
        family: "performance",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Repeated string concatenation inside loops can create O(n^2) growth and extra allocations.",
        binding_location: super::bindings::PYTHON_PERFORMANCE,
    },
    RuleDefinition {
        id: "temporary_collection_in_loop",
        language: RuleLanguage::Python,
        family: "performance",
        default_severity: RuleDefaultSeverity::Contextual,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "Loop-local list, dict, or set construction that likely adds avoidable allocation churn.",
        binding_location: super::bindings::PYTHON_PERFORMANCE,
    },
    perf_rule!(
        "repeated_file_open_for_same_resource_within_single_operation",
        "Flag workflows that reopen the same file repeatedly during one logical operation."
    ),
    perf_rule!(
        "eager_full_file_or_stream_read_when_incremental_iteration_suffices",
        "Flag code that reads whole files or streams into memory before simple sequential processing."
    ),
    perf_rule!(
        "bytes_text_bytes_roundtrip_without_transformation",
        "Flag code that decodes and re-encodes data without changing it."
    ),
    perf_rule!(
        "quadratic_string_building_via_plus_equals",
        "Flag loops that grow large strings with repeated +=."
    ),
    perf_rule!(
        "multiple_regex_passes_over_same_text_without_precompiled_plan",
        "Flag code that re-runs several overlapping regex passes on the same text."
    ),
    perf_rule!(
        "full_response_or_export_buffered_before_incremental_consumer_use",
        "Flag producers that fully buffer large outputs before handing them to a consumer."
    ),
    perf_rule!(
        "temporary_file_used_for_pure_in_memory_transformation",
        "Flag workflows that spill to disk despite an in-memory transform being sufficient."
    ),
    perf_rule!(
        "thread_pool_or_process_pool_created_and_destroyed_per_call",
        "Flag per-call executor allocation for repeatable work."
    ),
    perf_rule!(
        "large_object_cloned_before_read_only_operation",
        "Flag code that deep-copies large structures even when the next steps are read-only."
    ),
    perf_rule!(
        "repeated_stat_or_exists_calls_before_single_followup_operation",
        "Flag paths that perform duplicate filesystem checks before one operation."
    ),
    perf_rule!(
        "batchable_writes_executed_one_at_a_time",
        "Flag repeated write operations that could be grouped or buffered."
    ),
    perf_rule!(
        "same_dataset_normalized_in_multiple_full_passes",
        "Flag code that walks the same dataset several times for normalization steps that can be fused."
    ),
    perf_rule!(
        "generator_materialized_to_tuple_or_list_only_for_len_or_truthiness",
        "Flag iterator pipelines that materialize only to test truthiness or size."
    ),
    perf_rule!(
        "full_collection_sorted_when_partial_order_or_selection_suffices",
        "Flag full sorts used where top-k or one-pass selection would work."
    ),
    perf_rule!(
        "compression_hashing_or_encoding_performed_before_cheap_reject_checks",
        "Flag expensive transforms performed before simple guard checks that could skip the work."
    ),
    perf_rule!(
        "event_loop_path_executes_cpu_bound_transformation_synchronously",
        "Flag async paths that perform large CPU-bound transforms inline."
    ),
    perf_rule!(
        "repeated_small_writes_without_buffering_or_join",
        "Flag code that emits many tiny writes instead of buffering."
    ),
    perf_rule!(
        "copy_of_mapping_created_only_to_read_values",
        "Flag mappings copied defensively even though the next code only reads."
    ),
    perf_rule!(
        "serialization_cost_paid_only_to_compare_or_hash_intermediate_state",
        "Flag serialization used only for equality, cache key, or hashing comparisons."
    ),
    perf_rule!(
        "large_in_memory_intermediate_created_where_streaming_pipeline_would_do",
        "Flag workflows that build large temporary structures where streaming would suffice."
    ),
];
