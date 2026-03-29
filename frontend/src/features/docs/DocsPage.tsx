import { useState } from 'react'

import { currentRelease } from '../../content/site-content'

// ─── Types ────────────────────────────────────────────────────────────────────

type Language = 'go' | 'python' | 'rust'
type SectionId =
  | 'overview'
  | 'detection-rules'
  | 'cli-commands'
  | 'pipeline'
  | 'limitations'
  | 'about'

interface NavSection {
  id: SectionId
  label: string
  icon: string
}

interface Rule {
  id: string
  description: string
}

interface CliCommand {
  cmd: string
  desc: string
}

// ─── Static Data ──────────────────────────────────────────────────────────────

const languages: { id: Language; label: string }[] = [
  { id: 'go', label: 'Go' },
  { id: 'python', label: 'Python' },
  { id: 'rust', label: 'Rust' },
]

const sections: NavSection[] = [
  { id: 'overview', label: 'Overview', icon: '◈' },
  { id: 'detection-rules', label: 'Detection Rules', icon: '⊹' },
  { id: 'cli-commands', label: 'CLI Commands', icon: '❯' },
  { id: 'pipeline', label: 'Pipeline', icon: '◎' },
  { id: 'limitations', label: 'Limitations', icon: '△' },
  { id: 'about', label: 'About', icon: '♡' },
]

// ─── Content Data ────────────────────────────────────────────────────────────

const goRules: Rule[] = [
  { id: 'generic_name', description: 'Function names that are overly generic without stronger contextual signals.' },
  { id: 'overlong_name', description: 'Very long identifiers with too many descriptive tokens.' },
  { id: 'weak_typing', description: 'Signatures that rely on any or interface{}.' },
  { id: 'dropped_error', description: 'Blank identifier assignments that discard an err-like value.' },
  { id: 'panic_on_error', description: 'err != nil branches that jump straight to panic or log.Fatal style exits.' },
  { id: 'error_wrapping_misuse', description: 'fmt.Errorf calls that reference err without %w.' },
  { id: 'missing_context', description: 'Standard-library context-aware calls from functions that do not accept context.Context.' },
  { id: 'missing_context_propagation', description: 'Functions that already accept context.Context but still call context-free stdlib APIs like http.Get or exec.Command.' },
  { id: 'context_background_used', description: 'Functions that already accept context.Context but still create context.Background() or context.TODO() locally.' },
  { id: 'missing_cancel_call', description: 'Derived contexts where deslop cannot find a local cancel() or defer cancel() call.' },
  { id: 'sleep_polling', description: 'time.Sleep inside loops — often indicates polling or busy-wait style code.' },
  { id: 'busy_waiting', description: 'select { default: ... } inside loops, which often spins instead of blocking.' },
  { id: 'goroutine_without_coordination', description: 'Raw go statements without an obvious context or WaitGroup-like coordination signal.' },
  { id: 'goroutine_spawn_in_loop', description: 'Raw go statements launched from inside loops without obvious WaitGroup coordination.' },
  { id: 'goroutine_without_shutdown_path', description: 'Looping goroutine literals without an obvious ctx.Done() or done-channel shutdown path.' },
  { id: 'goroutine_derived_context_unmanaged', description: 'Likely long-lived goroutines launched after a derived context is created and before the matching cancel call is observed.' },
  { id: 'mutex_in_loop', description: 'Repeated Lock or RLock acquisition inside loops.' },
  { id: 'blocking_call_while_locked', description: 'Potentially blocking calls observed between Lock and Unlock.' },
  { id: 'string_concat_in_loop', description: 'Repeated string concatenation inside loops when the function is clearly building a string incrementally.' },
  { id: 'likely_n_squared_string_concat', description: 'Opt-in deeper semantic signal for repeated string concatenation inside nested loops without obvious builder usage.' },
  { id: 'repeated_json_marshaling', description: 'encoding/json.Marshal or MarshalIndent inside loops — repeated allocation and serialization hot spots.' },
  { id: 'allocation_churn_in_loop', description: 'Obvious make, new, or buffer-construction calls inside loops.' },
  { id: 'likely_n_squared_allocation', description: 'Opt-in deeper semantic signal for allocations that also sit inside nested loop structure.' },
  { id: 'fmt_hot_path', description: 'fmt formatting calls such as Sprintf inside loops.' },
  { id: 'reflection_hot_path', description: 'reflect package calls inside loops.' },
  { id: 'full_dataset_load', description: 'Calls such as io.ReadAll or os.ReadFile that load an entire payload into memory instead of streaming.' },
  { id: 'n_plus_one_query', description: 'Database-style query calls issued inside loops. The opt-in semantic pack can raise severity when nested loops also appear.' },
  { id: 'wide_select_query', description: 'Literal SELECT * query shapes.' },
  { id: 'likely_unindexed_query', description: 'Query shapes like leading-wildcard LIKE or ORDER BY without LIMIT that often scale poorly.' },
  { id: 'weak_crypto', description: 'Direct use of weak standard-library crypto packages such as crypto/md5, crypto/sha1, crypto/des, and crypto/rc4.' },
  { id: 'hardcoded_secret', description: 'Secret-like identifiers assigned direct string literals instead of environment or secret-manager lookups.' },
  { id: 'sql_string_concat', description: 'Query execution calls where SQL is constructed dynamically with concatenation or fmt.Sprintf.' },
  { id: 'inconsistent_package_name', description: 'Directories that mix base Go package names after ignoring the _test suffix.' },
  { id: 'misgrouped_imports', description: 'Import blocks that place stdlib imports after third-party imports.' },
  { id: 'mixed_receiver_kinds', description: 'Methods on the same receiver type mix pointer and value receivers.' },
  { id: 'malformed_struct_tag', description: 'Struct field tags that do not parse as valid Go tag key/value pairs.' },
  { id: 'duplicate_struct_tag_key', description: 'Struct field tags that repeat the same key more than once.' },
  { id: 'hallucinated_import_call', description: 'Package-qualified calls that do not match locally indexed symbols for the imported package.' },
  { id: 'hallucinated_local_call', description: 'Same-package calls to symbols not present in the scanned local package context.' },
  { id: 'test_without_assertion_signal', description: 'Tests that call production code without any obvious assertion or failure signal.' },
  { id: 'happy_path_only_test', description: 'Tests that assert success expectations without any obvious negative-path signal.' },
  { id: 'placeholder_test_body', description: 'Tests that look skipped, TODO-shaped, or otherwise placeholder-like rather than validating behavior.' },
  { id: 'comment_style_title_case', description: 'Heading-like Title Case doc comments.' },
  { id: 'comment_style_tutorial', description: 'Tutorial-style comments that narrate obvious implementation steps.' },
]

const pythonRules: Rule[] = [
  { id: 'blocking_sync_io_in_async', description: 'Synchronous network, subprocess, sleep, or file I/O calls made from async def functions.' },
  { id: 'exception_swallowed', description: 'Broad exception handlers like except: or except Exception: that immediately suppress the error with pass, continue, break, or return.' },
  { id: 'broad_exception_handler', description: 'Broad except Exception: style handlers that still obscure failure shape even when not fully swallowed.' },
  { id: 'eval_exec_usage', description: 'Direct eval() or exec() usage in non-test Python code.' },
  { id: 'print_debugging_leftover', description: 'print() calls left in non-test Python functions that do not look like obvious main-entrypoint output.' },
  { id: 'generic_name', description: 'Generic function names that lack stronger domain-specific signals (shared signal).' },
  { id: 'overlong_name', description: 'Overly descriptive function names with too many name tokens (shared signal).' },
  { id: 'weak_typing', description: 'Weakly typed inputs or outputs, including vague any-style signatures when the parser can see them (shared signal).' },
  { id: 'none_comparison', description: '== None or != None checks instead of is None or is not None.' },
  { id: 'side_effect_comprehension', description: 'List, set, or dict comprehensions used as standalone statements where the result is discarded.' },
  { id: 'redundant_return_none', description: 'Explicit return None in simple code paths where Python would already return None implicitly.' },
  { id: 'hardcoded_path_string', description: 'Hardcoded filesystem path literals assigned inside non-test Python functions.' },
  { id: 'variadic_public_api', description: 'Public Python functions that expose *args or **kwargs instead of a clearer interface.' },
  { id: 'list_materialization_first_element', description: 'list(...)[0] style access that materializes a whole list just to read the first element.' },
  { id: 'deque_candidate_queue', description: 'Queue-style list operations like pop(0) or insert(0, ...) that may want collections.deque.' },
  { id: 'temporary_collection_in_loop', description: 'Loop-local list, dict, or set construction that likely adds avoidable allocation churn.' },
  { id: 'recursive_traversal_risk', description: 'Direct recursion in traversal-style helpers that may be safer as iterative walks for deep inputs.' },
  { id: 'list_membership_in_loop', description: 'Repeated membership checks against obviously list-like containers inside loops.' },
  { id: 'repeated_len_in_loop', description: 'Repeated len(...) checks inside loops when the receiver appears unchanged locally.' },
  { id: 'god_function', description: 'Very large Python functions with high control-flow and call-surface concentration.' },
  { id: 'god_class', description: 'Python classes that concentrate unusually high method count, public surface area, and mutable instance state.' },
  { id: 'monolithic_init_module', description: '__init__.py files that carry enough imports and behavior to look like monolithic modules.' },
  { id: 'monolithic_module', description: 'Non-__init__.py modules that are unusually large and combine many imports with orchestration-heavy behavior.' },
  { id: 'too_many_instance_attributes', description: 'Classes that assign an unusually large number of instance attributes across their methods.' },
  { id: 'eager_constructor_collaborators', description: 'Constructors that instantiate several collaborators eagerly inside __init__.' },
  { id: 'over_abstracted_wrapper', description: 'Ceremonial wrapper-style or tiny data-container classes that add little beyond storing constructor state.' },
  { id: 'mixed_concerns_function', description: 'Functions that mix HTTP, persistence, and filesystem-style concerns in one body.' },
  { id: 'name_responsibility_mismatch', description: 'Read-style, transformation-style, or utility-style names that still perform mutation or own multiple infrastructure concerns.' },
  { id: 'deep_inheritance_hierarchy', description: 'Repository-local Python class chains with unusually deep inheritance depth.' },
  { id: 'tight_module_coupling', description: 'Modules that depend on a large number of repository-local Python modules.' },
  { id: 'unrelated_heavy_import', description: 'Heavy ecosystem imports with little local evidence of real need.' },
  { id: 'public_api_missing_type_hints', description: 'Public Python functions that omit complete parameter or return annotations.' },
  { id: 'mixed_sync_async_module', description: 'Modules that expose public sync and async entry points together.' },
  { id: 'duplicate_error_handler_block', description: 'Repeated exception-handling block shapes in one file.' },
  { id: 'duplicate_validation_pipeline', description: 'Repeated validation guard pipelines across functions in one file.' },
  { id: 'cross_file_copy_paste_function', description: 'Highly similar non-test function bodies repeated across multiple Python files.' },
  { id: 'hallucinated_import_call', description: 'Package-qualified calls that do not match locally indexed symbols for the imported Python package.' },
  { id: 'hallucinated_local_call', description: 'Same-package calls to Python symbols not present in the scanned local package context.' },
  { id: 'hardcoded_business_rule', description: 'Hardcoded threshold, rate-limit, or pricing-style literals assigned inside non-test Python functions.' },
  { id: 'magic_value_branching', description: 'Repeated branch-shaping numeric or string literals that likely want an explicit constant or policy name.' },
  { id: 'reinvented_utility', description: 'Obvious locally implemented utility helpers that overlap with already-imported standard-library style helpers.' },
  { id: 'builtin_reduction_candidate', description: 'Loop shapes that look like obvious sum, any, or all candidates.' },
  { id: 'missing_context_manager', description: 'Resource management (files, network connections) inside non-test Python functions that omits with-statement context managers.' },
  { id: 'environment_boundary_without_fallback', description: 'Environment-variable lookups that omit a default value or explicit failure handler.' },
  { id: 'mixed_naming_conventions', description: 'File mixes snake_case and camelCase function naming conventions.' },
  { id: 'comment_style_title_case', description: 'Title Case function documentation that reads more like a heading than intent-focused docs (shared signal).' },
  { id: 'comment_style_tutorial', description: 'Verbose tutorial-style function documentation that narrates implementation steps (shared signal).' },
  { id: 'textbook_docstring_small_helper', description: 'Very small helper functions that have unusually long, textbook-style docstrings.' },
  { id: 'obvious_commentary', description: 'Comments that narrate obvious implementation steps instead of explaining intent.' },
  { id: 'enthusiastic_commentary', description: 'Unusually enthusiastic or emoji-heavy production comments.' },
  { id: 'commented_out_code', description: 'Blocks of commented-out source code left in production files.' },
  { id: 'repeated_string_literal', description: 'Project repeats the same long string literal multiple times in one file.' },
  { id: 'cross_file_repeated_literal', description: 'Project repeats the same long string literal across multiple files.' },
  { id: 'test_without_assertion_signal', description: 'Python tests that exercise production code without an obvious assertion or failure signal (shared signal).' },
  { id: 'happy_path_only_test', description: 'Python tests that appear to cover only success expectations and no visible negative path (shared signal).' },
  { id: 'placeholder_test_body', description: 'Python tests that look skipped, TODO-shaped, or otherwise placeholder-like rather than validating behavior (shared signal).' },
  { id: 'duplicate_test_utility_logic', description: 'Highly similar utility logic shared between test and production code.' },
  { id: 'duplicate_query_fragment', description: 'Repository repeats the same SQL-like query fragment across multiple files.' },
  { id: 'duplicate_transformation_pipeline', description: 'Repository repeats the same data transformation pipeline stages across multiple functions.' },
  { id: 'network_boundary_without_timeout', description: 'Request, sync, or job-style Python functions that call HTTP boundaries with no obvious timeout or retry policy.' },
  { id: 'external_input_without_validation', description: 'Request or CLI entry points that trust external input without obvious validation or guard checks.' },
  { id: 'hardcoded_secret', description: 'Secret-like identifiers assigned direct string literals (shared signal).' },
  { id: 'full_dataset_load', description: 'Calls that load an entire payload into memory instead of streaming it (shared signal).' },
  { id: 'string_concat_in_loop', description: 'Repeated string concatenation inside loops (shared signal).' },
]

const rustRules: Rule[] = [
  { id: 'todo_macro_leftover', description: 'todo!() left in non-test Rust code.' },
  { id: 'unimplemented_macro_leftover', description: 'unimplemented!() left in non-test Rust code.' },
  { id: 'dbg_macro_leftover', description: 'dbg!() left in non-test Rust code.' },
  { id: 'panic_macro_leftover', description: 'panic!() left in non-test Rust code.' },
  { id: 'unreachable_macro_leftover', description: 'unreachable!() left in non-test Rust code.' },
  { id: 'unwrap_in_non_test_code', description: '.unwrap() used in non-test Rust code.' },
  { id: 'expect_in_non_test_code', description: '.expect(...) used in non-test Rust code.' },
  { id: 'unsafe_without_safety_comment', description: 'unsafe fn or unsafe block without a nearby SAFETY: comment within the previous two lines.' },
  { id: 'todo_doc_comment_leftover', description: 'Rust doc comments that still contain a TODO marker in non-test code.' },
  { id: 'fixme_doc_comment_leftover', description: 'Rust doc comments that still contain a FIXME marker in non-test code.' },
  { id: 'hallucinated_import_call', description: 'Covers crate::, self::, and super:: module paths when deslop can map them back to locally indexed Rust modules, plus direct calls through locally imported function aliases.' },
  { id: 'hallucinated_local_call', description: 'Direct same-module calls when the callee name is not locally bound and does not exist in the indexed Rust module.' },
  { id: 'rust_blocking_io_in_async', description: 'Blocking I/O or blocking work observed in async Rust code.' },
  { id: 'rust_unbuffered_file_writes', description: 'File-like writes performed inside a loop without buffering or batching.' },
  { id: 'rust_lines_allocate_per_line', description: '.lines() iteration used in a loop where per-item allocation may matter.' },
  { id: 'rust_hashmap_default_hasher', description: 'HashMap default-hasher construction in a likely hot path.' },
  { id: 'rust_lock_across_await', description: 'A lock appears to be held across an .await boundary.' },
  { id: 'rust_tokio_mutex_unnecessary', description: 'tokio::sync::Mutex used in a fully synchronous critical path with no await.' },
  { id: 'rust_blocking_drop', description: 'A Drop implementation performs blocking work.' },
  { id: 'rust_pointer_chasing_vec_box', description: 'Pointer-heavy boxed vector-style storage that may hurt cache locality.' },
  { id: 'rust_path_join_absolute', description: 'Path::join used with an absolute segment that discards the existing base path.' },
  { id: 'rust_utf8_validate_hot_path', description: 'UTF-8 validation appears in a likely hot path and may deserve profiling.' },
  { id: 'rust_large_future_stack', description: 'Large allocations may be captured across await points and bloat future size.' },
  { id: 'rust_aos_hot_path', description: 'Repeated struct-field dereferences inside a loop that may indicate an array-of-structs hot path.' },
  { id: 'rust_async_std_mutex_await', description: 'std::sync::Mutex appears to be held across .await in async code.' },
  { id: 'rust_async_hold_permit_across_await', description: 'A permit or pooled resource may be held across an .await.' },
  { id: 'rust_async_spawn_cancel_at_await', description: 'Async work is spawned without an obvious cancellation path.' },
  { id: 'rust_async_missing_fuse_pin', description: 'select! reuse lacks pinning or fusing markers for repeated polling.' },
  { id: 'rust_async_recreate_future_in_select', description: 'A select! loop may recreate futures instead of reusing long-lived ones.' },
  { id: 'rust_async_monopolize_executor', description: 'An async function may monopolize the executor with blocking work and no await.' },
  { id: 'rust_async_blocking_drop', description: 'A Drop implementation does blocking work that can surface in async contexts.' },
  { id: 'rust_async_invariant_broken_at_await', description: 'Related state mutations appear split around an await boundary.' },
  { id: 'rust_async_lock_order_cycle', description: 'Conflicting lock acquisition order suggests a lock-order cycle.' },
  { id: 'rust_domain_raw_primitive', description: 'Business-facing data is stored as a raw primitive instead of a stronger domain type.' },
  { id: 'rust_domain_float_for_money', description: 'Floating-point storage is used for money-like values.' },
  { id: 'rust_domain_impossible_combination', description: 'A boolean toggle is mixed with optional credentials, creating invalid-state combinations.' },
  { id: 'rust_domain_default_produces_invalid', description: 'Default is derived or implemented on a type that likely cannot have a safe default state.' },
  { id: 'rust_debug_secret', description: 'Debug is derived on a type that carries secret-like fields.' },
  { id: 'rust_serde_sensitive_deserialize', description: 'Deserialize is derived for sensitive fields without obvious validation.' },
  { id: 'rust_serde_sensitive_serialize', description: 'Serialize is derived for secret-like fields that may need redaction or exclusion.' },
  { id: 'rust_domain_optional_secret_default', description: 'A defaultable type includes optional secret-like fields, which can hide invalid configuration.' },
  { id: 'rust_unsafe_get_unchecked', description: 'Unsafe use of get_unchecked without proof of bounds invariants.' },
  { id: 'rust_unsafe_from_raw_parts', description: 'Unsafe raw slice construction that depends on lifetime and length invariants.' },
  { id: 'rust_unsafe_set_len', description: 'Unsafe Vec::set_len use that requires initialized elements and correct capacity invariants.' },
  { id: 'rust_unsafe_assume_init', description: 'Unsafe MaybeUninit::assume_init use without proof of full initialization.' },
  { id: 'rust_unsafe_transmute', description: 'Unsafe transmute use that requires layout and validity proof.' },
  { id: 'rust_unsafe_raw_pointer_cast', description: 'Unsafe raw pointer cast that depends on aliasing and lifetime guarantees.' },
  { id: 'rust_unsafe_aliasing_assumption', description: 'Unsafe code mixes interior mutability and mutable references in ways that need careful aliasing review.' },
]

// ─── CLI commands by language ─────────────────────────────────────────────────

const cliCommands: Record<Language, CliCommand[]> = {
  go: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Scan a Go repository and print a compact finding summary.' },
    { cmd: 'cargo run -- scan --details /path/to/repo', desc: 'Include full per-function fingerprint details and detail-only findings.' },
    { cmd: 'cargo run -- scan --json /path/to/repo', desc: 'Emit structured JSON output for pipeline integration.' },
    { cmd: 'cargo run -- scan --json --details /path/to/repo', desc: 'Combine JSON and full detail output.' },
    { cmd: 'cargo run -- scan --enable-semantic /path/to/repo', desc: 'Enable the opt-in deeper semantic Go pack for nested-loop allocation, concat, and stronger N+1 correlation.' },
    { cmd: 'cargo run -- scan --ignore dropped_error,panic_on_error /path/to/repo', desc: 'Ignore selected Go rule IDs for one run without changing repository config.' },
    { cmd: 'cargo run -- scan /path/to/repo > results.txt', desc: 'Write the text report directly to a file.' },
    { cmd: 'cargo run -- scan --no-ignore /path/to/repo', desc: 'Scan without .gitignore filtering.' },
    { cmd: 'cargo run -- bench /path/to/repo', desc: 'Benchmark the full pipeline against a local repository.' },
    { cmd: 'cargo run -- bench --enable-semantic /path/to/repo', desc: 'Benchmark the Go pipeline with the opt-in semantic pack enabled.' },
    { cmd: 'cargo run -- bench --warmups 2 --repeats 5 /path/to/repo', desc: 'Benchmark with explicit warmup and repeat counts.' },
    { cmd: 'cargo run -- bench --json /path/to/repo', desc: 'Emit benchmarking data as JSON.' },
  ],
  python: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Auto-detect and scan Python files alongside any Go or Rust files in the repository.' },
    { cmd: 'cargo run -- scan --details /path/to/repo', desc: 'Include full Python per-function fingerprint breakdown.' },
    { cmd: 'cargo run -- scan --json /path/to/repo', desc: 'Emit findings for Python files in structured JSON.' },
    { cmd: 'cargo run -- scan --json --details /path/to/repo', desc: 'Combine JSON output with full Python detail-only diagnostics.' },
    { cmd: 'cargo run -- scan --ignore exception_swallowed,print_debugging_leftover /path/to/repo', desc: 'Ignore selected Python rule IDs for one run without changing repository config.' },
    { cmd: 'cargo run -- scan /path/to/repo > results.txt', desc: 'Save the Python scan report to a file for review.' },
    { cmd: 'cargo run -- scan --no-ignore /path/to/repo', desc: 'Override .gitignore filtering when scanning Python projects.' },
    { cmd: 'cargo run -- bench /path/to/repo', desc: 'Benchmark discovery, parse, index, heuristic, and total runtime stages for a Python-heavy repository.' },
    { cmd: 'cargo run -- bench --warmups 2 --repeats 5 /path/to/repo', desc: 'Benchmark Python scans with explicit warmup and repeat counts.' },
    { cmd: 'cargo run -- bench --json /path/to/repo', desc: 'Emit benchmarking data as JSON for CI or local comparisons.' },
  ],
  rust: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Auto-detect and scan Rust files in the repository using the Rust rule pack.' },
    { cmd: 'cargo run -- scan --details /path/to/repo', desc: 'Include full Rust per-function fingerprint details.' },
    { cmd: 'cargo run -- scan --json /path/to/repo', desc: 'Emit Rust findings in structured JSON.' },
    { cmd: 'cargo run -- scan --json --details /path/to/repo', desc: 'Combine JSON output with full Rust detail-only diagnostics.' },
    { cmd: 'cargo run -- scan --ignore rust_async_std_mutex_await,rust_lock_across_await /path/to/repo', desc: 'Ignore specific rule IDs for one scan invocation without changing repository config.' },
    { cmd: 'cargo run -- scan /path/to/repo > results.txt', desc: 'Save the Rust scan report to a file.' },
    { cmd: 'cargo run -- scan --no-ignore /path/to/repo', desc: 'Override .gitignore filtering when scanning Rust projects.' },
    { cmd: 'cargo run -- bench /path/to/repo', desc: 'Benchmark discovery, parse, index, heuristic, and total runtime stages.' },
    { cmd: 'cargo run -- bench --warmups 2 --repeats 5 /path/to/repo', desc: 'Benchmark with explicit warmup and repeat counts.' },
    { cmd: 'cargo run -- bench --json /path/to/repo', desc: 'Emit benchmarking data as JSON.' },
  ],
}

const githubActionWorkflow = `name: Deslop

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ${currentRelease.actionRef}
        with:
          path: .`

const githubActionJsonExample = `- uses: actions/checkout@v4
- uses: ${currentRelease.actionRef}
  with:
    path: .
    json: 'true'
    details: 'true'
    fail-on-findings: 'false'`

const githubActionBenchExample = `- uses: actions/checkout@v4
- uses: ${currentRelease.actionRef}
  with:
    command: bench
    path: .
    repeats: '10'
    warmups: '2'`

const repositoryConfigExample = `go_semantic_experimental = true
rust_async_experimental = true
disabled_rules = ["panic_macro_leftover"]
suppressed_paths = ["tests/fixtures"]

[severity_overrides]
unwrap_in_non_test_code = "error"
missing_context_propagation = "error"`

function CodeBlock({ code }: { code: string }) {
  return (
    <pre className="docs-code-block">
      <code>{code}</code>
    </pre>
  )
}

// ─── Overview content by language ─────────────────────────────────────────────

const overviewContent = {
  go: {
    title: 'Go Analysis',
    lead: 'deslop ships its broadest heuristic surface area for Go. It walks the repository with .gitignore awareness, parses source structure with tree-sitter-go, builds a local package index keyed by package plus directory, and now covers repo-wide style checks, receiver-field and nested wrapper propagation, derived-context goroutine lifetime analysis, security, performance, and an opt-in deeper semantic loop pass with explainable rules.',
    bullets: [
      '.gitignore-aware walk; skips vendor/ and generated files by default',
      'Parses package names, imports, declared symbols, call sites, and function fingerprints',
      'Builds a repository-local symbol index for same-package and import hallucination checks',
      'Includes repo-wide package and import style checks plus wrapper-level context propagation heuristics',
      'Covers receiver-field wrappers, local wrapper chains, and Query versus QueryContext mismatches',
      'Adds an opt-in semantic pack for nested-loop allocation, concat, and stronger N+1 signals',
      'Produces compact text output by default; full detail and JSON via flags',
      'Supports standalone Go repos and mixed Go + Python + Rust repositories',
    ],
  },
  python: {
    title: 'Python Analysis',
    lead: 'Python support now spans function-level, file-level, and repo-level heuristics backed by fixture-driven parser evidence. The shipped rule pack covers async boundary misuse, exception handling, duplication families, class and module structure smells, AI-style commentary signals, and repository-local hallucination checks without crossing language boundaries in mixed repos.',
    bullets: [
      'Parses .py files with tree-sitter-python alongside Go and Rust files',
      'Extracts imports, declared symbols, call sites, docstrings, class summaries, and phase-4 evidence used by maintainability and structure rules',
      'Runs 60-plus Python-specific and shared heuristics across function, file, and repository scopes',
      'Language-scoped local index prevents symbol cross-contamination in mixed repos and powers Python hallucination checks',
      'Fixture-backed parser and integration coverage keeps rule families stable as the Python surface area grows',
      'Conservative about flagging policy: favors lower false-positive rates over exhaustive coverage',
    ],
  },
  rust: {
    title: 'Rust Analysis',
    lead: 'Rust support now covers hygiene leftovers, crate-local hallucination checks, async/runtime hazards, performance smells, domain-modeling anti-patterns, and unsafe-soundness hot spots. The Rust rule pack runs on the same fast pipeline as Go and Python: tree-sitter parsing, a language-scoped local index, and explainable heuristic output.',
    bullets: [
      'Parses .rs files with tree-sitter-rust',
      'Detects leftovers, unsafe hygiene, crate-local hallucinations, async runtime hazards, and domain-modeling issues',
      'Flags unsafe blocks and functions without a nearby SAFETY: comment plus soundness-sensitive unsafe operations',
      'Covers crate::, self::, and super:: import-call hallucinations via a local Rust module index',
      'Adds Rust-specific async, performance, and domain-modeling heuristics without crossing language boundaries',
      'Language-scoped index prevents symbol merging with Go or Python in mixed repositories',
    ],
  },
}

// ─── Pipeline content ─────────────────────────────────────────────────────────

const pipelineStages = [
  {
    name: 'Discover',
    summary: 'Walk the repository with .gitignore awareness. Skip vendor/ and known generated-code paths. Keep file selection independent from later analysis.',
    detail: 'Discovery runs before any parsing so the pipeline stays composable. Supported file extensions are routed to the correct language backend. The --no-ignore flag disables .gitignore filtering when needed.',
  },
  {
    name: 'Parse',
    summary: 'Parse source structure, declared symbols, and call patterns using tree-sitter grammars without forcing a heavy semantic stack.',
    detail: 'Go files are parsed with tree-sitter-go, Python files with tree-sitter-python, and Rust files with tree-sitter-rust. The parser is syntax-tolerant: even files with errors will still yield partial structure for downstream heuristics.',
  },
  {
    name: 'Index',
    summary: 'Build a lightweight repository-local symbol index keyed by package plus directory. Scope the index per language for mixed repositories.',
    detail: 'The index is intentionally modest — it improves same-package and import-qualified call checks without pretending to replace full type analysis. In mixed-language repos, Go, Python, and Rust symbols are tracked separately so hallucination checks stay correct.',
  },
  {
    name: 'Heuristics',
    summary: 'Run explainable rule families that emit rule IDs, severity, messages, and evidence. Hold detail-only diagnostics back from default output.',
    detail: 'Each finding includes a rule ID, severity level, file path, line number, and an evidence payload written for human review. The --details flag adds full per-function fingerprint breakdowns. JSON output is available for pipeline integration.',
  },
]

// ─── Limitations by language ──────────────────────────────────────────────────

const limitations = {
  go: [
    'No authoritative Go type checking. Heuristics use structural patterns, not go/types.',
    'No full interprocedural or type-aware context propagation. Wrapper-chain reasoning stays repository-local and conservative.',
    'No proof of goroutine leaks, N+1 queries, or runtime performance regressions — only pattern signals.',
    'Package-method and local-symbol checks are repository-local; external packages are not indexed.',
    'The opt-in deeper semantic Go pack is still heuristic: it correlates nested-loop structure but does not prove asymptotic complexity or schema-aware DB cost.',
  ],
  python: [
    'No Python module graph resolution or installed-package awareness.',
    'No authoritative Python type analysis — hints are structural and conservative.',
    'No interprocedural propagation. Checks are local to individual functions or files.',
    'No proof of runtime behavior or end-to-end asyncio correctness — async findings remain syntax-driven heuristics.',
    'Cross-file duplicate detection is conservative and normalized; it is not exhaustive pairwise semantic comparison.',
  ],
  rust: [
    'No Rust trait resolution, cargo workspace modeling, or macro expansion.',
    'Rust rule pack is still growing, but current coverage already includes hygiene, hallucination, async/runtime, performance, domain-modeling, and unsafe-soundness checks.',
    'No proof of memory safety violations or lifetime errors from static analysis alone.',
    'Hallucination checks cover crate-local imports only; external crates are not indexed.',
    'No interprocedural analysis or cross-crate symbol resolution.',
  ],
}

// ─── Component ────────────────────────────────────────────────────────────────

export function DocsPage() {
  const [activeLang, setActiveLang] = useState<Language>('go')
  const [activeSection, setActiveSection] = useState<SectionId>('overview')

  const langClass = `lang-${activeLang}`
  const overview = overviewContent[activeLang]
  const rules = activeLang === 'go' ? goRules : activeLang === 'python' ? pythonRules : rustRules
  const commands = cliCommands[activeLang]
  const limits = limitations[activeLang]

  const handleLangChange = (lang: Language) => {
    setActiveLang(lang)
    setActiveSection('overview')
  }

  return (
    <div className="docs-layout">
      {/* ── Sidebar ─────────────────────────────────────────────────────── */}
      <aside className="docs-sidebar">
        {/* Language tabs */}
        <div className="docs-lang-tabs">
          {languages.map((lang) => (
            <button
              key={lang.id}
              className={`docs-lang-tab lang-${lang.id}${activeLang === lang.id ? ' active' : ''}`}
              onClick={() => handleLangChange(lang.id)}
              type="button"
            >
              {lang.label}
            </button>
          ))}
        </div>

        {/* Section navigation */}
        <p className="docs-sidebar-section-label">Sections</p>
        {sections.map((section) => (
          <button
            key={section.id}
            className={`docs-nav-item${activeSection === section.id ? ` active ${langClass}` : ''}`}
            onClick={() => setActiveSection(section.id)}
            type="button"
          >
            <span style={{ fontSize: '0.75rem', opacity: 0.65 }}>{section.icon}</span>
            {section.label}
          </button>
        ))}

        {/* Language note */}
        <div style={{ margin: '2rem 1.25rem 0', borderTop: '1px solid var(--border)', paddingTop: '1.25rem' }}>
          <p style={{ fontSize: '0.75rem', lineHeight: 1.6, color: 'var(--muted)', margin: 0 }}>
            Showing documentation for{' '}
            <span style={{ color: `var(--lang-${activeLang})`, fontFamily: 'var(--mono-font)', fontWeight: 600 }}>
              {activeLang}
            </span>
            . Switch the tab above to view another language.
          </p>
        </div>
      </aside>

      {/* ── Main content ─────────────────────────────────────────────────── */}
      <main className="docs-content">

        {/* OVERVIEW */}
        <div className={`docs-section${activeSection === 'overview' ? ' active' : ''}`}>
          <div className={`docs-eyebrow ${langClass}`}>{overview.title}</div>
          <h1 className="docs-h1">
            {activeLang === 'go' && 'Static analysis for Go repositories.'}
            {activeLang === 'python' && 'Static analysis for Python repositories.'}
            {activeLang === 'rust' && 'Static analysis for Rust repositories.'}
          </h1>
          <p className="docs-lead">{overview.lead}</p>

          <h2 className="docs-h2">What deslop does</h2>
          <p className="docs-p">
            deslop is a Rust-based static analyzer that looks for signals commonly associated with low-context or AI-generated code.
            It is intentionally conservative: findings are heuristics, not compile-time proof. The goal is to surface suspicious
            patterns quickly, explain why they were flagged, and let a reviewer decide whether the code is actually a problem.
          </p>

          <div className="docs-callout" style={{ borderLeftColor: `var(--lang-${activeLang})`, background: `var(--lang-${activeLang}-soft)` }}>
            <p>
              deslop auto-detects supported source files. The same command works for {overview.title.replace(' Analysis', '')}-only
              repositories and mixed-language repositories containing Go, Python, and Rust files.
            </p>
          </div>

          <h2 className="docs-h2">Pipeline properties</h2>
          <div className="docs-pill-list">
            {overview.bullets.map((bullet) => (
              <span key={bullet} className="docs-pill">{bullet}</span>
            ))}
          </div>

          <h2 className="docs-h2">Installation</h2>
          <p className="docs-p">Install the CLI from crates.io using Cargo:</p>
          <CodeBlock code="cargo install deslop" />
          <p className="docs-p">Or download prebuilt binaries from the GitHub release page:</p>
          <div className="docs-download-grid">
            {currentRelease.assets.map((asset) => (
              <a
                key={asset.id}
                className="docs-download-card"
                href={asset.url}
                target="_blank"
                rel="noreferrer"
              >
                <span className="docs-download-label">{asset.label}</span>
                <span className="docs-download-file">{asset.fileName}</span>
              </a>
            ))}
          </div>
          <p className="docs-p">
            Release overview:{' '}
            <a className="docs-link" href={currentRelease.releasePage} target="_blank" rel="noreferrer">
              {currentRelease.releasePage}
            </a>
          </p>
          <p className="docs-p">Or use the composite GitHub Action which downloads the correct binary for your runner automatically.</p>

          <h2 className="docs-h2">GitHub Actions</h2>
          <p className="docs-p">Scan the checked-out repository with defaults:</p>
          <CodeBlock code={githubActionWorkflow} />
          <p className="docs-p">Emit JSON, include detail-only findings, and keep the workflow green while you evaluate the report:</p>
          <CodeBlock code={githubActionJsonExample} />
          <p className="docs-p">Run benchmark mode instead of a scan:</p>
          <CodeBlock code={githubActionBenchExample} />

          <h3 className="docs-h3">Action inputs</h3>
          <table className="cli-table">
            <colgroup>
              <col className="cli-col-command" />
              <col className="cli-col-description" />
            </colgroup>
            <thead>
              <tr>
                <th>Input</th>
                <th>Description</th>
              </tr>
            </thead>
            <tbody>
              <tr><td>version</td><td>Release tag to install. When omitted, deslop uses the action ref if it is a full release tag; otherwise it downloads the latest release binary.</td></tr>
              <tr><td>command</td><td>Subcommand to run: <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem', color: 'var(--code)' }}>scan</code> or <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem', color: 'var(--code)' }}>bench</code>.</td></tr>
              <tr><td>path</td><td>Path to the repository you want to analyze. Defaults to <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem', color: 'var(--code)' }}>.</code></td></tr>
              <tr><td>json</td><td>Set to true to emit JSON output.</td></tr>
              <tr><td>details</td><td>Set to true to include detail-only findings for scan.</td></tr>
              <tr><td>no-ignore</td><td>Set to true to disable .gitignore filtering.</td></tr>
              <tr><td>enable-semantic</td><td>Set to true to enable the opt-in deeper semantic Go rule pack.</td></tr>
              <tr><td>fail-on-findings</td><td>Set to false to exit 0 even when scan findings are present.</td></tr>
              <tr><td>repeats</td><td>Benchmark repeat count for bench. Defaults to 5.</td></tr>
              <tr><td>warmups</td><td>Benchmark warmup count for bench. Defaults to 1.</td></tr>
            </tbody>
          </table>
        </div>

        {/* DETECTION RULES */}
        <div className={`docs-section${activeSection === 'detection-rules' ? ' active' : ''}`}>
          <div className={`docs-eyebrow ${langClass}`}>Detection rules</div>
          <h1 className="docs-h1">
            {rules.length} rules for {activeLang === 'go' ? 'Go' : activeLang === 'python' ? 'Python' : 'Rust'}.
          </h1>
          <p className="docs-lead">
            Each rule produces a finding with a rule ID, severity, file path, line number, and human-readable evidence.
            Findings are heuristics, not compile-time proof. deslop is conservative where full type information is missing.
          </p>

          <div className="docs-callout" style={{ borderLeftColor: `var(--lang-${activeLang})`, background: `var(--lang-${activeLang}-soft)` }}>
            <p>
              By default, <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem' }}>deslop scan</code> prints the standard finding set.
              Detail-only diagnostics are held back unless you pass <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem' }}>--details</code>.
            </p>
          </div>

          <h2 className="docs-h2">All {activeLang === 'go' ? 'Go' : activeLang === 'python' ? 'Python' : 'Rust'} rules</h2>
          <div className="rule-grid">
            {rules.map((rule) => (
              <div key={rule.id} className="rule-item">
                <div>
                  <span className={`rule-tag ${langClass}`}>{rule.id}</span>
                </div>
                <div>
                  <p className="rule-desc">{rule.description}</p>
                </div>
              </div>
            ))}
          </div>

          {activeLang === 'go' && (
            <>
              <h2 className="docs-h2">Detection philosophy</h2>
              <p className="docs-p">Findings are heuristics, not compile-time proof. The analyzer is intentionally conservative where full type information is missing.</p>
              <p className="docs-p">Rules are designed to produce readable evidence so humans can validate them quickly. Local repository context is used where possible, but deslop does not replace go/types.</p>
            </>
          )}
          {activeLang === 'python' && (
            <>
              <h2 className="docs-h2">Shared signals</h2>
              <p className="docs-p">Python also inherits the shared cross-language layer when parser evidence supports it, including naming-quality checks, doc-comment hygiene, conservative test-quality findings, <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem', color: 'var(--code)' }}>hardcoded_secret</code>, <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem', color: 'var(--code)' }}>full_dataset_load</code>, and <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem', color: 'var(--code)' }}>string_concat_in_loop</code>.</p>
            </>
          )}
          {activeLang === 'rust' && (
            <>
              <h2 className="docs-h2">Growing rule pack</h2>
              <p className="docs-p">The Rust rule pack now covers leftovers and comment hygiene, crate-local hallucination checks, async/runtime hazards, performance smells, domain-modeling anti-patterns, and unsafe-soundness operations. Stronger trait resolution, macro expansion, and cargo-workspace modeling are still pending.</p>
            </>
          )}
        </div>

        {/* CLI COMMANDS */}
        <div className={`docs-section${activeSection === 'cli-commands' ? ' active' : ''}`}>
          <div className={`docs-eyebrow ${langClass}`}>CLI reference</div>
          <h1 className="docs-h1">Commands and flags.</h1>
          <p className="docs-lead">
            Run deslop from the repository root. The same binary handles Go, Python, and Rust files — language detection is automatic based on file extensions.
          </p>

          <h2 className="docs-h2">
            {activeLang === 'go' ? 'Go' : activeLang === 'python' ? 'Python' : 'Rust'} commands
          </h2>
          <table className="cli-table">
            <colgroup>
              <col className="cli-col-command" />
              <col className="cli-col-description" />
            </colgroup>
            <thead>
              <tr>
                <th>Command</th>
                <th>Description</th>
              </tr>
            </thead>
            <tbody>
              {commands.map((command) => (
                <tr key={command.cmd}>
                  <td>{command.cmd}</td>
                  <td>{command.desc}</td>
                </tr>
              ))}
            </tbody>
          </table>

          <h2 className="docs-h2">Global flags</h2>
          <table className="cli-table">
            <colgroup>
              <col className="cli-col-command" />
              <col className="cli-col-description" />
            </colgroup>
            <thead>
              <tr>
                <th>Flag</th>
                <th>Description</th>
              </tr>
            </thead>
            <tbody>
              <tr><td>--details</td><td>Include full per-function fingerprint details and detail-only findings.</td></tr>
              <tr><td>--enable-semantic</td><td>Enable the opt-in deeper semantic Go heuristics for the current scan or benchmark run.</td></tr>
              <tr><td>--ignore RULE1,RULE2</td><td>Ignore specific rule IDs for one scan invocation after analysis completes.</td></tr>
              <tr><td>--json</td><td>Emit structured JSON instead of human-readable text output.</td></tr>
              <tr><td>--no-fail</td><td>Exit 0 even when findings are present.</td></tr>
              <tr><td>--no-ignore</td><td>Disable .gitignore filtering and scan all files under the target path.</td></tr>
              <tr><td>--warmups N</td><td>Benchmark warmup iterations for bench. Defaults to 1.</td></tr>
              <tr><td>--repeats N</td><td>Benchmark repeat count for bench. Defaults to 5.</td></tr>
            </tbody>
          </table>

          <h2 className="docs-h2">Repository config</h2>
          <p className="docs-p">
            Repository-local behavior can also be tuned with a <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem', color: 'var(--code)' }}>.deslop.toml</code>
            file at the scan root. The current config surface supports disabled rules, severity overrides, suppressed path prefixes, the opt-in Go semantic pack, and the staged Rust async pack toggle.
          </p>
          <CodeBlock code={repositoryConfigExample} />

          <h2 className="docs-h2">Output modes</h2>
          <p className="docs-p">Text output (default) prints the scan summary plus the standard finding set. JSON output is available for pipeline integration. The --details flag adds per-function fingerprint data to either output mode.</p>
          <CodeBlock code={`# Text output (default)
deslop scan . > results.txt

# JSON output
deslop scan --json . > results.json

# Full detail output
deslop scan --details --json .`} />
        </div>

        {/* PIPELINE */}
        <div className={`docs-section${activeSection === 'pipeline' ? ' active' : ''}`}>
          <div className={`docs-eyebrow ${langClass}`}>Pipeline</div>
          <h1 className="docs-h1">A local analysis pipeline built for speed and readable output.</h1>
          <p className="docs-lead">
            deslop discovers files, parses structure, builds a lightweight language-scoped index, and runs explainable heuristics.
            Each stage is designed to be fast and independently composable.
          </p>

          {pipelineStages.map((stage, index) => (
            <div key={stage.name} style={{ marginBottom: '2.5rem' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '0.75rem' }}>
                <span style={{
                  fontFamily: 'var(--mono-font)',
                  fontSize: '0.72rem',
                  fontWeight: 700,
                  color: `var(--lang-${activeLang})`,
                  background: `var(--lang-${activeLang}-badge)`,
                  padding: '0.15rem 0.5rem',
                  letterSpacing: '0.08em',
                }}>
                  {String(index + 1).padStart(2, '0')}
                </span>
                <h2 style={{ margin: 0, fontFamily: 'var(--heading-font)', fontSize: '1.2rem', fontWeight: 700, letterSpacing: '-0.03em', color: 'var(--text-strong)' }}>
                  {stage.name}
                </h2>
              </div>
              <p className="docs-lead" style={{ fontSize: '1rem', marginBottom: '0.75rem' }}>{stage.summary}</p>
              <p className="docs-p">{stage.detail}</p>
            </div>
          ))}

          <h2 className="docs-h2">Benchmarking</h2>
          <p className="docs-p">
            The <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem', color: 'var(--code)' }}>bench</code> command measures each pipeline stage individually — discovery, parse, index, heuristics, and total runtime.
            A documented baseline of 180.80 ms was measured against a realistic Go repository of 89 files and 702 functions.
          </p>
          <CodeBlock code="cargo run -- bench --warmups 2 --repeats 5 /path/to/repo" />

          <h2 className="docs-h2">Mixed-language repositories</h2>
          <p className="docs-p">
            deslop handles mixed Go, Python, and Rust repositories in a single scan pass. The local symbol index is language-scoped, so
            Go, Python, and Rust symbols are tracked separately. Hallucination checks remain accurate across language boundaries.
          </p>
        </div>

        {/* LIMITATIONS */}
        <div className={`docs-section${activeSection === 'limitations' ? ' active' : ''}`}>
          <div className={`docs-eyebrow ${langClass}`}>Limitations</div>
          <h1 className="docs-h1">What deslop does not do.</h1>
          <p className="docs-lead">
            deslop is explicit about what it can and cannot prove. It surfaces suspicious patterns quickly and leaves the final
            judgment to engineers. The following limitations apply to{' '}
            {activeLang === 'go' ? 'Go' : activeLang === 'python' ? 'Python' : 'Rust'} analysis specifically.
          </p>

          <div className="docs-callout" style={{ borderLeftColor: `var(--lang-${activeLang})`, background: `var(--lang-${activeLang}-soft)` }}>
            <p>
              Findings are heuristics, not compile-time proof. The analyzer is intentionally conservative where full type
              information is missing.
            </p>
          </div>

          <h2 className="docs-h2">
            {activeLang === 'go' ? 'Go' : activeLang === 'python' ? 'Python' : 'Rust'} analysis limitations
          </h2>
          <div className="rule-grid">
            {limits.map((limit) => (
              <div key={limit} style={{ padding: '0.85rem 1rem', border: '1px solid var(--border)', background: 'var(--accent-soft)', fontSize: '0.9rem', lineHeight: 1.65, color: 'var(--muted)' }}>
                {limit}
              </div>
            ))}
          </div>

          <h2 className="docs-h2">General limitations</h2>
          <div className="rule-grid">
            {[
              'No full interprocedural context propagation. Most analysis is local to each function, with only conservative repository-local wrapper-chain awareness.',
              'Package-method and local-symbol checks are repository-local and language-scoped for mixed-language repositories.',
              'No proof of runtime behavior: goroutine leaks, N+1 query counts, or actual memory pressure are not detectable from static structure alone.',
            ].map((limit) => (
              <div key={limit} style={{ padding: '0.85rem 1rem', border: '1px solid var(--border)', background: 'var(--accent-soft)', fontSize: '0.9rem', lineHeight: 1.65, color: 'var(--muted)' }}>
                {limit}
              </div>
            ))}
          </div>

          <h2 className="docs-h2">Planned improvements</h2>
          <p className="docs-p">The following capabilities are pending or in development:</p>
          <div className="docs-pill-list">
            {activeLang === 'go' && ['Index-assisted DB call classification', 'Public-API-aware context propagation', 'AST-resolved ctx.Done detection inside goroutines', 'Type-aware Go analysis'].map((item) => <span key={item} className="docs-pill">{item}</span>)}
            {activeLang === 'python' && ['Installed-package and module-graph awareness', 'Deeper interprocedural asyncio reasoning', 'Optional type-aware data-flow analysis', 'Framework-specific rule packs (Django/FastAPI)'].map((item) => <span key={item} className="docs-pill">{item}</span>)}
            {activeLang === 'rust' && ['Trait resolution', 'Cargo workspace modeling', 'Macro expansion analysis', 'Deeper Rust rule pack', 'Cross-crate symbol resolution'].map((item) => <span key={item} className="docs-pill">{item}</span>)}
          </div>
        </div>

        {/* ABOUT */}
        <div className={`docs-section${activeSection === 'about' ? ' active' : ''}`}>
          <div className="docs-eyebrow" style={{ color: 'var(--muted)' }}>About</div>
          <h1 className="docs-h1">A "Sloppy" Attempt at a Slop Detector.</h1>
          <p className="docs-lead">
            This is an early-stage experiment in identifying AI-generated slop.
          </p>
          <h3>The Philosophy</h3>
          <p className="docs-p">
            If the folks at Anthropic and Peter Steinberger can generate full-fledged applications without manually writing every line of code, then we can certainly vibecode a tool to detect the resulting "slop."
          </p>
          <p className="docs-p">
            We’re fighting fire with fire. This project is mostly vibecoded, but the architecture is built with intention. Instead of just calling things slop, let’s build a better filter together.
          </p>

          <div className="docs-callout" style={{ borderLeftColor: 'var(--border-strong)', background: 'var(--accent-soft)' }}>
            <p>
              Before coming @ me — I am trying to solve a real problem. The project is mostly vibecoded,
              but the architecture is thought through as per my best knowledge. Instead of calling this slop,
              let's try to work together if you want. Send me more ideas by{' '}
              <a
                href="https://github.com/chinmay-sawant/deslop/issues/new"
                target="_blank"
                rel="noreferrer"
                style={{ color: 'var(--text-strong)', textDecoration: 'underline', textDecorationColor: 'var(--border-strong)', textUnderlineOffset: '3px' }}
              >
                creating a new issue
              </a>
              .
            </p>
          </div>

          <h2 className="docs-h2">Open-source & free</h2>
          <p className="docs-p">
            Going to keep this as open-source. Got no intention to monetize the application — for now :3
          </p>
          <p className="docs-p">
            The full source is on{' '}
            <a
              href="https://github.com/chinmay-sawant/deslop"
              target="_blank"
              rel="noreferrer"
              style={{ color: 'var(--text-strong)', textDecoration: 'underline', textDecorationColor: 'var(--border-strong)', textUnderlineOffset: '3px' }}
            >
              GitHub
            </a>
            {' '}under the MIT license. Contributions, ideas, and bug reports are all welcome.
          </p>

          <h2 className="docs-h2">Built & vibecoded by</h2>
          <p className="docs-p">
            <a
              href="https://github.com/chinmay-sawant"
              target="_blank"
              rel="noreferrer"
              style={{ color: 'var(--text-strong)', textDecoration: 'underline', textDecorationColor: 'var(--border-strong)', textUnderlineOffset: '3px' }}
            >
              Chinmay Sawant
            </a>
            {' '}with ❤️
          </p>
        </div>

      </main>
    </div>
  )
}
