use super::{RuleConfigurability, RuleDefaultSeverity, RuleDefinition, RuleLanguage, RuleStatus};

pub(crate) const RULE_DEFINITIONS: &[RuleDefinition] = &[
    RuleDefinition {
        id: "append_then_sort_each_iteration",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "A collection is appended to and then sorted on each iteration instead of sorting once after accumulation.",
        binding_location: "src/heuristics/python/hotpath_ext.rs",
    },
    RuleDefinition {
        id: "csv_writer_flush_per_row",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "csv.Writer flushes on each row instead of buffering a larger batch.",
        binding_location: "src/heuristics/python/hotpath.rs",
    },
    RuleDefinition {
        id: "filter_then_count_then_iterate",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "The same collection is traversed repeatedly for filtering, counting, and later iteration.",
        binding_location: "src/heuristics/python/hotpath_ext.rs",
    },
    RuleDefinition {
        id: "json_encoder_recreated_per_item",
        language: RuleLanguage::Python,
        family: "hot_path",
        default_severity: RuleDefaultSeverity::Info,
        status: RuleStatus::Stable,
        configurability: &[
            RuleConfigurability::Disable,
            RuleConfigurability::Ignore,
            RuleConfigurability::SeverityOverride,
        ],
        description: "A JSON encoder object is recreated per item instead of being reused for the stream.",
        binding_location: "src/heuristics/python/hotpath_ext.rs",
    },
];
