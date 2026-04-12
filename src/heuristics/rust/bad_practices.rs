#![allow(clippy::too_many_lines, clippy::uninlined_format_args)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use toml::Value;

use crate::analysis::{ParsedFile, ParsedFunction};
use crate::index::RepositoryIndex;
use crate::io::{DEFAULT_MAX_BYTES, read_to_string_limited};
use crate::model::{Finding, Severity};

pub(crate) const BINDING_LOCATION: &str = file!();

pub(crate) const RULE_DEFINITIONS: &[crate::rules::catalog::RuleDefinition] = &[
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_parser_created_per_file_without_reuse",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `tree_sitter::Parser::new()` construction inside per-file or per-function loops where a language-specific parser could be reused per worker.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_set_language_repeated_inside_hot_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated `parser.set_language(...)` calls inside loops that parse the same language repeatedly.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_language_conversion_inside_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated `tree_sitter_rust::LANGUAGE.into()` or equivalent grammar conversion in inner loops instead of caching the converted language.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_query_compiled_per_call",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `tree_sitter::Query::new(...)` inside functions called per file or per node, where query compilation should be cached.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_parse_result_unwrapped",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `parser.parse(...).unwrap ()` or `expect (...)` at parser boundaries where parse cancellation or invalid parser state should become a typed finding/error.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_error_tree_ignored",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag parser pipelines that use the root node without checking `tree.root_node().has_error()` or equivalent syntax-error evidence.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_recursive_walk_without_depth_guard",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag recursive AST walkers over repository input without an explicit depth guard, iterative cursor, or stack budget.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_node_text_redecoded_in_nested_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated `node.utf8_text(source)` or source slicing for the same node inside nested traversal loops.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_child_lookup_by_field_name_in_hot_walk",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated `child_by_field_name(\"...\")` string lookups in hot recursive walkers where field ids, cursors, or one-pass extraction would avoid repeated lookup work.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_descendant_for_point_range_in_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated descendant/range queries inside loops over sibling nodes, which can turn a tree walk into avoidable quadratic traversal.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_collects_all_captures_before_filtering",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag query capture pipelines that collect every capture into `Vec` and then filter locally instead of filtering as captures are visited.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_byte_offset_used_as_char_index",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag byte offsets from `start_byte` or `end_byte` used as character counts, `chars().nth(...)`, or display-column positions without conversion.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_old_tree_discarded_in_reparse_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated parse loops over the same buffer that always pass `None` instead of reusing an old tree for incremental parsing.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tree_sitter_parser_shared_global_with_lock",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag global `Mutex<Parser>` or `RwLock<Parser>` shared across threads where per-thread parser ownership would avoid lock contention and parser state coupling.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_nested_parallel_iterators",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag nested `par_iter`, `into_par_iter`, or `par_bridge` calls that can oversubscribe worker threads or fragment work.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_mutex_push_in_parallel_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `Mutex<Vec<_>>`, `RwLock<Vec<_>>`, or shared collection mutation inside `for_each` where `map/filter_map/collect/reduce` would avoid lock contention.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_collect_all_then_filter_sequentially",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `par_iter().map(...).collect::<Vec<_>>()` immediately followed by sequential filtering or flattening that could stay in the parallel pipeline.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_sequential_collect_then_par_iter",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag collecting an intermediate `Vec` only to immediately call `par_iter` or `into_par_iter` when the producer could feed the parallel stage directly.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_par_bridge_on_small_or_indexed_iterator",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `par_bridge()` over known indexed collections or tiny iterators where normal `par_iter` or sequential iteration is likely cheaper.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_blocking_io_in_cpu_parallel_pool",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag filesystem, network, or process I/O inside Rayon CPU-bound closures unless the project documents that Rayon owns the I/O concurrency budget.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_heavy_clone_per_item",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated `.clone()` of config, source text, rule catalogs, AST summaries, or large state inside parallel closures.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_large_move_capture",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `move` closures in parallel iterators that capture large owned values rather than borrowing or sharing cheap handles.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_ordering_mutex_for_result_stability",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag parallel closures that serialize through `Mutex<BTreeMap<_>>`, `Mutex<Vec<_>>`, or sequence counters just to regain output order.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_global_pool_built_by_library_code",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `ThreadPoolBuilder::build_global()` outside a binary entry point, test harness, or explicit process bootstrap.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_custom_pool_created_per_call",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `ThreadPoolBuilder::build()` inside frequently-called functions instead of process-level or subsystem-level pool ownership.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_unhandled_panic_in_parallel_scan",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `unwrap`, `expect`, or panicking conversions inside parallel scan/evaluation closures where one bad file can abort the whole batch.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_flat_map_allocates_nested_vectors",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag parallel map stages that return `Vec<Vec<T>>` and later flatten, where `flat_map_iter`, `reduce`, or direct collection can reduce allocation churn.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rayon_parallelism_for_trivial_per_item_work",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag parallel iteration over simple string predicates or metadata checks when there is no expensive per-item work and the input is below a conservative threshold.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_walker_disables_standard_filters_without_policy",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `.ignore(false)`, `.git_ignore(false)`, `.git_exclude(false)`, `.hidden(false)`, or `.parents(false)` unless paired with a named user option or security review note.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_follow_links_without_cycle_or_root_policy",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `.follow_links(true)` without visible cycle, root containment, or same-filesystem policy.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_walk_error_silently_discarded",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag walk result handling that uses `filter_map(Result::ok)` or `ok()?` without recording ignored traversal errors.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_direntry_unwrapped",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `entry.unwrap ()` or `expect (...)` on walker entries in production scan code.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_reads_file_before_file_type_check",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag read/open operations on walker paths before checking `file_type().is_file()` or equivalent.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_walker_rebuilt_inside_directory_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `WalkBuilder::new(...)` construction inside recursive or per-directory loops.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_override_pattern_unwraps_user_input",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `OverrideBuilder::add(...)` or glob override construction from user input followed by `unwrap` or `expect`.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_parallel_walker_unbounded_accumulation",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag parallel walker callbacks that push every path into an unbounded `Vec` or channel before processing.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_filter_entry_allocates_path_string_per_node",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `filter_entry` closures that allocate `String` or call `to_string_lossy()` for every node when path component checks would work.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ignore_sort_by_path_after_full_walk",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag full repository path collection followed by sort before processing where deterministic output could be produced after finding collection instead.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_json_value_internal_hot_path",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `serde_json::Value` used as an internal data model in non-boundary modules where typed structs would reduce dynamic lookups and runtime errors.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_json_indexing_without_type_guard",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `value[\"key\"]`, `value[index]`, or chained indexing without typed deserialization, `.get(...)`, or null/type checks.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_json_to_string_pretty_in_machine_path",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `serde_json::to_string_pretty` in non-human-output hot paths, APIs, or report generation loops.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_json_clone_value_in_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `serde_json::Value::clone()` or cloned JSON maps inside loops.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_json_from_str_after_unbounded_read",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `serde_json::from_str` or `from_slice` fed by unbounded file/network reads rather than size-limited input.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_json_roundtrip_conversion",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `to_value` followed by `from_value`, or `to_string` followed by `from_str`, when a direct conversion or typed boundary is available.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_json_whole_array_loaded_for_streaming_input",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag deserializing large JSON arrays into `Vec<T>` in reader-like code where streaming deserialization could reduce peak memory.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_custom_deserialize_panics",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag custom `Deserialize` impls or visitors that call `unwrap`, `expect`, `panic`, or unchecked indexing.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_borrow_missing_for_large_readonly_payload",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag large readonly payload structs that deserialize owned `String` or `Vec<u8>` in hot paths where `Cow<'de, str>` or borrowed fields could avoid copies.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_skip_serializing_secret_without_deserialize_guard",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag secret-like fields skipped during serialization but still accepted during deserialization without validation/redaction policy.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_wire_enum_missing_stable_rename_policy",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag public wire-facing enums that derive `Serialize` or `Deserialize` without `rename_all` or explicit renames.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_toml_value_config_boundary",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag application config parsed into `toml::Value` and queried dynamically instead of deserializing into a typed config struct.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_toml_parse_in_hot_path",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `toml::from_str`, `toml::from_slice`, or `str::parse::<toml::Value>()` inside request paths, scan loops, or repeated functions.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_toml_config_without_unknown_field_rejection",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag TOML-facing config structs that derive `Deserialize` without `#[serde(deny_unknown_fields)]` when they appear to represent project configuration.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_toml_manifest_parse_without_size_limit",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag manifest/config parsing helpers that read and parse TOML without a byte limit.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_serde_default_masks_parse_error",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag broad `unwrap_or_default`, `Default::default`, or `#[serde(default)]` use around configuration fields that look required for correctness.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_anyhow_context_missing_on_boundary_io",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `?` on filesystem, environment, process, parser, config, or network calls in CLI/boundary code without `.context(...)` or `.with_context(...)`.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_anyhow_eager_format_context",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `.context(format!(...))` where `.with_context(|| format!(...))` would avoid allocation on the success path.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_anyhow_error_string_matching",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag branching on `err.to_string()`, `format!(\"{err}\")`, or message substrings instead of typed errors or downcasts.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_anyhow_downcast_without_fallback_context",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `downcast_ref`/`downcast` handling that drops the original context or returns a generic fallback.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_anyhow_bail_in_low_level_library_module",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `anyhow::bail!` in domain, parser, storage, or library modules that should usually expose typed error variants.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_thiserror_variant_wraps_source_without_source_attr",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag error variants with source-like fields that lack `#[source]`, `#[from]`, or transparent wrapping.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_thiserror_display_leaks_secret_field",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `#[error(...)]` format strings that interpolate token, password, secret, key, auth, cookie, or credential fields.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_thiserror_stringly_typed_variant",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag error enum variants whose only payload is `String` or `&'static str` and whose name does not encode a specific error kind.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_thiserror_transparent_on_contextual_variant",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `#[error(transparent)]` variants that also carry context-like fields or lose higher-level operation details.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_error_logged_and_returned",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag functions that log an error and then return the same error upward, causing duplicate logging at callers.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_result_ignored_with_let_underscore",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `let _ = fallible_call()` outside cleanup, telemetry, or best-effort contexts.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_question_mark_after_partial_side_effect_without_cleanup",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `?` after partially mutating files, shared state, transactions, or output buffers without rollback or cleanup.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_closed_set_manual_string_match",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag CLI string arguments that are manually matched against a fixed set instead of using `ValueEnum` or `value_parser`.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_path_arg_used_without_validation",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `PathBuf` or path-like CLI values used for reads/writes without root containment, canonicalization strategy, or symlink policy.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_default_value_manual_parse",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `default_value` strings that are parsed manually later instead of typed `value_parser` or typed fields.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_unbounded_vec_arg_on_scan_path",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `Vec<String>` or variadic CLI arguments that feed scan/filter work without a limit, deduplication, or validation.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_secret_arg_derive_debug",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag CLI structs deriving `Debug` while containing token, password, secret, key, cookie, or auth fields.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_opposing_flags_without_conflicts",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag pairs such as `--json/--text`, `--include/--exclude`, or `--enable/--disable` without `conflicts_with`, `overrides_with`, or explicit precedence.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_rule_id_arg_without_catalog_validation",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag rule-id-like CLI arguments accepted as strings without checking against the known rule registry.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_subcommand_reloads_config_in_each_branch",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated config loading inside every subcommand branch instead of a shared pre-dispatch normalization step.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_parse_called_below_main",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `Cli::parse()` or `try_parse()` called from library code or tests without dependency injection, making the code hard to reuse and test.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clap_env_var_without_redaction_policy",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `#[arg(env = \"...\")]` on secret-like fields without redacted display/report behavior.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libc_call_without_platform_cfg",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag direct `libc::*` calls without `#[cfg(unix)]`, `#[cfg(target_os = \"...\")]`, or a platform abstraction.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libc_return_value_not_checked",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag libc calls whose integer or pointer return value is ignored or not checked for `-1`, null, or documented failure sentinels.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libc_errno_read_after_intervening_call",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag errno access after another call that could overwrite the original failure cause.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libc_raw_fd_lifetime_escape",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `as_raw_fd()` results stored, returned, or moved into long-lived structs while the owning file/socket may drop.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libc_into_raw_fd_without_reclaim",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `into_raw_fd()` without a visible `from_raw_fd`, `OwnedFd`, or close handoff.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libc_cstring_unwrap_on_external_input",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `CString::new(user_input).unwrap ()` or `expect (...)` where embedded NUL bytes should become a recoverable error.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libc_open_without_cloexec",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag direct `libc::open` or equivalent without `O_CLOEXEC` in programs that may spawn child processes.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libc_no_follow_without_eloop_handling",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `O_NOFOLLOW` usage without explicit symlink-loop or unsupported-platform error handling.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ffi_extern_block_without_abi_comment",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag FFI declarations without a nearby note about ABI, ownership, nullability, or lifetime expectations.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_ffi_slice_from_raw_parts_without_length_guard",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag safe wrappers around raw pointer and length pairs that do not validate null pointers, maximum length, or alignment before slice construction.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tempfile_named_path_used_after_drop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `NamedTempFile::path()` stored or returned while the temporary file owner may drop before the path is used.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tempfile_persist_without_cleanup_assertion",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `persist`, `keep`, or `into_temp_path().keep()` in tests without cleanup or an assertion that the file must survive.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tempfile_predictable_name_in_shared_tmp",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag tests that combine `std::env::temp_dir()` with predictable filenames instead of `tempfile`.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_tempfile_builder_prefix_from_test_name_only",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag temporary directories/files that use only a fixed prefix and shared parent for parallel tests without unique isolation.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_proptest_unbounded_string_or_vec_strategy",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `\".*\"`, `any::<String>()`, or unbounded `vec(...)` strategies in parser/scanner tests without a size cap.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_proptest_assume_filters_most_cases",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag multiple `prop_assume!` calls or assume-heavy strategies that likely discard most generated cases.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_proptest_strategy_recreates_expensive_fixture",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag strategies that rebuild repositories, parsers, or large fixtures for each case rather than sharing setup safely.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_proptest_no_regression_case_for_parser_crash",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag parser property tests without a regression fixture or seed-capture path for minimized failures.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libfuzzer_target_unwraps_parse_or_utf8",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag fuzz targets that call `unwrap`/`expect` on parser, UTF-8, TOML, JSON, or path conversion results.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_libfuzzer_target_allocates_unbounded_string",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag fuzz targets that convert arbitrary byte slices to owned strings without a size cap or lossy/borrowed strategy.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_collect_then_single_iteration",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `.collect::<Vec<_>>()` immediately followed by one `for` loop, `.iter().any`, `.iter().find`, or `.len()` where streaming would avoid allocation.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_vec_push_without_capacity_from_known_bound",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `Vec::new()` followed by pushes inside a loop over a known bounded iterator without `with_capacity`.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_string_push_without_capacity_from_known_bound",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `String::new()` plus repeated `push_str`, `push`, or `write!` with a known bound and no capacity reservation.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_format_macro_inside_append_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `format!` in loops when appending to an existing `String` would avoid temporary allocation.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_to_string_on_str_in_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `.to_string()` or `String::from(...)` on borrowed strings inside loops where borrowing would satisfy the callee.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_clone_to_satisfy_borrow_in_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `.clone()` in loops immediately passed by reference or consumed only for read-only access.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_regex_compiled_in_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `Regex::new(...)` inside loops or hot functions when the `regex` crate is imported.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_sort_then_first_or_last",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag sorting an entire collection only to take min/max-like first or last values.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_vec_remove_zero_in_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated `Vec::remove(0)` in loops where `VecDeque` or index traversal would avoid shifting.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_hashmap_contains_then_insert",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `contains_key` followed by `insert` or `get_mut` where `entry` would avoid duplicate hashing.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_iterator_nth_inside_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `.nth(i)` or repeated indexed traversal over non-indexed iterators inside loops.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_drain_collect_then_drop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `drain(..).collect::<Vec<_>>()` followed by drop or one-pass processing that can operate directly on the drain iterator.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_vec_extend_from_intermediate_allocation",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag copying a slice into a temporary Vec and immediately extending another Vec from it.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_bytes_to_vec_for_readonly_use",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `bytes.to_vec()` on byte slices that are only read afterward.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_cow_to_owned_without_mutation",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `Cow::to_mut`, `to_owned`, or `into_owned` where the owned value is never mutated or stored past the borrow lifetime.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_option_clone_then_unwrap_or",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag cloning `Option<T>` or `Result<T, E>` only to unwrap/default instead of borrowing with `as_ref`, `as_deref`, or `map`.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_large_enum_variant_without_boxing",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag enums with one much larger variant causing every enum value to carry the largest layout.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_boxed_trait_object_in_inner_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated `Box<dyn Trait>` allocation inside loops where generics, enum dispatch, or object reuse may be better.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_iterator_chain_allocates_intermediate_strings",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag iterator chains that map every item through `format!`, `to_string`, or JSON conversion before a simple predicate or grouping.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_read_to_string_for_line_scan",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag whole-file `read_to_string` or `fs::read_to_string` followed only by line scanning or predicate checks.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_file_open_in_loop_without_buffered_reader",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag opening and reading files inside loops without `BufReader`, batching, or reuse.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_flush_inside_write_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `flush()` inside per-item write loops unless the code is interactive terminal output.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_create_dir_all_per_file",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `create_dir_all` inside loops for the same parent directory.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_metadata_called_repeatedly_same_path",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated `metadata`, `symlink_metadata`, `exists`, or `is_file` checks for the same path in one function.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_path_to_string_lossy_in_hot_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `to_string_lossy`, `display().to_string`, or path formatting inside repository/file traversal loops.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_stdout_println_in_library_or_hot_path",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `println`, `eprintln`, or direct stdout/stderr writes inside library code, scan loops, or reusable components.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_read_dir_collect_sort_before_filter",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `read_dir` entries collected and sorted before filtering by type/extension.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_file_handle_returned_without_close_owner_contract",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag functions that return raw file handles, descriptors, or paths tied to temporary resources without documenting ownership.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_blocking_process_output_read_unbounded",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `Command::output()` or piped process reads where stdout/stderr may be large and no size bound exists.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_path_canonicalize_in_scan_inner_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated canonicalization for every path in a hot repository traversal when relative normalized paths would be enough.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_os_string_lossy_conversion_before_filter",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag lossy path/string conversions before simple extension, file-name, or component filters.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_mutex_guard_lives_through_cpu_heavy_work",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag lock guards that remain live across sorting, parsing, serialization, filesystem I/O, or large loops.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_rwlock_write_guard_for_readonly_access",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag write locks where the guarded value is only read.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_mutex_lock_unwrap_panics_on_poison",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `lock().unwrap ()` or `write().unwrap ()` in production code where poison recovery or contextual error reporting would be safer.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_atomic_seqcst_without_comment",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `Ordering::SeqCst` in non-trivial code without a nearby comment explaining the synchronization requirement.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_unbounded_channel_in_producer_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag unbounded channel sends inside loops or request paths without backpressure or shutdown policy.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_thread_spawn_in_loop_without_join_limit",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `std::thread::spawn` in loops without a join handle collection limit, pool, or semaphore.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_arc_clone_inside_inner_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag repeated `Arc::clone` in inner loops when a borrowed reference or cloned handle outside the loop would work.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_mpsc_receiver_iter_without_shutdown_signal",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag blocking receiver iteration without timeout, close path, cancellation, or sentinel.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_condvar_wait_without_predicate_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `Condvar::wait` not wrapped in a predicate loop.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_sleep_polling_loop",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `thread::sleep` or runtime sleep in polling loops without backoff, notification, or timeout ownership.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_join_handle_dropped_after_spawn",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag spawned threads whose join handles are immediately dropped outside explicit detached-worker patterns.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_once_lock_initializes_fallible_resource_with_unwrap",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Warning,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `OnceLock`, `LazyLock`, or lazy initialization that unwraps fallible setup instead of returning initialization errors at bootstrap.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_manifest_wildcard_dependency_version",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `*` dependency versions or unconstrained git/path dependencies outside local workspace development.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_manifest_dependency_default_features_unreviewed",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag heavy dependencies with default features enabled when only a narrow feature set appears to be used.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_manifest_duplicate_direct_dependency_versions",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag the same crate required at multiple direct versions across workspace members.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_manifest_dev_dependency_used_in_src",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag dev-only crates such as `tempfile`, `proptest`, or fuzz helpers imported from production `src` code.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_manifest_build_dependency_used_at_runtime",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag build-dependencies imported from runtime code or runtime dependencies used only by `build.rs`.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_manifest_workspace_dependency_not_centralized",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag multi-crate workspaces that repeat dependency versions instead of using `[workspace.dependencies]`.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_manifest_release_lto_missing_for_cli_binary",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag CLI/binary crates with release profiles that omit any LTO setting when binary size or startup matters.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_manifest_bench_or_fuzz_target_in_default_members",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag fuzz/bench crates included in default workspace members without an explicit opt-in.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_build_script_missing_rerun_if_changed",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag `build.rs` files that read files, env vars, or external commands without `cargo:rerun-if-changed` or `cargo:rerun-if-env-changed`.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
    crate::rules::catalog::RuleDefinition {
        id: "rust_build_script_network_or_git_call",
        language: crate::rules::catalog::RuleLanguage::Rust,
        family: "bad_practices",
        default_severity: crate::rules::catalog::RuleDefaultSeverity::Contextual,
        status: crate::rules::catalog::RuleStatus::Stable,
        configurability: &[
            crate::rules::catalog::RuleConfigurability::Disable,
            crate::rules::catalog::RuleConfigurability::Ignore,
            crate::rules::catalog::RuleConfigurability::SeverityOverride,
        ],
        description: "flag build scripts that invoke network, git, curl, package managers, or shell commands that make builds non-hermetic.",
        binding_location: crate::rules::catalog::bindings::RUST_BAD_PRACTICES,
    },
];

#[derive(Debug, Clone, Copy)]
struct RuleSpec {
    id: &'static str,
    section: &'static str,
    description: &'static str,
    markers: &'static [&'static str],
    import_gate: &'static [&'static str],
    loop_only: bool,
    test_only: bool,
    manifest_only: bool,
    file_only: bool,
}

const RULE_SPECS: &[RuleSpec] = &[
    RuleSpec {
        id: "rust_tree_sitter_parser_created_per_file_without_reuse",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag `tree_sitter::Parser::new()` construction inside per-file or per-function loops where a language-specific parser could be reused per worker.",
        markers: &[
            "tree_sitter::Parser::new()",
            "tree",
            "sitter",
            "parser",
            "created",
            "file",
            "reuse",
        ],
        import_gate: &["tree_sitter"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_set_language_repeated_inside_hot_loop",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag repeated `parser.set_language(...)` calls inside loops that parse the same language repeatedly.",
        markers: &["parser.set_language(", "tree", "sitter", "language"],
        import_gate: &["tree_sitter"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_language_conversion_inside_loop",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag repeated `tree_sitter_rust::LANGUAGE.into()` or equivalent grammar conversion in inner loops instead of caching the converted language.",
        markers: &[
            "tree_sitter_rust::LANGUAGE.into()",
            "tree",
            "sitter",
            "language",
            "conversion",
        ],
        import_gate: &["tree_sitter"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_query_compiled_per_call",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag `tree_sitter::Query::new(...)` inside functions called per file or per node, where query compilation should be cached.",
        markers: &[
            "tree_sitter::Query::new(",
            "tree",
            "sitter",
            "query",
            "compiled",
        ],
        import_gate: &["tree_sitter"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_parse_result_unwrapped",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag `parser.parse(...).unwrap ()` or `expect (...)` at parser boundaries where parse cancellation or invalid parser state should become a typed finding/error.",
        markers: &[
            "parser.parse(.unwrap ()",
            "expect (",
            "tree",
            "sitter",
            "parse",
            "result",
            "unwrapped",
        ],
        import_gate: &["tree_sitter"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_error_tree_ignored",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag parser pipelines that use the root node without checking `tree.root_node().has_error()` or equivalent syntax-error evidence.",
        markers: &[
            "tree.root_node().has_error()",
            "tree",
            "sitter",
            "error",
            "ignored",
        ],
        import_gate: &["tree_sitter"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_recursive_walk_without_depth_guard",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag recursive AST walkers over repository input without an explicit depth guard, iterative cursor, or stack budget.",
        markers: &["tree", "sitter", "recursive", "walk", "depth", "guard"],
        import_gate: &["tree_sitter"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_node_text_redecoded_in_nested_loop",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag repeated `node.utf8_text(source)` or source slicing for the same node inside nested traversal loops.",
        markers: &[
            "node.utf8_text(source)",
            "tree",
            "sitter",
            "node",
            "text",
            "redecoded",
            "nested",
        ],
        import_gate: &["tree_sitter"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_child_lookup_by_field_name_in_hot_walk",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag repeated `child_by_field_name(\"...\")` string lookups in hot recursive walkers where field ids, cursors, or one-pass extraction would avoid repeated lookup work.",
        markers: &[
            "child_by_field_name(\"\")",
            "tree",
            "sitter",
            "child",
            "lookup",
            "field",
            "name",
            "walk",
        ],
        import_gate: &["tree_sitter"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_descendant_for_point_range_in_loop",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag repeated descendant/range queries inside loops over sibling nodes, which can turn a tree walk into avoidable quadratic traversal.",
        markers: &["tree", "sitter", "descendant", "point", "range"],
        import_gate: &["tree_sitter"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_collects_all_captures_before_filtering",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag query capture pipelines that collect every capture into `Vec` and then filter locally instead of filtering as captures are visited.",
        markers: &["Vec", "tree", "sitter", "collects", "captures", "filtering"],
        import_gate: &["tree_sitter"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_byte_offset_used_as_char_index",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag byte offsets from `start_byte` or `end_byte` used as character counts, `chars().nth(...)`, or display-column positions without conversion.",
        markers: &[
            "start_byte",
            "end_byte",
            "chars().nth(",
            "tree",
            "sitter",
            "byte",
            "offset",
            "char",
        ],
        import_gate: &["tree_sitter"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_old_tree_discarded_in_reparse_loop",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag repeated parse loops over the same buffer that always pass `None` instead of reusing an old tree for incremental parsing.",
        markers: &["None", "tree", "sitter", "discarded", "reparse"],
        import_gate: &["tree_sitter"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tree_sitter_parser_shared_global_with_lock",
        section: "tree-sitter Parser Pipeline Rules",
        description: "flag global `Mutex<Parser>` or `RwLock<Parser>` shared across threads where per-thread parser ownership would avoid lock contention and parser state coupling.",
        markers: &[
            "Mutex<Parser>",
            "RwLock<Parser>",
            "tree",
            "sitter",
            "parser",
            "shared",
            "global",
            "lock",
        ],
        import_gate: &["tree_sitter"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_nested_parallel_iterators",
        section: "rayon Parallel Pipeline Rules",
        description: "flag nested `par_iter`, `into_par_iter`, or `par_bridge` calls that can oversubscribe worker threads or fragment work.",
        markers: &[
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "nested",
            "parallel",
            "iterators",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_mutex_push_in_parallel_loop",
        section: "rayon Parallel Pipeline Rules",
        description: "flag `Mutex<Vec<_>>`, `RwLock<Vec<_>>`, or shared collection mutation inside `for_each` where `map/filter_map/collect/reduce` would avoid lock contention.",
        markers: &[
            "Mutex<Vec<>",
            "RwLock<Vec<>",
            "for_each",
            "map/filter_map/collect/reduce",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_collect_all_then_filter_sequentially",
        section: "rayon Parallel Pipeline Rules",
        description: "flag `par_iter().map(...).collect::<Vec<_>>()` immediately followed by sequential filtering or flattening that could stay in the parallel pipeline.",
        markers: &[
            "par_iter().map(.collect::<Vec<>()",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "collect::<Vec<_>>()",
            "filter",
            "sequentially",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_sequential_collect_then_par_iter",
        section: "rayon Parallel Pipeline Rules",
        description: "flag collecting an intermediate `Vec` only to immediately call `par_iter` or `into_par_iter` when the producer could feed the parallel stage directly.",
        markers: &[
            "Vec",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "sequential",
            "collect::<Vec<_>>()",
            "iter",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_par_bridge_on_small_or_indexed_iterator",
        section: "rayon Parallel Pipeline Rules",
        description: "flag `par_bridge()` over known indexed collections or tiny iterators where normal `par_iter` or sequential iteration is likely cheaper.",
        markers: &[
            "par_bridge()",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "bridge",
            "small",
            "indexed",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_blocking_io_in_cpu_parallel_pool",
        section: "rayon Parallel Pipeline Rules",
        description: "flag filesystem, network, or process I/O inside Rayon CPU-bound closures unless the project documents that Rayon owns the I/O concurrency budget.",
        markers: &[
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "blocking",
            "parallel",
            "pool",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_heavy_clone_per_item",
        section: "rayon Parallel Pipeline Rules",
        description: "flag repeated `.clone()` of config, source text, rule catalogs, AST summaries, or large state inside parallel closures.",
        markers: &[
            ".clone()",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "heavy",
            "clone",
            "item",
        ],
        import_gate: &["rayon"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_large_move_capture",
        section: "rayon Parallel Pipeline Rules",
        description: "flag `move` closures in parallel iterators that capture large owned values rather than borrowing or sharing cheap handles.",
        markers: &[
            "move",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "large",
            "capture",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_ordering_mutex_for_result_stability",
        section: "rayon Parallel Pipeline Rules",
        description: "flag parallel closures that serialize through `Mutex<BTreeMap<_>>`, `Mutex<Vec<_>>`, or sequence counters just to regain output order.",
        markers: &[
            "Mutex<BTreeMap<>",
            "Mutex<Vec<>",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "ordering",
            "Mutex",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_global_pool_built_by_library_code",
        section: "rayon Parallel Pipeline Rules",
        description: "flag `ThreadPoolBuilder::build_global()` outside a binary entry point, test harness, or explicit process bootstrap.",
        markers: &[
            "ThreadPoolBuilder::build_global()",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "global",
            "pool",
            "built",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_custom_pool_created_per_call",
        section: "rayon Parallel Pipeline Rules",
        description: "flag `ThreadPoolBuilder::build()` inside frequently-called functions instead of process-level or subsystem-level pool ownership.",
        markers: &[
            "ThreadPoolBuilder::build()",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "custom",
            "pool",
            "created",
        ],
        import_gate: &["rayon"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_unhandled_panic_in_parallel_scan",
        section: "rayon Parallel Pipeline Rules",
        description: "flag `unwrap`, `expect`, or panicking conversions inside parallel scan/evaluation closures where one bad file can abort the whole batch.",
        markers: &[
            "unwrap",
            "expect",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "unhandled",
            "panic",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_flat_map_allocates_nested_vectors",
        section: "rayon Parallel Pipeline Rules",
        description: "flag parallel map stages that return `Vec<Vec<T>>` and later flatten, where `flat_map_iter`, `reduce`, or direct collection can reduce allocation churn.",
        markers: &[
            "Vec<Vec<T>>",
            "flat_map_iter",
            "reduce",
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "flat",
        ],
        import_gate: &["rayon"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rayon_parallelism_for_trivial_per_item_work",
        section: "rayon Parallel Pipeline Rules",
        description: "flag parallel iteration over simple string predicates or metadata checks when there is no expensive per-item work and the input is below a conservative threshold.",
        markers: &[
            "par_iter",
            "into_par_iter",
            "par_bridge",
            "ThreadPoolBuilder",
            "parallelism",
            "trivial",
            "item",
            "work",
        ],
        import_gate: &["rayon"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_walker_disables_standard_filters_without_policy",
        section: "ignore Repository Walker Rules",
        description: "flag `.ignore(false)`, `.git_ignore(false)`, `.git_exclude(false)`, `.hidden(false)`, or `.parents(false)` unless paired with a named user option or security review note.",
        markers: &[
            ".ignore(false)",
            ".git_ignore(false)",
            ".git_exclude(false)",
            ".hidden(false)",
            ".parents(false)",
            "WalkBuilder::new",
            "OverrideBuilder::add",
            "filter_map(Result::ok)",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_follow_links_without_cycle_or_root_policy",
        section: "ignore Repository Walker Rules",
        description: "flag `.follow_links(true)` without visible cycle, root containment, or same-filesystem policy.",
        markers: &[
            ".follow_links(true)",
            "WalkBuilder::new",
            "OverrideBuilder::add",
            "filter_map(Result::ok)",
            "follow",
            "links",
            "cycle",
            "root",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_walk_error_silently_discarded",
        section: "ignore Repository Walker Rules",
        description: "flag walk result handling that uses `filter_map(Result::ok)` or `ok()?` without recording ignored traversal errors.",
        markers: &[
            "filter_map(Result::ok)",
            "ok()?",
            "WalkBuilder::new",
            "OverrideBuilder::add",
            ".follow_links(true)",
            "walk",
            "error",
            "silently",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_direntry_unwrapped",
        section: "ignore Repository Walker Rules",
        description: "flag `entry.unwrap ()` or `expect (...)` on walker entries in production scan code.",
        markers: &[
            "entry.unwrap ()",
            "expect (",
            "WalkBuilder::new",
            "OverrideBuilder::add",
            "filter_map(Result::ok)",
            ".follow_links(true)",
            "direntry",
            "unwrapped",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_reads_file_before_file_type_check",
        section: "ignore Repository Walker Rules",
        description: "flag read/open operations on walker paths before checking `file_type().is_file()` or equivalent.",
        markers: &[
            "file_type().is_file()",
            "WalkBuilder::new",
            "OverrideBuilder::add",
            "filter_map(Result::ok)",
            ".follow_links(true)",
            "reads",
            "file",
            "type",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_walker_rebuilt_inside_directory_loop",
        section: "ignore Repository Walker Rules",
        description: "flag `WalkBuilder::new(...)` construction inside recursive or per-directory loops.",
        markers: &[
            "WalkBuilder::new(",
            "WalkBuilder::new",
            "OverrideBuilder::add",
            "filter_map(Result::ok)",
            ".follow_links(true)",
            "walker",
            "rebuilt",
            "directory",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_override_pattern_unwraps_user_input",
        section: "ignore Repository Walker Rules",
        description: "flag `OverrideBuilder::add(...)` or glob override construction from user input followed by `unwrap` or `expect`.",
        markers: &[
            "OverrideBuilder::add(",
            "unwrap",
            "expect",
            "WalkBuilder::new",
            "OverrideBuilder::add",
            "filter_map(Result::ok)",
            ".follow_links(true)",
            "override",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_parallel_walker_unbounded_accumulation",
        section: "ignore Repository Walker Rules",
        description: "flag parallel walker callbacks that push every path into an unbounded `Vec` or channel before processing.",
        markers: &[
            "Vec",
            "WalkBuilder::new",
            "OverrideBuilder::add",
            "filter_map(Result::ok)",
            ".follow_links(true)",
            "parallel",
            "walker",
            "accumulation",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_filter_entry_allocates_path_string_per_node",
        section: "ignore Repository Walker Rules",
        description: "flag `filter_entry` closures that allocate `String` or call `to_string_lossy()` for every node when path component checks would work.",
        markers: &[
            "filter_entry",
            "String",
            "to_string_lossy()",
            "WalkBuilder::new",
            "OverrideBuilder::add",
            "filter_map(Result::ok)",
            ".follow_links(true)",
            "filter",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ignore_sort_by_path_after_full_walk",
        section: "ignore Repository Walker Rules",
        description: "flag full repository path collection followed by sort before processing where deterministic output could be produced after finding collection instead.",
        markers: &[
            "WalkBuilder::new",
            "OverrideBuilder::add",
            "filter_map(Result::ok)",
            ".follow_links(true)",
            "sort",
            "Path::",
            "canonicalize(",
            "to_string_lossy(",
        ],
        import_gate: &["ignore"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_json_value_internal_hot_path",
        section: "serde, serde_json, and toml Rules",
        description: "flag `serde_json::Value` used as an internal data model in non-boundary modules where typed structs would reduce dynamic lookups and runtime errors.",
        markers: &[
            "serde_json::Value",
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
            "json",
            "value",
            "internal",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_json_indexing_without_type_guard",
        section: "serde, serde_json, and toml Rules",
        description: "flag `value[\"key\"]`, `value[index]`, or chained indexing without typed deserialization, `.get(...)`, or null/type checks.",
        markers: &[
            "value[\"key\"]",
            "value[index]",
            ".get(",
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
            "json",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_json_to_string_pretty_in_machine_path",
        section: "serde, serde_json, and toml Rules",
        description: "flag `serde_json::to_string_pretty` in non-human-output hot paths, APIs, or report generation loops.",
        markers: &[
            "serde_json::to_string_pretty",
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
            "json",
            "String::new()",
            "push_str(",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_json_clone_value_in_loop",
        section: "serde, serde_json, and toml Rules",
        description: "flag `serde_json::Value::clone()` or cloned JSON maps inside loops.",
        markers: &[
            "serde_json::Value::clone()",
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
            "json",
            "clone",
            "value",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_json_from_str_after_unbounded_read",
        section: "serde, serde_json, and toml Rules",
        description: "flag `serde_json::from_str` or `from_slice` fed by unbounded file/network reads rather than size-limited input.",
        markers: &[
            "serde_json::from_str",
            "from_slice",
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
            "json",
            "read",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_json_roundtrip_conversion",
        section: "serde, serde_json, and toml Rules",
        description: "flag `to_value` followed by `from_value`, or `to_string` followed by `from_str`, when a direct conversion or typed boundary is available.",
        markers: &[
            "to_value",
            "from_value",
            "to_string",
            "from_str",
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_json_whole_array_loaded_for_streaming_input",
        section: "serde, serde_json, and toml Rules",
        description: "flag deserializing large JSON arrays into `Vec<T>` in reader-like code where streaming deserialization could reduce peak memory.",
        markers: &[
            "Vec<T>",
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
            "json",
            "whole",
            "array",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_custom_deserialize_panics",
        section: "serde, serde_json, and toml Rules",
        description: "flag custom `Deserialize` impls or visitors that call `unwrap`, `expect`, `panic`, or unchecked indexing.",
        markers: &[
            "Deserialize",
            "unwrap",
            "expect",
            "panic",
            "serde_json",
            "serde(",
            "Serialize",
            "custom",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_borrow_missing_for_large_readonly_payload",
        section: "serde, serde_json, and toml Rules",
        description: "flag large readonly payload structs that deserialize owned `String` or `Vec<u8>` in hot paths where `Cow<'de, str>` or borrowed fields could avoid copies.",
        markers: &[
            "String",
            "Vec<u8>",
            "Cow<'de, str>",
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
            "borrow",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_skip_serializing_secret_without_deserialize_guard",
        section: "serde, serde_json, and toml Rules",
        description: "flag secret-like fields skipped during serialization but still accepted during deserialization without validation/redaction policy.",
        markers: &[
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
            "skip",
            "serializing",
            "secret",
            "deserialize",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_wire_enum_missing_stable_rename_policy",
        section: "serde, serde_json, and toml Rules",
        description: "flag public wire-facing enums that derive `Serialize` or `Deserialize` without `rename_all` or explicit renames.",
        markers: &[
            "Serialize",
            "Deserialize",
            "rename_all",
            "serde_json",
            "serde(",
            "wire",
            "enum",
            "stable",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: true,
    },
    RuleSpec {
        id: "rust_toml_value_config_boundary",
        section: "serde, serde_json, and toml Rules",
        description: "flag application config parsed into `toml::Value` and queried dynamically instead of deserializing into a typed config struct.",
        markers: &[
            "toml::Value",
            "toml::",
            "toml::from_str",
            "value",
            "config",
            "boundary",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_toml_parse_in_hot_path",
        section: "serde, serde_json, and toml Rules",
        description: "flag `toml::from_str`, `toml::from_slice`, or `str::parse::<toml::Value>()` inside request paths, scan loops, or repeated functions.",
        markers: &[
            "toml::from_str",
            "toml::from_slice",
            "str::parse::<toml::Value>()",
            "toml::",
            "toml::Value",
            "parse",
            "Path::",
            "canonicalize(",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_toml_config_without_unknown_field_rejection",
        section: "serde, serde_json, and toml Rules",
        description: "flag TOML-facing config structs that derive `Deserialize` without `#[serde(deny_unknown_fields)]` when they appear to represent project configuration.",
        markers: &[
            "Deserialize",
            "#[serde(deny_unknown_fields)]",
            "toml::",
            "toml::from_str",
            "toml::Value",
            "config",
            "unknown",
            "field",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: true,
    },
    RuleSpec {
        id: "rust_toml_manifest_parse_without_size_limit",
        section: "serde, serde_json, and toml Rules",
        description: "flag manifest/config parsing helpers that read and parse TOML without a byte limit.",
        markers: &[
            "toml::",
            "toml::from_str",
            "toml::Value",
            "[dependencies]",
            "[workspace]",
            "[profile.release]",
            "parse",
            "size",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_serde_default_masks_parse_error",
        section: "serde, serde_json, and toml Rules",
        description: "flag broad `unwrap_or_default`, `Default::default`, or `#[serde(default)]` use around configuration fields that look required for correctness.",
        markers: &[
            "unwrap_or_default",
            "Default::default",
            "#[serde(default)]",
            "serde_json",
            "serde(",
            "Deserialize",
            "Serialize",
            "default",
        ],
        import_gate: &["serde", "serde_json", "toml"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_anyhow_context_missing_on_boundary_io",
        section: "anyhow and thiserror Rules",
        description: "flag `?` on filesystem, environment, process, parser, config, or network calls in CLI/boundary code without `.context(...)` or `.with_context(...)`.",
        markers: &[
            ".context(",
            ".with_context(",
            "anyhow",
            "bail!",
            "context",
            "boundary",
        ],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_anyhow_eager_format_context",
        section: "anyhow and thiserror Rules",
        description: "flag `.context(format!(...))` where `.with_context(|| format!(...))` would avoid allocation on the success path.",
        markers: &[
            ".context(format!()",
            ".with_context(|| format!()",
            "anyhow",
            ".context(",
            ".with_context(",
            "bail!",
            "eager",
            "format",
        ],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_anyhow_error_string_matching",
        section: "anyhow and thiserror Rules",
        description: "flag branching on `err.to_string()`, `format!(\"{err}\")`, or message substrings instead of typed errors or downcasts.",
        markers: &[
            "err.to_string()",
            "format!(\"{err}\")",
            "anyhow",
            ".context(",
            ".with_context(",
            "bail!",
            "error",
            "String::new()",
        ],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_anyhow_downcast_without_fallback_context",
        section: "anyhow and thiserror Rules",
        description: "flag `downcast_ref`/`downcast` handling that drops the original context or returns a generic fallback.",
        markers: &[
            "downcast_ref",
            "downcast",
            "anyhow",
            ".context(",
            ".with_context(",
            "bail!",
            "fallback",
            "context",
        ],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_anyhow_bail_in_low_level_library_module",
        section: "anyhow and thiserror Rules",
        description: "flag `anyhow::bail!` in domain, parser, storage, or library modules that should usually expose typed error variants.",
        markers: &[
            "anyhow::bail!",
            "anyhow",
            ".context(",
            ".with_context(",
            "bail!",
            "bail",
            "level",
            "library",
        ],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_thiserror_variant_wraps_source_without_source_attr",
        section: "anyhow and thiserror Rules",
        description: "flag error variants with source-like fields that lack `#[source]`, `#[from]`, or transparent wrapping.",
        markers: &[
            "#[source]",
            "#[from]",
            "thiserror",
            "#[error(",
            "variant",
            "wraps",
            "source",
            "attr",
        ],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_thiserror_display_leaks_secret_field",
        section: "anyhow and thiserror Rules",
        description: "flag `#[error(...)]` format strings that interpolate token, password, secret, key, auth, cookie, or credential fields.",
        markers: &[
            "#[error(]",
            "thiserror",
            "#[error(",
            "#[source]",
            "#[from]",
            "display",
            "leaks",
            "secret",
        ],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_thiserror_stringly_typed_variant",
        section: "anyhow and thiserror Rules",
        description: "flag error enum variants whose only payload is `String` or `&'static str` and whose name does not encode a specific error kind.",
        markers: &[
            "String",
            "&'static str",
            "thiserror",
            "#[error(",
            "#[source]",
            "#[from]",
            "stringly",
            "typed",
        ],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_thiserror_transparent_on_contextual_variant",
        section: "anyhow and thiserror Rules",
        description: "flag `#[error(transparent)]` variants that also carry context-like fields or lose higher-level operation details.",
        markers: &[
            "#[error(transparent)]",
            "thiserror",
            "#[error(",
            "#[source]",
            "#[from]",
            "transparent",
            "contextual",
            "variant",
        ],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_error_logged_and_returned",
        section: "anyhow and thiserror Rules",
        description: "flag functions that log an error and then return the same error upward, causing duplicate logging at callers.",
        markers: &["error", "logged", "returned"],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_result_ignored_with_let_underscore",
        section: "anyhow and thiserror Rules",
        description: "flag `let _ = fallible_call()` outside cleanup, telemetry, or best-effort contexts.",
        markers: &["let _ = fallible_call()", "result", "ignored", "underscore"],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_question_mark_after_partial_side_effect_without_cleanup",
        section: "anyhow and thiserror Rules",
        description: "flag `?` after partially mutating files, shared state, transactions, or output buffers without rollback or cleanup.",
        markers: &["question", "mark", "partial", "side", "effect", "cleanup"],
        import_gate: &["anyhow", "thiserror"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clap_closed_set_manual_string_match",
        section: "clap CLI Boundary Rules",
        description: "flag CLI string arguments that are manually matched against a fixed set instead of using `ValueEnum` or `value_parser`.",
        markers: &[
            "ValueEnum",
            "value_parser",
            "clap",
            "#[arg(",
            "Cli::parse",
            "closed",
            "manual",
            "String::new()",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clap_path_arg_used_without_validation",
        section: "clap CLI Boundary Rules",
        description: "flag `PathBuf` or path-like CLI values used for reads/writes without root containment, canonicalization strategy, or symlink policy.",
        markers: &[
            "PathBuf",
            "clap",
            "#[arg(",
            "ValueEnum",
            "Cli::parse",
            "Path::",
            "canonicalize(",
            "to_string_lossy(",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clap_default_value_manual_parse",
        section: "clap CLI Boundary Rules",
        description: "flag `default_value` strings that are parsed manually later instead of typed `value_parser` or typed fields.",
        markers: &[
            "default_value",
            "value_parser",
            "clap",
            "#[arg(",
            "ValueEnum",
            "Cli::parse",
            "default",
            "value",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clap_unbounded_vec_arg_on_scan_path",
        section: "clap CLI Boundary Rules",
        description: "flag `Vec<String>` or variadic CLI arguments that feed scan/filter work without a limit, deduplication, or validation.",
        markers: &[
            "Vec<String>",
            "clap",
            "#[arg(",
            "ValueEnum",
            "Cli::parse",
            "Vec::new()",
            "Vec::with_capacity(",
            "remove(0)",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clap_secret_arg_derive_debug",
        section: "clap CLI Boundary Rules",
        description: "flag CLI structs deriving `Debug` while containing token, password, secret, key, cookie, or auth fields.",
        markers: &[
            "Debug",
            "clap",
            "#[arg(",
            "ValueEnum",
            "Cli::parse",
            "secret",
            "derive",
            "debug",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: true,
    },
    RuleSpec {
        id: "rust_clap_opposing_flags_without_conflicts",
        section: "clap CLI Boundary Rules",
        description: "flag pairs such as `--json/--text`, `--include/--exclude`, or `--enable/--disable` without `conflicts_with`, `overrides_with`, or explicit precedence.",
        markers: &[
            "--json/--text",
            "--include/--exclude",
            "--enable/--disable",
            "conflicts_with",
            "overrides_with",
            "clap",
            "#[arg(",
            "ValueEnum",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clap_rule_id_arg_without_catalog_validation",
        section: "clap CLI Boundary Rules",
        description: "flag rule-id-like CLI arguments accepted as strings without checking against the known rule registry.",
        markers: &[
            "clap",
            "#[arg(",
            "ValueEnum",
            "Cli::parse",
            "catalog",
            "validation",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clap_subcommand_reloads_config_in_each_branch",
        section: "clap CLI Boundary Rules",
        description: "flag repeated config loading inside every subcommand branch instead of a shared pre-dispatch normalization step.",
        markers: &[
            "clap",
            "#[arg(",
            "ValueEnum",
            "Cli::parse",
            "subcommand",
            "reloads",
            "config",
            "each",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clap_parse_called_below_main",
        section: "clap CLI Boundary Rules",
        description: "flag `Cli::parse()` or `try_parse()` called from library code or tests without dependency injection, making the code hard to reuse and test.",
        markers: &[
            "Cli::parse()",
            "try_parse()",
            "clap",
            "#[arg(",
            "ValueEnum",
            "Cli::parse",
            "parse",
            "below",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clap_env_var_without_redaction_policy",
        section: "clap CLI Boundary Rules",
        description: "flag `#[arg(env = \"...\")]` on secret-like fields without redacted display/report behavior.",
        markers: &[
            "#[arg(env = \"\")]",
            "clap",
            "#[arg(",
            "ValueEnum",
            "Cli::parse",
            "redaction",
            "policy",
        ],
        import_gate: &["clap"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libc_call_without_platform_cfg",
        section: "libc and FFI Boundary Rules",
        description: "flag direct `libc::*` calls without `#[cfg(unix)]`, `#[cfg(target_os = \"...\")]`, or a platform abstraction.",
        markers: &[
            "libc::*",
            "#[cfg(unix)]",
            "#[cfg(target_os = \"\")]",
            "libc::",
            "CString::new(",
            "as_raw_fd(",
            "into_raw_fd(",
            "platform",
        ],
        import_gate: &["libc"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libc_return_value_not_checked",
        section: "libc and FFI Boundary Rules",
        description: "flag libc calls whose integer or pointer return value is ignored or not checked for `-1`, null, or documented failure sentinels.",
        markers: &[
            "libc::",
            "CString::new(",
            "as_raw_fd(",
            "into_raw_fd(",
            "return",
            "value",
            "checked",
        ],
        import_gate: &["libc"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libc_errno_read_after_intervening_call",
        section: "libc and FFI Boundary Rules",
        description: "flag errno access after another call that could overwrite the original failure cause.",
        markers: &[
            "libc::",
            "CString::new(",
            "as_raw_fd(",
            "into_raw_fd(",
            "errno",
            "read",
            "intervening",
        ],
        import_gate: &["libc"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libc_raw_fd_lifetime_escape",
        section: "libc and FFI Boundary Rules",
        description: "flag `as_raw_fd()` results stored, returned, or moved into long-lived structs while the owning file/socket may drop.",
        markers: &[
            "as_raw_fd()",
            "libc::",
            "CString::new(",
            "as_raw_fd(",
            "into_raw_fd(",
            "lifetime",
            "escape",
        ],
        import_gate: &["libc"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libc_into_raw_fd_without_reclaim",
        section: "libc and FFI Boundary Rules",
        description: "flag `into_raw_fd()` without a visible `from_raw_fd`, `OwnedFd`, or close handoff.",
        markers: &[
            "into_raw_fd()",
            "from_raw_fd",
            "OwnedFd",
            "libc::",
            "CString::new(",
            "as_raw_fd(",
            "into_raw_fd(",
            "reclaim",
        ],
        import_gate: &["libc"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libc_cstring_unwrap_on_external_input",
        section: "libc and FFI Boundary Rules",
        description: "flag `CString::new(user_input).unwrap ()` or `expect (...)` where embedded NUL bytes should become a recoverable error.",
        markers: &[
            "CString::new(user_input).unwrap ()",
            "expect (",
            "libc::",
            "CString::new(",
            "as_raw_fd(",
            "into_raw_fd(",
            "cstring",
            "unwrap",
        ],
        import_gate: &["libc"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libc_open_without_cloexec",
        section: "libc and FFI Boundary Rules",
        description: "flag direct `libc::open` or equivalent without `O_CLOEXEC` in programs that may spawn child processes.",
        markers: &[
            "libc::open",
            "O_CLOEXEC",
            "libc::",
            "CString::new(",
            "as_raw_fd(",
            "into_raw_fd(",
            "open",
            "cloexec",
        ],
        import_gate: &["libc"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libc_no_follow_without_eloop_handling",
        section: "libc and FFI Boundary Rules",
        description: "flag `O_NOFOLLOW` usage without explicit symlink-loop or unsupported-platform error handling.",
        markers: &[
            "O_NOFOLLOW",
            "libc::",
            "CString::new(",
            "as_raw_fd(",
            "into_raw_fd(",
            "follow",
            "eloop",
            "handling",
        ],
        import_gate: &["libc"],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ffi_extern_block_without_abi_comment",
        section: "libc and FFI Boundary Rules",
        description: "flag FFI declarations without a nearby note about ABI, ownership, nullability, or lifetime expectations.",
        markers: &[
            "extern \"C\"",
            "from_raw_parts",
            "extern",
            "block",
            "comment",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_ffi_slice_from_raw_parts_without_length_guard",
        section: "libc and FFI Boundary Rules",
        description: "flag safe wrappers around raw pointer and length pairs that do not validate null pointers, maximum length, or alignment before slice construction.",
        markers: &[
            "extern \"C\"",
            "from_raw_parts",
            "slice",
            "parts",
            "length",
            "guard",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tempfile_named_path_used_after_drop",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag `NamedTempFile::path()` stored or returned while the temporary file owner may drop before the path is used.",
        markers: &[
            "NamedTempFile::path()",
            "NamedTempFile",
            "tempfile::",
            "persist(",
            "keep(",
            "named",
            "Path::",
            "canonicalize(",
        ],
        import_gate: &["tempfile"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tempfile_persist_without_cleanup_assertion",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag `persist`, `keep`, or `into_temp_path().keep()` in tests without cleanup or an assertion that the file must survive.",
        markers: &[
            "persist",
            "keep",
            "into_temp_path().keep()",
            "NamedTempFile",
            "tempfile::",
            "persist(",
            "keep(",
            "cleanup",
        ],
        import_gate: &["tempfile"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tempfile_predictable_name_in_shared_tmp",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag tests that combine `std::env::temp_dir()` with predictable filenames instead of `tempfile`.",
        markers: &[
            "std::env::temp_dir()",
            "tempfile",
            "NamedTempFile",
            "tempfile::",
            "persist(",
            "keep(",
            "predictable",
            "name",
        ],
        import_gate: &["tempfile"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_tempfile_builder_prefix_from_test_name_only",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag temporary directories/files that use only a fixed prefix and shared parent for parallel tests without unique isolation.",
        markers: &[
            "NamedTempFile",
            "tempfile::",
            "persist(",
            "keep(",
            "builder",
            "prefix",
            "test",
            "name",
        ],
        import_gate: &["tempfile"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_proptest_unbounded_string_or_vec_strategy",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag `\".*\"`, `any::<String>()`, or unbounded `vec(...)` strategies in parser/scanner tests without a size cap.",
        markers: &[
            "\".*\"",
            "any::<String>()",
            "vec(",
            "proptest",
            "prop_assume!",
            "String::new()",
            "push_str(",
            "to_string(",
        ],
        import_gate: &["proptest"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_proptest_assume_filters_most_cases",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag multiple `prop_assume!` calls or assume-heavy strategies that likely discard most generated cases.",
        markers: &[
            "prop_assume!",
            "proptest",
            "any::<String>()",
            "vec(",
            "assume",
            "filters",
            "most",
            "cases",
        ],
        import_gate: &["proptest"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_proptest_strategy_recreates_expensive_fixture",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag strategies that rebuild repositories, parsers, or large fixtures for each case rather than sharing setup safely.",
        markers: &[
            "proptest",
            "prop_assume!",
            "any::<String>()",
            "vec(",
            "strategy",
            "expensive",
            "fixture",
        ],
        import_gate: &["proptest"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_proptest_no_regression_case_for_parser_crash",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag parser property tests without a regression fixture or seed-capture path for minimized failures.",
        markers: &[
            "proptest",
            "prop_assume!",
            "any::<String>()",
            "vec(",
            "regression",
            "case",
            "parser",
            "crash",
        ],
        import_gate: &["proptest"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libfuzzer_target_unwraps_parse_or_utf8",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag fuzz targets that call `unwrap`/`expect` on parser, UTF-8, TOML, JSON, or path conversion results.",
        markers: &[
            "unwrap",
            "expect",
            "libfuzzer",
            "fuzz_target!",
            "unwrap(",
            "String::from_utf8",
            "target",
            "unwraps",
        ],
        import_gate: &["libfuzzer"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_libfuzzer_target_allocates_unbounded_string",
        section: "proptest, tempfile, and libfuzzer Rules",
        description: "flag fuzz targets that convert arbitrary byte slices to owned strings without a size cap or lossy/borrowed strategy.",
        markers: &[
            "libfuzzer",
            "fuzz_target!",
            "unwrap(",
            "String::from_utf8",
            "target",
            "allocates",
            "String::new()",
            "push_str(",
        ],
        import_gate: &["libfuzzer"],
        loop_only: false,
        test_only: true,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_collect_then_single_iteration",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `.collect::<Vec<_>>()` immediately followed by one `for` loop, `.iter().any`, `.iter().find`, or `.len()` where streaming would avoid allocation.",
        markers: &[
            ".collect::<Vec<>()",
            "for",
            ".iter().any",
            ".iter().find",
            ".len()",
            "collect::<Vec<_>>()",
            "single",
            "iteration",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_vec_push_without_capacity_from_known_bound",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `Vec::new()` followed by pushes inside a loop over a known bounded iterator without `with_capacity`.",
        markers: &[
            "Vec::new()",
            "with_capacity",
            "Vec::with_capacity(",
            "remove(0)",
            "push",
            "capacity",
            "known",
            "bound",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_string_push_without_capacity_from_known_bound",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `String::new()` plus repeated `push_str`, `push`, or `write!` with a known bound and no capacity reservation.",
        markers: &[
            "String::new()",
            "push_str",
            "push",
            "write!",
            "push_str(",
            "to_string(",
            "capacity",
            "known",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_format_macro_inside_append_loop",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `format!` in loops when appending to an existing `String` would avoid temporary allocation.",
        markers: &["format!", "String", "format", "macro", "append"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_to_string_on_str_in_loop",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `.to_string()` or `String::from(...)` on borrowed strings inside loops where borrowing would satisfy the callee.",
        markers: &[
            ".to_string()",
            "String::from(",
            "String::new()",
            "push_str(",
            "to_string(",
        ],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_clone_to_satisfy_borrow_in_loop",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `.clone()` in loops immediately passed by reference or consumed only for read-only access.",
        markers: &[".clone()", "clone", "satisfy", "borrow"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_regex_compiled_in_loop",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `Regex::new(...)` inside loops or hot functions when the `regex` crate is imported.",
        markers: &["Regex::new(", "regex", "compiled"],
        import_gate: &["regex"],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_sort_then_first_or_last",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag sorting an entire collection only to take min/max-like first or last values.",
        markers: &["sort", "first", "last"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_vec_remove_zero_in_loop",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag repeated `Vec::remove(0)` in loops where `VecDeque` or index traversal would avoid shifting.",
        markers: &[
            "Vec::remove(0)",
            "VecDeque",
            "Vec::new()",
            "Vec::with_capacity(",
            "remove(0)",
            "remove",
            "zero",
        ],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_hashmap_contains_then_insert",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `contains_key` followed by `insert` or `get_mut` where `entry` would avoid duplicate hashing.",
        markers: &[
            "contains_key",
            "insert",
            "get_mut",
            "entry",
            "HashMap::",
            "contains_key(",
            ".insert(",
            ".entry(",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_iterator_nth_inside_loop",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `.nth(i)` or repeated indexed traversal over non-indexed iterators inside loops.",
        markers: &[".nth(i)", "iterator"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_drain_collect_then_drop",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `drain(..).collect::<Vec<_>>()` followed by drop or one-pass processing that can operate directly on the drain iterator.",
        markers: &[
            "drain(..).collect::<Vec<>()",
            "drain",
            "collect::<Vec<_>>()",
            "drop",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_vec_extend_from_intermediate_allocation",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag copying a slice into a temporary Vec and immediately extending another Vec from it.",
        markers: &[
            "current_module_segments",
            "segments.to_vec()",
            "resolved.extend(",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_bytes_to_vec_for_readonly_use",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `bytes.to_vec()` on byte slices that are only read afterward.",
        markers: &["bytes.to_vec()"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_cow_to_owned_without_mutation",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag `Cow::to_mut`, `to_owned`, or `into_owned` where the owned value is never mutated or stored past the borrow lifetime.",
        markers: &["Cow::to_mut", "to_owned", "into_owned", "owned", "mutation"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_option_clone_then_unwrap_or",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag cloning `Option<T>` or `Result<T, E>` only to unwrap/default instead of borrowing with `as_ref`, `as_deref`, or `map`.",
        markers: &[
            "Option<T>",
            "Result<T, E>",
            "as_ref",
            "as_deref",
            "map",
            "option",
            "clone",
            "unwrap",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_large_enum_variant_without_boxing",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag enums with one much larger variant causing every enum value to carry the largest layout.",
        markers: &["large", "enum", "variant", "boxing"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: true,
    },
    RuleSpec {
        id: "rust_boxed_trait_object_in_inner_loop",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag repeated `Box<dyn Trait>` allocation inside loops where generics, enum dispatch, or object reuse may be better.",
        markers: &["Box<dyn Trait>", "boxed", "trait", "object", "inner"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_iterator_chain_allocates_intermediate_strings",
        section: "General Memory, Allocation, and Collection Rules",
        description: "flag iterator chains that map every item through `format!`, `to_string`, or JSON conversion before a simple predicate or grouping.",
        markers: &[
            "format!",
            "to_string",
            "iterator",
            "chain",
            "allocates",
            "intermediate",
            "strings",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_read_to_string_for_line_scan",
        section: "General I/O, Path, and Resource Rules",
        description: "flag whole-file `read_to_string` or `fs::read_to_string` followed only by line scanning or predicate checks.",
        markers: &[
            "read_to_string",
            "fs::read_to_string",
            "read",
            "String::new()",
            "push_str(",
            "to_string(",
            "line",
            "scan",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_file_open_in_loop_without_buffered_reader",
        section: "General I/O, Path, and Resource Rules",
        description: "flag opening and reading files inside loops without `BufReader`, batching, or reuse.",
        markers: &["BufReader", "file", "open", "buffered", "reader"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_flush_inside_write_loop",
        section: "General I/O, Path, and Resource Rules",
        description: "flag `flush()` inside per-item write loops unless the code is interactive terminal output.",
        markers: &["flush()", "flush", "write"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_create_dir_all_per_file",
        section: "General I/O, Path, and Resource Rules",
        description: "flag `create_dir_all` inside loops for the same parent directory.",
        markers: &["create_dir_all", "create", "file"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_metadata_called_repeatedly_same_path",
        section: "General I/O, Path, and Resource Rules",
        description: "flag repeated `metadata`, `symlink_metadata`, `exists`, or `is_file` checks for the same path in one function.",
        markers: &[
            "metadata",
            "symlink_metadata",
            "exists",
            "is_file",
            "repeatedly",
            "Path::",
            "canonicalize(",
            "to_string_lossy(",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_path_to_string_lossy_in_hot_loop",
        section: "General I/O, Path, and Resource Rules",
        description: "flag `to_string_lossy`, `display().to_string`, or path formatting inside repository/file traversal loops.",
        markers: &[
            "to_string_lossy",
            "display().to_string",
            "Path::",
            "canonicalize(",
            "to_string_lossy(",
            "String::new()",
            "push_str(",
            "to_string(",
        ],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_stdout_println_in_library_or_hot_path",
        section: "General I/O, Path, and Resource Rules",
        description: "flag `println`, `eprintln`, or direct stdout/stderr writes inside library code, scan loops, or reusable components.",
        markers: &[
            "println",
            "eprintln",
            "stdout",
            "library",
            "Path::",
            "canonicalize(",
            "to_string_lossy(",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_read_dir_collect_sort_before_filter",
        section: "General I/O, Path, and Resource Rules",
        description: "flag `read_dir` entries collected and sorted before filtering by type/extension.",
        markers: &["read_dir", "read", "collect::<Vec<_>>()", "sort", "filter"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_file_handle_returned_without_close_owner_contract",
        section: "General I/O, Path, and Resource Rules",
        description: "flag functions that return raw file handles, descriptors, or paths tied to temporary resources without documenting ownership.",
        markers: &["file", "handle", "returned", "close", "owner", "contract"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_blocking_process_output_read_unbounded",
        section: "General I/O, Path, and Resource Rules",
        description: "flag `Command::output()` or piped process reads where stdout/stderr may be large and no size bound exists.",
        markers: &["Command::output()", "blocking", "process", "output", "read"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_path_canonicalize_in_scan_inner_loop",
        section: "General I/O, Path, and Resource Rules",
        description: "flag repeated canonicalization for every path in a hot repository traversal when relative normalized paths would be enough.",
        markers: &[
            "Path::",
            "canonicalize(",
            "to_string_lossy(",
            "canonicalize",
            "scan",
            "inner",
        ],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_os_string_lossy_conversion_before_filter",
        section: "General I/O, Path, and Resource Rules",
        description: "flag lossy path/string conversions before simple extension, file-name, or component filters.",
        markers: &[
            "String::new()",
            "push_str(",
            "to_string(",
            "lossy",
            "conversion",
            "filter",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_mutex_guard_lives_through_cpu_heavy_work",
        section: "General Concurrency and Synchronization Rules",
        description: "flag lock guards that remain live across sorting, parsing, serialization, filesystem I/O, or large loops.",
        markers: &[
            "Mutex",
            "lock().unwrap ()",
            "RwLock",
            "guard",
            "lives",
            "through",
            "heavy",
            "work",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_rwlock_write_guard_for_readonly_access",
        section: "General Concurrency and Synchronization Rules",
        description: "flag write locks where the guarded value is only read.",
        markers: &["RwLock", "write", "guard", "readonly", "access"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_mutex_lock_unwrap_panics_on_poison",
        section: "General Concurrency and Synchronization Rules",
        description: "flag `lock().unwrap ()` or `write().unwrap ()` in production code where poison recovery or contextual error reporting would be safer.",
        markers: &[
            "lock().unwrap ()",
            "write().unwrap ()",
            "Mutex",
            "RwLock",
            "lock",
            "unwrap",
            "panics",
            "poison",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_atomic_seqcst_without_comment",
        section: "General Concurrency and Synchronization Rules",
        description: "flag `Ordering::SeqCst` in non-trivial code without a nearby comment explaining the synchronization requirement.",
        markers: &["Ordering::SeqCst", "atomic", "SeqCst", "comment"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_unbounded_channel_in_producer_loop",
        section: "General Concurrency and Synchronization Rules",
        description: "flag unbounded channel sends inside loops or request paths without backpressure or shutdown policy.",
        markers: &["channel", "producer"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_thread_spawn_in_loop_without_join_limit",
        section: "General Concurrency and Synchronization Rules",
        description: "flag `std::thread::spawn` in loops without a join handle collection limit, pool, or semaphore.",
        markers: &["std::thread::spawn", "JoinHandle", "spawn", "join", "limit"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_arc_clone_inside_inner_loop",
        section: "General Concurrency and Synchronization Rules",
        description: "flag repeated `Arc::clone` in inner loops when a borrowed reference or cloned handle outside the loop would work.",
        markers: &["Arc::clone", "clone", "inner"],
        import_gate: &[],
        loop_only: true,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_mpsc_receiver_iter_without_shutdown_signal",
        section: "General Concurrency and Synchronization Rules",
        description: "flag blocking receiver iteration without timeout, close path, cancellation, or sentinel.",
        markers: &["mpsc", "receiver", "iter", "shutdown", "signal"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_condvar_wait_without_predicate_loop",
        section: "General Concurrency and Synchronization Rules",
        description: "flag `Condvar::wait` not wrapped in a predicate loop.",
        markers: &["Condvar::wait", "condvar", "wait", "predicate"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_sleep_polling_loop",
        section: "General Concurrency and Synchronization Rules",
        description: "flag `thread::sleep` or runtime sleep in polling loops without backoff, notification, or timeout ownership.",
        markers: &["thread::sleep", "sleep", "polling"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_join_handle_dropped_after_spawn",
        section: "General Concurrency and Synchronization Rules",
        description: "flag spawned threads whose join handles are immediately dropped outside explicit detached-worker patterns.",
        markers: &["join", "handle", "dropped", "spawn"],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_once_lock_initializes_fallible_resource_with_unwrap",
        section: "General Concurrency and Synchronization Rules",
        description: "flag `OnceLock`, `LazyLock`, or lazy initialization that unwraps fallible setup instead of returning initialization errors at bootstrap.",
        markers: &[
            "OnceLock",
            "LazyLock",
            "once",
            "lock",
            "initializes",
            "fallible",
            "resource",
            "unwrap",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: false,
        file_only: false,
    },
    RuleSpec {
        id: "rust_manifest_wildcard_dependency_version",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag `*` dependency versions or unconstrained git/path dependencies outside local workspace development.",
        markers: &[
            "[dependencies]",
            "[workspace]",
            "[profile.release]",
            "wildcard",
            "dependency",
            "version",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
    RuleSpec {
        id: "rust_manifest_dependency_default_features_unreviewed",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag heavy dependencies with default features enabled when only a narrow feature set appears to be used.",
        markers: &[
            "[dependencies]",
            "[workspace]",
            "[profile.release]",
            "dependency",
            "default",
            "features",
            "unreviewed",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
    RuleSpec {
        id: "rust_manifest_duplicate_direct_dependency_versions",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag the same crate required at multiple direct versions across workspace members.",
        markers: &[
            "[dependencies]",
            "[workspace]",
            "[profile.release]",
            "duplicate",
            "direct",
            "dependency",
            "versions",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
    RuleSpec {
        id: "rust_manifest_dev_dependency_used_in_src",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag dev-only crates such as `tempfile`, `proptest`, or fuzz helpers imported from production `src` code.",
        markers: &[
            "tempfile",
            "proptest",
            "src",
            "[dependencies]",
            "[workspace]",
            "[profile.release]",
            "dependency",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
    RuleSpec {
        id: "rust_manifest_build_dependency_used_at_runtime",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag build-dependencies imported from runtime code or runtime dependencies used only by `build.rs`.",
        markers: &[
            "build.rs",
            "[dependencies]",
            "[workspace]",
            "[profile.release]",
            "rerun-if-changed",
            "Command::new(",
            "dependency",
            "runtime",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
    RuleSpec {
        id: "rust_manifest_workspace_dependency_not_centralized",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag multi-crate workspaces that repeat dependency versions instead of using `[workspace.dependencies]`.",
        markers: &[
            "[workspace.dependencies]",
            "[dependencies]",
            "[workspace]",
            "[profile.release]",
            "workspace",
            "dependency",
            "centralized",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
    RuleSpec {
        id: "rust_manifest_release_lto_missing_for_cli_binary",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag CLI/binary crates with release profiles that omit any LTO setting when binary size or startup matters.",
        markers: &[
            "[dependencies]",
            "[workspace]",
            "[profile.release]",
            "release",
            "binary",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
    RuleSpec {
        id: "rust_manifest_bench_or_fuzz_target_in_default_members",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag fuzz/bench crates included in default workspace members without an explicit opt-in.",
        markers: &[
            "[dependencies]",
            "[workspace]",
            "[profile.release]",
            "bench",
            "fuzz",
            "target",
            "default",
            "members",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
    RuleSpec {
        id: "rust_build_script_missing_rerun_if_changed",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag `build.rs` files that read files, env vars, or external commands without `cargo:rerun-if-changed` or `cargo:rerun-if-env-changed`.",
        markers: &[
            "build.rs",
            "cargo:rerun-if-changed",
            "cargo:rerun-if-env-changed",
            "rerun-if-changed",
            "Command::new(",
            "script",
            "rerun",
            "changed",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
    RuleSpec {
        id: "rust_build_script_network_or_git_call",
        section: "Cargo, Feature, Build, and Packaging Rules",
        description: "flag build scripts that invoke network, git, curl, package managers, or shell commands that make builds non-hermetic.",
        markers: &[
            "build.rs",
            "rerun-if-changed",
            "Command::new(",
            "script",
            "network",
        ],
        import_gate: &[],
        loop_only: false,
        test_only: false,
        manifest_only: true,
        file_only: false,
    },
];

#[derive(Debug, Clone)]
struct BodyLine {
    line: usize,
    text: String,
    in_loop: bool,
}

pub(crate) fn bad_practices_function_findings(
    file: &ParsedFile,
    function: &ParsedFunction,
) -> Vec<Finding> {
    let lines = body_lines(function);
    let body = function.body_text.as_str();
    let body_lower = body.to_ascii_lowercase();
    let mut findings = Vec::new();

    for spec in RULE_SPECS {
        if spec.manifest_only || spec.file_only {
            continue;
        }
        if spec.test_only && !is_test_like_context(file, function) {
            continue;
        }
        if !spec.import_gate.is_empty() && !has_any_import_gate(file, spec.import_gate) {
            continue;
        }
        if !rule_body_match(spec, &body_lower) {
            continue;
        }

        let Some((line, marker)) = first_match_line(spec, &lines, &body_lower) else {
            continue;
        };

        findings.push(Finding {
            rule_id: spec.id.to_string(),
            severity: rule_severity(spec.id),
            path: file.path.clone(),
            function_name: Some(function.fingerprint.name.clone()),
            start_line: line,
            end_line: line,
            message: format!(
                "{} in function {}",
                spec.description, function.fingerprint.name
            ),
            evidence: vec![
                format!("section={}", spec.section),
                format!("matched_marker={}", marker),
            ],
        });
    }

    findings
}

pub(crate) fn bad_practices_file_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    // File-structure checks that do not require function-body evidence.
    findings.extend(serde_wire_enum_findings(file));
    findings.extend(toml_unknown_field_findings(file));
    findings.extend(clap_secret_debug_findings(file));
    findings.extend(large_enum_layout_findings(file));
    findings.extend(build_dependency_runtime_usage_findings(file));

    findings
}

pub(crate) fn bad_practices_indexed_file_findings(
    file: &ParsedFile,
    index: &RepositoryIndex,
) -> Vec<Finding> {
    if !should_emit_manifest_findings(file, index.root()) {
        return Vec::new();
    }

    let root_manifest = index.root().join("Cargo.toml");
    let Ok(root_source) = read_to_string_limited(&root_manifest, DEFAULT_MAX_BYTES) else {
        return Vec::new();
    };
    let Ok(root_toml) = root_source.parse::<Value>() else {
        return Vec::new();
    };

    let mut findings = Vec::new();
    findings.extend(manifest_wildcard_dependency_findings(
        &root_manifest,
        &root_toml,
    ));
    findings.extend(manifest_default_features_findings(
        &root_manifest,
        &root_toml,
    ));
    findings.extend(manifest_workspace_dependency_centralization_findings(
        &root_manifest,
        &root_toml,
    ));
    findings.extend(manifest_release_lto_findings(
        index.root(),
        &root_manifest,
        &root_toml,
    ));
    findings.extend(manifest_bench_fuzz_default_members_findings(
        &root_manifest,
        &root_toml,
    ));
    findings.extend(manifest_duplicate_version_findings(
        index.root(),
        &root_manifest,
        &root_toml,
    ));
    findings.extend(build_script_rerun_findings(index.root()));
    findings.extend(build_script_network_findings(index.root()));

    findings
}

fn should_emit_manifest_findings(file: &ParsedFile, root: &Path) -> bool {
    let candidates = [
        root.join("src/lib.rs"),
        root.join("src/main.rs"),
        root.join("lib.rs"),
        root.join("main.rs"),
    ];
    candidates.iter().any(|candidate| candidate == &file.path)
}

fn is_test_like_context(file: &ParsedFile, function: &ParsedFunction) -> bool {
    if function.is_test_function || file.is_test_file {
        return true;
    }
    let path = file.path.to_string_lossy().to_ascii_lowercase();
    path.contains("/tests/") || path.contains("/fuzz/") || path.ends_with("_test.rs")
}

fn has_any_import_gate(file: &ParsedFile, gates: &[&str]) -> bool {
    file.imports.iter().any(|import| {
        let path = import.path.to_ascii_lowercase();
        let alias = import.alias.to_ascii_lowercase();
        gates.iter().any(|gate| {
            let gate = gate.to_ascii_lowercase();
            path.contains(&gate) || alias.contains(&gate)
        })
    })
}

fn rule_body_match(spec: &RuleSpec, body_lower: &str) -> bool {
    if spec.markers.is_empty() {
        return false;
    }
    spec.markers
        .iter()
        .filter(|marker| marker_is_actionable(marker))
        .any(|marker| body_lower.contains(&marker.to_ascii_lowercase()))
}

fn first_match_line(
    spec: &RuleSpec,
    lines: &[BodyLine],
    body_lower: &str,
) -> Option<(usize, String)> {
    for marker in spec.markers {
        if !marker_is_actionable(marker) {
            continue;
        }
        let marker_lower = marker.to_ascii_lowercase();
        if !body_lower.contains(&marker_lower) {
            continue;
        }
        if let Some(line) = lines.iter().find(|line| {
            (!spec.loop_only || line.in_loop)
                && line.text.to_ascii_lowercase().contains(&marker_lower)
        }) {
            return Some((line.line, marker.to_string()));
        }
    }

    None
}

fn marker_is_actionable(marker: &str) -> bool {
    marker.contains("::")
        || marker.contains('(')
        || marker.contains(')')
        || marker.contains('.')
        || marker.contains('_')
        || marker.chars().any(|ch| ch.is_ascii_uppercase())
}

fn rule_severity(rule_id: &str) -> Severity {
    let lower = rule_id.to_ascii_lowercase();
    if [
        "unwrap",
        "panic",
        "unsafe",
        "secret",
        "without_cleanup",
        "without_policy",
        "without_validation",
        "without_reclaim",
        "not_checked",
        "poison",
        "ffi",
        "libc",
        "error_logged_and_returned",
        "question_mark_after_partial_side_effect_without_cleanup",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        Severity::Warning
    } else {
        Severity::Info
    }
}

fn body_lines(function: &ParsedFunction) -> Vec<BodyLine> {
    let mut brace_depth = 0usize;
    let mut loop_exit_depths = Vec::new();
    let mut lines = Vec::new();

    for (offset, raw_line) in function.body_text.lines().enumerate() {
        let absolute_line = function.body_start_line + offset;
        let stripped = raw_line.split("//").next().unwrap_or("").trim().to_string();
        let closing_braces = stripped.chars().filter(|ch| *ch == '}').count();
        for _ in 0..closing_braces {
            brace_depth = brace_depth.saturating_sub(1);
            while loop_exit_depths
                .last()
                .is_some_and(|exit_depth| *exit_depth > brace_depth)
            {
                loop_exit_depths.pop();
            }
        }

        let starts_loop = contains_keyword(&stripped, "for")
            || contains_keyword(&stripped, "while")
            || contains_keyword(&stripped, "loop");
        let in_loop = !loop_exit_depths.is_empty() || starts_loop;
        let opening_braces = stripped.chars().filter(|ch| *ch == '{').count();
        if starts_loop {
            loop_exit_depths.push(brace_depth + opening_braces.max(1));
        }
        brace_depth += opening_braces;

        lines.push(BodyLine {
            line: absolute_line,
            text: stripped,
            in_loop,
        });
    }

    lines
}

fn contains_keyword(line: &str, keyword: &str) -> bool {
    let bytes = line.as_bytes();
    let keyword_bytes = keyword.as_bytes();
    if keyword_bytes.is_empty() || bytes.len() < keyword_bytes.len() {
        return false;
    }

    for start in 0..=bytes.len() - keyword_bytes.len() {
        if &bytes[start..start + keyword_bytes.len()] != keyword_bytes {
            continue;
        }
        let left_ok =
            start == 0 || (!bytes[start - 1].is_ascii_alphanumeric() && bytes[start - 1] != b'_');
        let right_index = start + keyword_bytes.len();
        let right_ok = right_index == bytes.len()
            || (!bytes[right_index].is_ascii_alphanumeric() && bytes[right_index] != b'_');
        if left_ok && right_ok {
            return true;
        }
    }

    false
}

fn first_manifest_line(source: &str, marker: &str) -> usize {
    source
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(marker))
        .map_or(1, |(offset, _)| offset + 1)
}

fn manifest_wildcard_dependency_findings(manifest_path: &Path, manifest: &Value) -> Vec<Finding> {
    let mut findings = Vec::new();
    for table_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(table) = manifest.get(table_name).and_then(Value::as_table) else {
            continue;
        };
        for (name, value) in table {
            let wildcard = value.as_str() == Some("*")
                || value.get("version").and_then(Value::as_str) == Some("*");
            let unconstrained_git_or_path =
                value.get("git").is_some() || value.get("path").is_some();
            if wildcard || unconstrained_git_or_path {
                findings.push(Finding {
                    rule_id: "rust_manifest_wildcard_dependency_version".to_string(),
                    severity: Severity::Warning,
                    path: manifest_path.to_path_buf(),
                    function_name: None,
                    start_line: 1,
                    end_line: 1,
                    message: format!(
                        "dependency {} in {} is wildcard or unconstrained",
                        name, table_name
                    ),
                    evidence: vec![format!("table={}", table_name)],
                });
            }
        }
    }
    findings
}

fn manifest_default_features_findings(manifest_path: &Path, manifest: &Value) -> Vec<Finding> {
    let Some(deps) = manifest.get("dependencies").and_then(Value::as_table) else {
        return Vec::new();
    };

    let heavyweight = ["clap", "serde_json", "toml", "rayon", "tree-sitter"];
    let mut findings = Vec::new();
    for name in heavyweight {
        let Some(value) = deps.get(name) else {
            continue;
        };
        let default_features_enabled = if value.is_str() {
            true
        } else {
            value
                .get("default-features")
                .and_then(Value::as_bool)
                .unwrap_or(true)
        };

        if default_features_enabled {
            findings.push(Finding {
                rule_id: "rust_manifest_dependency_default_features_unreviewed".to_string(),
                severity: Severity::Info,
                path: manifest_path.to_path_buf(),
                function_name: None,
                start_line: 1,
                end_line: 1,
                message: format!("dependency {} enables default features", name),
                evidence: vec!["consider narrowing enabled features when possible".to_string()],
            });
        }
    }

    findings
}

fn manifest_workspace_dependency_centralization_findings(
    manifest_path: &Path,
    manifest: &Value,
) -> Vec<Finding> {
    let Some(workspace) = manifest.get("workspace").and_then(Value::as_table) else {
        return Vec::new();
    };
    let member_count = workspace
        .get("members")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    if member_count > 1 && workspace.get("dependencies").is_none() {
        return vec![Finding {
            rule_id: "rust_manifest_workspace_dependency_not_centralized".to_string(),
            severity: Severity::Info,
            path: manifest_path.to_path_buf(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "workspace does not define [workspace.dependencies]".to_string(),
            evidence: vec![format!("workspace_members={}", member_count)],
        }];
    }
    Vec::new()
}

fn manifest_release_lto_findings(
    root: &Path,
    manifest_path: &Path,
    manifest: &Value,
) -> Vec<Finding> {
    if !root.join("src/main.rs").exists() {
        return Vec::new();
    }
    let missing_lto = manifest
        .get("profile")
        .and_then(|profile| profile.get("release"))
        .and_then(Value::as_table)
        .is_none_or(|release| release.get("lto").is_none());

    if missing_lto {
        return vec![Finding {
            rule_id: "rust_manifest_release_lto_missing_for_cli_binary".to_string(),
            severity: Severity::Info,
            path: manifest_path.to_path_buf(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "release profile omits explicit lto setting for CLI binary".to_string(),
            evidence: vec![
                "add [profile.release].lto when startup or binary size matters".to_string(),
            ],
        }];
    }

    Vec::new()
}

fn manifest_bench_fuzz_default_members_findings(
    manifest_path: &Path,
    manifest: &Value,
) -> Vec<Finding> {
    let Some(workspace) = manifest.get("workspace").and_then(Value::as_table) else {
        return Vec::new();
    };
    let Some(default_members) = workspace.get("default-members").and_then(Value::as_array) else {
        return Vec::new();
    };

    let includes_bench_or_fuzz = default_members.iter().any(|member| {
        member
            .as_str()
            .is_some_and(|member| member.contains("bench") || member.contains("fuzz"))
    });

    if includes_bench_or_fuzz {
        return vec![Finding {
            rule_id: "rust_manifest_bench_or_fuzz_target_in_default_members".to_string(),
            severity: Severity::Info,
            path: manifest_path.to_path_buf(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: "workspace default-members include bench or fuzz targets".to_string(),
            evidence: vec!["consider opt-in members for heavier targets".to_string()],
        }];
    }

    Vec::new()
}

fn member_manifests(root: &Path, manifest: &Value) -> Vec<PathBuf> {
    let mut manifests = vec![root.join("Cargo.toml")];
    if let Some(workspace) = manifest.get("workspace").and_then(Value::as_table)
        && let Some(members) = workspace.get("members").and_then(Value::as_array)
    {
        for member in members {
            if let Some(member) = member.as_str() {
                manifests.push(root.join(member).join("Cargo.toml"));
            }
        }
    }
    manifests
}

fn manifest_duplicate_version_findings(
    root: &Path,
    manifest_path: &Path,
    manifest: &Value,
) -> Vec<Finding> {
    let manifests = member_manifests(root, manifest);
    let mut versions_by_dep: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for path in manifests {
        let Ok(source) = read_to_string_limited(&path, DEFAULT_MAX_BYTES) else {
            continue;
        };
        let Ok(parsed) = source.parse::<Value>() else {
            continue;
        };
        for table_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let Some(table) = parsed.get(table_name).and_then(Value::as_table) else {
                continue;
            };
            for (name, value) in table {
                let version = value
                    .as_str()
                    .or_else(|| value.get("version").and_then(Value::as_str))
                    .unwrap_or("(unversioned)")
                    .to_string();
                versions_by_dep
                    .entry(name.clone())
                    .or_default()
                    .insert(version);
            }
        }
    }

    if let Some((dep, versions)) = versions_by_dep
        .into_iter()
        .find(|(_, versions)| versions.len() > 1)
    {
        return vec![Finding {
            rule_id: "rust_manifest_duplicate_direct_dependency_versions".to_string(),
            severity: Severity::Info,
            path: manifest_path.to_path_buf(),
            function_name: None,
            start_line: 1,
            end_line: 1,
            message: format!("dependency {} appears with multiple direct versions", dep),
            evidence: vec![format!("versions={:?}", versions)],
        }];
    }

    Vec::new()
}

fn build_script_rerun_findings(root: &Path) -> Vec<Finding> {
    let path = root.join("build.rs");
    let Ok(source) = read_to_string_limited(&path, DEFAULT_MAX_BYTES) else {
        return Vec::new();
    };

    if source.contains("cargo:rerun-if-changed") || source.contains("cargo:rerun-if-env-changed") {
        return Vec::new();
    }

    vec![Finding {
        rule_id: "rust_build_script_missing_rerun_if_changed".to_string(),
        severity: Severity::Info,
        path,
        function_name: None,
        start_line: 1,
        end_line: 1,
        message: "build script does not declare rerun-if-changed/env-changed hints".to_string(),
        evidence: vec![
            "add cargo:rerun-if-changed or cargo:rerun-if-env-changed for hermetic rebuilds"
                .to_string(),
        ],
    }]
}

fn build_script_network_findings(root: &Path) -> Vec<Finding> {
    let path = root.join("build.rs");
    let Ok(source) = read_to_string_limited(&path, DEFAULT_MAX_BYTES) else {
        return Vec::new();
    };

    let risky = [
        "Command::new(\"git\")",
        "Command::new(\"curl\")",
        "http://",
        "https://",
        "reqwest",
        "cargo install",
    ];
    let Some(marker) = risky.iter().find(|marker| source.contains(*marker)) else {
        return Vec::new();
    };

    vec![Finding {
        rule_id: "rust_build_script_network_or_git_call".to_string(),
        severity: Severity::Warning,
        path,
        function_name: None,
        start_line: first_manifest_line(&source, marker),
        end_line: first_manifest_line(&source, marker),
        message: "build script appears to run network/git/package-manager operations".to_string(),
        evidence: vec![format!("matched_marker={}", marker)],
    }]
}

fn serde_wire_enum_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    for enum_summary in file.rust_enums() {
        if !enum_summary.visibility_pub {
            continue;
        }
        if !(enum_summary.has_serialize_derive || enum_summary.has_deserialize_derive) {
            continue;
        }
        let has_stable_rename = enum_summary
            .attributes
            .iter()
            .any(|attr| attr.contains("rename_all") || attr.contains("rename ="));
        if has_stable_rename {
            continue;
        }

        findings.push(Finding {
            rule_id: "rust_serde_wire_enum_missing_stable_rename_policy".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: enum_summary.line,
            end_line: enum_summary.line,
            message: format!(
                "public wire enum {} derives serde without explicit rename policy",
                enum_summary.name
            ),
            evidence: vec![
                "consider #[serde(rename_all = ...)] for stable wire contracts".to_string(),
            ],
        });
    }

    findings
}

fn toml_unknown_field_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    for summary in file.structs() {
        if !summary.has_deserialize_derive {
            continue;
        }
        let name = summary.name.to_ascii_lowercase();
        if !(name.contains("config") || name.contains("settings") || name.contains("options")) {
            continue;
        }
        let strict = summary
            .attributes
            .iter()
            .any(|attr| attr.contains("deny_unknown_fields"));
        if strict {
            continue;
        }
        findings.push(Finding {
            rule_id: "rust_toml_config_without_unknown_field_rejection".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: None,
            start_line: summary.line,
            end_line: summary.line,
            message: format!(
                "config-like struct {} deserializes without deny_unknown_fields",
                summary.name
            ),
            evidence: vec![
                "add #[serde(deny_unknown_fields)] for strict TOML boundaries".to_string(),
            ],
        });
    }
    findings
}

fn clap_secret_debug_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    for summary in file.structs() {
        if !summary.has_debug_derive {
            continue;
        }
        let looks_like_cli = summary.name.to_ascii_lowercase().contains("cli")
            || summary.name.to_ascii_lowercase().contains("args")
            || summary
                .attributes
                .iter()
                .any(|attr| attr.contains("derive(") && attr.contains("Parser"));
        if !looks_like_cli {
            continue;
        }
        let has_secret_field = summary.fields.iter().any(|field| {
            let name = field.name.to_ascii_lowercase();
            ["secret", "token", "password", "key", "auth", "cookie"]
                .iter()
                .any(|needle| name.contains(needle))
        });
        if !has_secret_field {
            continue;
        }
        findings.push(Finding {
            rule_id: "rust_clap_secret_arg_derive_debug".to_string(),
            severity: Severity::Warning,
            path: file.path.clone(),
            function_name: None,
            start_line: summary.line,
            end_line: summary.line,
            message: format!(
                "CLI struct {} derives Debug while carrying secret-like fields",
                summary.name
            ),
            evidence: vec!["redact or avoid Debug for secret-bearing CLI structs".to_string()],
        });
    }
    findings
}

fn large_enum_layout_findings(file: &ParsedFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    for enum_summary in file.rust_enums() {
        if enum_summary.variant_count < 10 {
            continue;
        }
        findings.push(Finding {
            rule_id: "rust_large_enum_variant_without_boxing".to_string(),
            severity: Severity::Info,
            path: file.path.clone(),
            function_name: None,
            start_line: enum_summary.line,
            end_line: enum_summary.line,
            message: format!(
                "enum {} has many variants and may hide large layout skew",
                enum_summary.name
            ),
            evidence: vec![format!("variant_count={}", enum_summary.variant_count)],
        });
    }
    findings
}

fn build_dependency_runtime_usage_findings(file: &ParsedFile) -> Vec<Finding> {
    let path_lower = file.path.to_string_lossy().to_ascii_lowercase();
    if !path_lower.contains("/src/") {
        return Vec::new();
    }

    let build_only = ["cc", "bindgen", "cmake"];
    let Some(import) = file.imports.iter().find(|import| {
        let path = import.path.to_ascii_lowercase();
        build_only.iter().any(|name| path.contains(name))
    }) else {
        return Vec::new();
    };

    vec![Finding {
        rule_id: "rust_manifest_build_dependency_used_at_runtime".to_string(),
        severity: Severity::Warning,
        path: file.path.clone(),
        function_name: None,
        start_line: import.line,
        end_line: import.line,
        message: "runtime source imports crates that are commonly build-only dependencies"
            .to_string(),
        evidence: vec![format!("import_path={}", import.path)],
    }]
}
