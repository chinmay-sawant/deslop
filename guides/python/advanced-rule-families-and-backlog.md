# Phase 4: Advanced Python Rule Families And Backlog Expansion

## Overview

This phase exists because the attached Python backlog in `python.md` is much broader than the currently shipped Python support. Phases 1 through 3 established a conservative backend, parser contract, first rule pack, and rollout baseline. They do not fully cover advanced Python performance smells, architectural and design smells, duplication, AI-generated-code signals, or the larger maintainability backlog.

The goal of this phase is not to promise that every item from the attached notes can be shipped as a low-noise static rule. The goal is to turn the remaining backlog into an explicit implementation contract with clear triage boundaries:

- what can be implemented conservatively with the current architecture
- what needs additional parser or repository-level evidence
- what likely needs a new shared abstraction or cross-file index support
- what should remain deferred because it depends on runtime profiling, framework semantics, or subjective review judgment

This phase should treat the user-supplied backlog as a rule-family roadmap, not as permission to add noisy detectors quickly.

Checkbox rule for this document:

- [x] A checked box means the backlog item now has a final Phase 4 disposition: shipped, partially shipped with a named rule, or intentionally deferred or non-goal with that status recorded explicitly.

## Current Implementation Snapshot

The current Phase 4 baseline is now implemented for a conservative subset of the backlog. The shipped additions are:

- [x] `none_comparison`
- [x] `side_effect_comprehension`
- [x] `redundant_return_none`
- [x] `hardcoded_path_string`
- [x] `variadic_public_api`
- [x] `list_materialization_first_element`
- [x] `deque_candidate_queue`
- [x] `temporary_collection_in_loop`
- [x] `recursive_traversal_risk`
- [x] `list_membership_in_loop`
- [x] `repeated_len_in_loop`
- [x] `builtin_reduction_candidate`
- [x] `broad_exception_handler`
- [x] `missing_context_manager`
- [x] `public_api_missing_type_hints`
- [x] `mixed_sync_async_module`
- [x] `god_function`
- [x] `god_class`
- [x] `monolithic_init_module`
- [x] `too_many_instance_attributes`
- [x] `eager_constructor_collaborators`
- [x] `over_abstracted_wrapper`
- [x] `mixed_concerns_function`
- [x] `name_responsibility_mismatch`
- [x] `deep_inheritance_hierarchy`
- [x] `tight_module_coupling`
- [x] `textbook_docstring_small_helper`
- [x] `mixed_naming_conventions`
- [x] `unrelated_heavy_import`
- [x] `obvious_commentary`
- [x] `enthusiastic_commentary`
- [x] `repeated_string_literal`
- [x] `duplicate_error_handler_block`
- [x] `duplicate_validation_pipeline`
- [x] `duplicate_test_utility_logic`
- [x] `cross_file_repeated_literal`
- [x] `duplicate_query_fragment`
- [x] `cross_file_copy_paste_function`
- [x] `duplicate_transformation_pipeline`
- [x] `monolithic_module`

The backlog items that still remain deferred are now explicitly marked as deferred or non-goals in the resolved checklist below rather than left as open tasks.

## In Scope

- [x] Expand the Python roadmap to explicitly cover the missing backlog from `python.md`
- [x] Group the missing backlog into rule families with implementable evidence policies
- [x] Define which advanced Python rules belong in shared heuristics versus `src/heuristics/python/`
- [x] Plan parser and analysis-model extensions required for the next Python rule families
- [x] Plan repository-level evidence needed for duplication and coupling-oriented signals
- [x] Define a conservative policy for AI-generated-code smell detection so subjective signals do not turn into arbitrary noise
- [x] Define which backlog items should remain permanently non-goals for static analysis
- [x] Establish fixture and verification expectations for each new Python rule family

## Out Of Scope

- Claiming that all backlog items are now implemented
- Shipping profiling-only performance findings as if they were statically provable
- Pretending framework-specific behavior is generic Python behavior without explicit evidence
- Rebuilding the whole index or parser architecture before a concrete rule needs it
- Turning subjective code review preferences into findings without a documented false-positive policy

## Dependencies

- Depends on Phases 1 through 3 because this phase builds on the released Python backend, parser contract, and rollout baseline.
- May require follow-on shared-model work if advanced Python rule families need cross-file evidence that the current `ParsedFile` and `ParsedFunction` contracts do not carry.
- May require future repository-index extensions if duplication, coupling, or module-shape rules need normalized cross-file summaries.

## Primary Code Areas

- `src/analysis/types.rs`
- `src/analysis/python/mod.rs`
- `src/analysis/python/parser/mod.rs`
- `src/analysis/python/parser/general.rs`
- `src/analysis/python/parser/comments.rs`
- `src/analysis/python/parser/performance.rs`
- `src/heuristics/mod.rs`
- `src/heuristics/common.rs`
- `src/heuristics/python/mod.rs`
- `src/heuristics/python/performance.rs`
- `src/heuristics/python/maintainability.rs`
- `src/heuristics/python/structure.rs`
- `src/heuristics/python/duplication.rs`
- `src/heuristics/python/ai_smells.rs`
- `src/index/mod.rs`
- `tests/integration_scan/python.rs`
- `tests/fixtures/python/`
- `guides/features-and-detections.md`
- `guides/implementation-guide.md`

## Implementation Checkpoints

1. Classify the missing backlog from `python.md` into implementation buckets.

	Required buckets:

	- [x] implementable now with mostly syntactic evidence
	- [x] implementable after modest parser enrichment
	- [x] implementable only after repository-level or cross-file evidence exists
	- [x] intentionally deferred because they need runtime profiling, framework semantics, or reviewer judgment

	This checkpoint is required because the missing backlog spans everything from obvious `== None` misuse to profiling-only query and container-efficiency claims.

2. Expand the Python performance backlog with conservative static policies.

	The current roadmap already covers:

	- [x] `string_concat_in_loop`
	- [x] `blocking_sync_io_in_async`
	- [x] `full_dataset_load`

	The missing performance backlog should be triaged explicitly as follows.

	Good Phase 4 candidates with conservative evidence policies:

	- [x] using `list(...) [0]` or equivalent eager materialization where `next(iter(...))` would avoid building a list
	- [x] repeated temporary list or dict construction inside obvious hot loops
	- [x] recursion on clearly deep traversal helpers when the function shape suggests unbounded nesting risk
	- [x] using list membership checks inside loops when the looked-up value is stable and the container is obviously list-like
	- [x] repeated `len()` calls inside loops only when the receiver is obviously unchanged and the pattern is directly local
	- [x] not using `collections.deque` for repeated queue-style `pop(0)` or `insert(0, ...)` operations

	Candidates that likely need stronger evidence before shipping:

	- [x] list-versus-set or dict lookup recommendations when the value flow is unclear remain deferred
	- [x] recomputing the same value repeatedly instead of caching remains deferred
	- [x] temporary-object churn in loops when the loop is not obviously hot is now partially covered by `temporary_collection_in_loop`
	- [x] overusing `try` and `except` in loops where context may matter remains deferred

	Candidates that should stay deferred or documentation-only because static analysis cannot justify them honestly:

	- [x] ignoring database indexes in the general case remains deferred
	- [x] N+1 queries without recognizable query-call evidence beyond the current generic data-access rules remain deferred
	- [x] pandas-versus-pure-Python recommendations based on dataset size or workload shape remain a non-goal
	- [x] overuse of globals as a performance claim rather than a design smell remains deferred

	Evidence rule:

	- [x] No Phase 4 performance rule claims a measured regression; messages stay at the level of likely hot-path risk or avoidable allocation pattern.

3. Define the advanced architecture and design-smell family.

	The current roadmap mentions monolithic `__init__.py`, god functions, and oversized classes only as backlog. This phase should turn them into explicit candidates with evidence thresholds.

	Candidate families:

	- [x] god functions based on line span, branching density, local-name churn, call-site fan-out, and mixed responsibility signals in a single function
	- [x] god classes based on method count, responsibility breadth, and instance-variable fan-out rather than a single raw threshold
	- [x] classes with excessive instance variables using `self.` assignment counts with constructor and method evidence
	- [x] deep inheritance hierarchies when base-class chains are locally visible and repository-scoped
	- [x] monolithic modules including oversized `__init__.py` files, especially when they mix exports, business logic, I/O, and configuration loading
	- [x] hardcoded business rules or magic decision tables remain deferred behind stronger evidence work
	- [x] mixing concerns such as HTTP, persistence, and business logic in one function when the call and import evidence is locally obvious
	- [x] constructors that create unrelated collaborators eagerly inside `__init__`
	- [x] misleading names that mask responsibility only when the mismatch is supported by structural evidence rather than taste alone are now partially covered by `name_responsibility_mismatch`
	- [x] over-fetching or overly broad data-return shapes only when the query or serialization evidence is visible locally remain deferred
	- [x] tight coupling between modules if import fan-in and direct constructor usage can be summarized conservatively
	- [x] over-abstraction patterns only when there is a low-noise structural signature, not because a design feels "too abstract"

	Threshold rule:

	- [x] Design-smell thresholds are explicitly documented and justified with fixtures so they do not become arbitrary style preferences.

4. Define the duplication strategy instead of leaving it as a vague non-goal.

	The attached backlog calls out duplication explicitly and the current roadmap does not provide a dedicated plan. This phase should define a duplication strategy with narrow initial scope.

	Recommended initial scope:

	- [x] repeated error-handling blocks with highly similar AST shapes in the same file
	- [x] repeated validation pipelines within the same module or directory
	- [x] repeated string literals or query fragments above a configurable threshold
	- [x] duplicate utility logic split between tests and production code when function fingerprints and token shapes are very close

	Recommended non-goals for the first duplication pass:

	- [x] whole-repository clone detection remains a non-goal
	- [x] semantic equivalence across renamed variables or reordered statements remains a non-goal
	- [x] cross-language duplication remains a non-goal

	Implementation guidance:

	- [x] start from function and block fingerprints already used elsewhere in deslop
	- [x] prefer near-duplicate block clusters over line-based similarity percentages
	- [x] require reviewer-readable evidence such as matched snippet summaries, block counts, and normalized token-shape counts

5. Define an AI-generated-code smell policy with hard false-positive boundaries.

	The current phases mention low-noise AI-smell detection only as backlog. This phase should make the policy explicit.

	Potential candidates with moderate static signal if kept conservative:

	- [x] inconsistent naming conventions within one file, such as mixed `snake_case` and `camelCase`, when the file is otherwise coherent enough to compare local symbols
	- [x] imports from obviously unrelated heavy ecosystems when usage evidence shows most imported modules are unused or irrelevant
	- [x] boilerplate try and except wrappers repeated around operations that do not raise the handled class in any visible way are now partially covered by `duplicate_error_handler_block`
	- [x] over-commenting of obvious operations when the comment-style heuristics and statement shape strongly agree
	- [x] emojis or highly enthusiastic comments in production code
	- [x] suspiciously textbook docstrings on tiny helpers when docstrings restate the code without adding domain context

	Candidates that should stay deferred unless a very strong evidence policy is defined:

	- [x] overly descriptive variable names by taste alone remain deferred
	- [x] perfectly balanced function decomposition as a finding by itself remains a non-goal
	- [x] "zero personality" or "textbook" structure without a measurable proxy remains a non-goal
	- [x] variable names that merely feel slightly off remain a non-goal

	Policy rule:

	- [x] An AI-smell finding always points to a concrete structural signal that would still matter in a non-AI-authored file.

6. Expand the maintainability backlog into explicit rule candidates.

	The current shipped Python rules only cover a subset of the maintainability list. This phase should define the missing maintainability family explicitly.

	Strong static candidates:

	- [x] `== None` and `!= None` comparisons instead of `is None` and `is not None`
	- [x] loop-based reductions or searches that should use obvious built-ins such as `any`, `all`, `sum`, `max`, or `min`
	- [x] list comprehensions used only for side effects
	- [x] explicit `return None` in functions that otherwise just fall through
	- [x] hardcoded file-system paths where `pathlib` or configuration indirection is more appropriate
	- [x] commented-out dead code blocks that look intentionally disabled rather than explanatory comments
	- [x] broad `except Exception:` handlers that suppress or obscure failure beyond the currently shipped swallowed-exception rule
	- [x] missing context-manager usage for files, locks, or other obviously closable resources when the open/close lifecycle is visible locally
	- [x] overuse of `*args` and `**kwargs` on public APIs when signatures become opaque without a strong forwarding reason
	- [x] mixed sync and async boundaries inside one module or function family when call and declaration evidence is strong enough

	Candidates that need careful policy work:

	- [x] public APIs missing type hints, because repository style and Python-version targets vary
	- [x] ignoring virtual environments or `pyproject.toml`, because those are repository-level conventions rather than per-file heuristics, remains deferred

	Compatibility rule:

	- [x] Where a maintainability signal is honestly cross-language in meaning, prefer extending shared heuristics instead of duplicating Python-only logic.

7. Define the parser and shared-model extensions required for Phase 4 rule families.

	Potential parser additions:

	- [x] richer assignment-shape capture for `self.` fields, constructor collaborators, and magic constants
	- [x] lightweight loop-body summaries for queue operations, temporary allocations, repeated length checks, and side-effect-only comprehensions
	- [x] richer call-target normalization for file I/O, HTTP, DB, and cache-like APIs
	- [x] class inheritance summaries and local base-resolution when repository-local evidence exists
	- [x] comment and docstring summaries that can distinguish explanatory context from restated code
	- [x] string-literal aggregation for repeated literal and query-fragment detection

	Shared-model rule:

	- [x] Do not bloat shared analysis types with Python-only detail unless another language can use the same concept honestly.

8. Define repository-level evidence needs before adding cross-file rules.

	Advanced Python rules that need more than one file should identify that dependency up front.

	Likely repository-level needs:

	- [x] file and module summaries for monolithic module detection
	- [x] symbol-to-module ownership for tight-coupling heuristics
	- [x] normalized duplicate-block fingerprints for duplication rules
	- [x] repeated literal and query-fragment frequency counts across a package or repository slice

	Boundary rule:

	- [x] If a rule needs repository aggregation, that dependency stays explicit in code structure, tests, and documentation rather than hidden inside a heuristic helper.

9. Define rule-family fixture strategy before implementation.

	Required fixture coverage for every Phase 4 rule family:

	- [x] positive and negative fixtures for each rule
	- [x] mixed-rule fixtures that prove signals compose without duplicate spam
	- [x] suppression fixtures for test code, CLI entrypoints, framework glue, and intentionally acceptable patterns
	- [x] false-positive fixtures for library-style APIs, generated-looking but legitimate documentation, and repository conventions that resemble AI or duplication smells

	Duplication-specific verification:

	- [x] Verify that near-duplicate blocks produce one understandable finding cluster instead of one finding per repeated line.

10. Produce an explicit status map for every backlog item from the attached notes.

	This phase should end with every backlog item from the user-provided list classified as one of:

	- [x] shipped already in Phases 1 through 3
	- [x] targeted by a concrete Phase 4 rule family
	- [x] deferred pending parser enrichment
	- [x] deferred pending repository-level evidence
	- [x] intentionally non-goal because static analysis would be misleading

	That status map is required so the roadmap can answer "do we cover this idea?" without rereading free-form prose.

## Backlog Status Map

This section maps the user-provided Python backlog to current roadmap status.

### Performance And Unoptimized Code

- [x] using `+` for string concatenation in a loop: already shipped in Phases 1 through 3 as `string_concat_in_loop`
- [x] looping over a list when a set or dict lookup would be more appropriate remains deferred pending stronger local-container evidence
- [x] using a list for membership checks instead of a set: shipped in the current Phase 4 baseline as `list_membership_in_loop`
- [x] repeatedly calling `len()` inside a tight loop: shipped in the current Phase 4 baseline as `repeated_len_in_loop`
- [x] overusing global variables as a performance claim remains deferred as a pure performance rule; it may be reconsidered later as a design smell instead
- [x] blocking async code with sync I/O: already shipped in Phases 1 through 3 as `blocking_sync_io_in_async`
- [x] using `list(...)[0]` instead of `next(iter(...))`: shipped in the current Phase 4 baseline as `list_materialization_first_element`
- [x] creating temporary lists or dicts inside hot loops: shipped in the current Phase 4 baseline as `temporary_collection_in_loop`
- [x] using recursion for deep structures: shipped in the current Phase 4 baseline as `recursive_traversal_risk`
- [x] not using `collections.deque` for queue-style operations: shipped in the current Phase 4 baseline for queue-style `pop(0)` and `insert(0, ...)` patterns as `deque_candidate_queue`
- [x] loading huge datasets into memory instead of streaming: partially shipped already through `full_dataset_load`; further streaming-specific expansion is still pending
- [x] using pandas for tiny data or pure Python for huge data remains an intentional non-goal for static analysis without workload evidence
- [x] recomputing the same value repeatedly instead of caching remains deferred pending stronger local data-flow evidence
- [x] overusing `try` and `except` inside loops remains deferred pending a low-noise evidence policy; Phase 4 only covers narrower repeated-wrapper shapes
- [x] ignoring database indexes or writing N+1 queries remains deferred; N+1-style query work already exists generically in deslop, but Python-specific index-awareness is still likely framework-specific

### Architectural And Design Smells

- [x] god classes or functions: shipped in the current Phase 4 baseline as `god_function` and `god_class`
- [x] classes with 20 or more instance variables: shipped in the current Python baseline through the escalated 20-plus tier of `too_many_instance_attributes`
- [x] using classes where a simple function or dataclass would suffice: partially shipped in the current Phase 4 baseline under `over_abstracted_wrapper`
- [x] deep inheritance hierarchies: shipped in the current Phase 4 baseline as `deep_inheritance_hierarchy`
- [x] monolithic `__init__.py` files or single huge modules: shipped in the current Python baseline as `monolithic_init_module` and a tightened `monolithic_module` policy that now requires a 1500-line floor plus coordinator-style or mixed-concern evidence
- [x] hardcoded business logic instead of configuration remains deferred behind stronger evidence thresholds
- [x] mixing business logic, HTTP, and DB concerns in one function: shipped in the current Phase 4 baseline as `mixed_concerns_function`
- [x] creating unrelated objects inside `__init__`: shipped in the current Phase 4 baseline as `eager_constructor_collaborators`
- [x] misleading names that hide the real responsibility are partially shipped in the current Phase 4 baseline as `name_responsibility_mismatch`
- [x] verb and subject reversal in naming such as `process_user` versus `user.process` remains deferred because this is often too style- and codebase-dependent
- [x] returning more data than needed remains deferred pending stronger call-site and data-shape evidence
- [x] tight coupling between modules: shipped in the current Phase 4 baseline as `tight_module_coupling`
- [x] magic numbers or strings without constants or enums remain deferred
- [x] reinventing the wheel such as a custom JSON parser instead of established libraries remains deferred because library-choice judgment is hard to make honestly with local syntax alone
- [x] over-abstraction patterns where a simple map or helper would do: shipped in the current Phase 4 baseline as `over_abstracted_wrapper`

### Code Duplication

- [x] copy-paste functions across files are now shipped through `cross_file_copy_paste_function` with `duplicate_test_utility_logic` still covering the test-versus-production overlap case
- [x] duplicate error-handling blocks repeated many times: shipped in the current Phase 4 baseline as `duplicate_error_handler_block`
- [x] same validation logic across endpoints or services: shipped in the current Phase 4 baseline as `duplicate_validation_pipeline`
- [x] repeated string literals or query fragments: shipped in the current Python baseline as `repeated_string_literal` and `duplicate_query_fragment`
- [x] duplicate data-transformation pipelines are now shipped as `duplicate_transformation_pipeline`
- [x] same utility logic in tests and production code: shipped in the current Phase 4 baseline as `duplicate_test_utility_logic`

### AI-Generated-Code Smells

- [x] overly descriptive variable names remain deferred because this is too taste-driven without a stronger evidence policy
- [x] tiny functions with Wikipedia-style docstrings: shipped in the current Phase 4 baseline as `textbook_docstring_small_helper`
- [x] imports from unrelated ecosystems when only one small capability is used: shipped in the current Phase 4 baseline as `unrelated_heavy_import`
- [x] over-commenting the obvious: shipped in the current Phase 4 baseline as `obvious_commentary`
- [x] perfectly balanced tiny functions that should be merged remains an intentional non-goal unless a future structural proxy proves low-noise enough
- [x] boilerplate try and except everywhere is now partially covered by `duplicate_error_handler_block`
- [x] emojis or overly enthusiastic comments: shipped in the current Phase 4 baseline as `enthusiastic_commentary`
- [x] structurally flawless code that lacks real-world context such as logging, retries, or environment handling remains deferred because absence-of-context is too subjective without framework-aware evidence
- [x] toy-problem solutions that assume perfect input remains deferred because robustness expectations vary by project and layer
- [x] inconsistent naming in the same file: shipped in the current Phase 4 baseline as `mixed_naming_conventions`
- [x] unexplained magic numbers like `17` or `2.718` remain deferred
- [x] classes for what should be two lines of code: partially shipped in the current Phase 4 baseline under `over_abstracted_wrapper`
- [x] variable names that are just slightly off remains an intentional non-goal without a measurable policy
- [x] feels like textbook code with no personality remains an intentional non-goal unless it can be reduced to a concrete structural smell

### Maintainability And Readability

- [x] using `== None` instead of `is None`: shipped in the current Phase 4 baseline as `none_comparison`
- [x] writing loops instead of clear built-ins such as `any`, `all`, `sum`, or `max`: shipped in the current Phase 4 baseline as `builtin_reduction_candidate`
- [x] unnecessary list comprehensions for side effects: shipped in the current Phase 4 baseline as `side_effect_comprehension`
- [x] returning `None` explicitly in every function: partially shipped in the current Phase 4 baseline as `redundant_return_none`
- [x] using `eval()` or `exec()`: already shipped in Phases 1 through 3 as `eval_exec_usage`
- [x] hardcoding file paths instead of using `pathlib` plus configuration: shipped in the current Phase 4 baseline as `hardcoded_path_string`
- [x] ignoring virtual environments and `pyproject.toml` remains deferred as a repository-convention check rather than a per-file code smell
- [x] print debugging in production code: already shipped in Phases 1 through 3 as `print_debugging_leftover`
- [x] no type hints on public APIs: shipped in the current Phase 4 baseline as `public_api_missing_type_hints`
- [x] overusing `**kwargs` and `*args` to avoid interface design: shipped in the current Phase 4 baseline as `variadic_public_api`
- [x] commenting out dead code instead of deleting it: shipped in the current Phase 4 baseline as `commented_out_code`
- [x] huge `try`/`except Exception:` blocks with `pass`: partially shipped already through `exception_swallowed`; broader obscuring-exception coverage is still pending
- [x] not using context managers for files, locks, or DB connections: shipped in the current Phase 4 baseline as `missing_context_manager`
- [x] mixing sync and async in the same codebase without clear boundaries: shipped in the current Phase 4 baseline as `mixed_sync_async_module`

## Future Work Checkpoints For Deferred And Partial Items

This section tightens the deferred entries above into explicit follow-up checkpoints. The goal is not to silently broaden Phase 4 scope after the fact. The goal is to make the remaining backlog actionable enough that later work can promote or reject items deliberately instead of reopening the entire backlog from scratch.

The detailed follow-on execution contract for these checkpoints lives in [Phase 5](profiling-sensitive-and-deferred-gap-closure.md). This Phase 4 section remains the authoritative status map for what is already shipped, what is partial, and what is still deferred.

### Profiling-Sensitive Performance Follow-Ups

1. Container-choice and repeated-work signals need a stricter static proxy before they graduate.

	Future work should only expand these rules if the implementation can prove one of the following with local evidence and low noise:

	- a list or tuple is used repeatedly for membership checks in a hot-looking loop and is not intentionally tiny or append-only
	- the same normalized expression is recomputed multiple times in one loop body with no intervening writes that would justify recomputation
	- a repeated `try` and `except` wrapper sits inside an iterative path and the exception handling is materially part of the loop body instead of one defensive outer boundary

	Any such work must keep benchmark visibility because this backlog category was originally motivated by performance claims rather than only readability.

2. Streaming versus eager loading needs a narrower evidence policy than "large load equals bad."

	Future expansion should distinguish between:

	- legitimate bootstrap or configuration loads
	- application paths that materialize large intermediate collections and then consume only part of them
	- file, response, or query result flows where generator-style or chunked handling is visible in the surrounding syntax

	This should likely extend `full_dataset_load` rather than create a broad new smell with weaker evidence.

3. Query and database performance work must stay honest about framework knowledge.

	N+1, index usage, and over-fetching should not be promoted to a generic Python rule unless there is one of these narrower anchors:

	- repository-level repeated query-fragment signatures that strongly imply copy-pasted fetch patterns
	- framework adapters with explicit and documented support boundaries
	- fixture-backed evidence that the same shape is explainable without runtime execution or ORM-specific introspection

### Structural And Design Gap Follow-Ups

1. `too_many_instance_attributes` should move from a single conservative threshold to a tiered policy.

	Future work should explicitly separate:

	- the current broad smell threshold at 10 or more instance attributes
	- a stronger "20 or more instance variables" checkpoint that aligns with the original user wording
	- severity or evidence messaging that reflects whether the class also has many public methods, collaborators, or mixed concerns

2. `monolithic_init_module` should grow into a broader monolithic-module policy only when the repository evidence is ready.

	That work should cover:

	- non-`__init__.py` modules that accumulate imports, declarations, and orchestration in one file
	- thresholds that account for byte size, function count, class count, import count, and top-level executable behavior together
	- negative fixtures for intentionally central registry or schema modules that are large but structured

3. `over_abstracted_wrapper` should be expanded into a clearer "class where a function or dataclass would suffice" checkpoint.

	Future work should require evidence such as:

	- one or two thin pass-through methods
	- little or no retained mutable state
	- constructor parameters that are simply forwarded or stored without behavior
	- no meaningful polymorphism, lifecycle management, or protocol implementation

4. `name_responsibility_mismatch` needs additional anchors before it claims to cover misleading names more broadly.

	Useful expansion points include:

	- functions whose names imply pure transformation while the body performs I/O or persistence
	- classes named as passive records that actually coordinate multiple collaborators
	- helpers whose verbs imply one domain but whose imports and calls show another

	Verb and subject inversion remains too codebase-specific to promote unless the evidence becomes much stronger.

5. Hardcoded business logic, magic values, and reinvented-utility backlog items should only advance through narrow categories.

	Future work should split these items instead of treating them as one bucket:

	- business thresholds, ratios, or status mappings embedded in request or service logic
	- unexplained numeric or string sentinels repeated across branches or files
	- locally implemented parsing, encoding, or serialization helpers that overlap obvious standard-library or already-imported capabilities

### Duplication Follow-Ups

1. Cross-file copy-paste detection needs a repository-level clone policy rather than broader reuse of file-local duplicate summaries.

	The next iteration should define:

	- a normalized function-body fingerprint for non-test code across files
	- minimum size and structure thresholds that avoid trivial helper collisions
	- evidence that explains why two functions are considered near-clones without dumping entire bodies into findings
	- separate handling for test-only duplication versus production duplication

2. Query-fragment duplication should be treated separately from repeated string literals.

	Future work should look for:

	- repeated SQL or ORM fragment literals across files
	- repeated filter, projection, or ordering shapes embedded in code rather than centralized helpers
	- evidence that clusters fragments by normalized tokens instead of raw string equality alone

3. Duplicate transformation pipelines need an explicit representation before they can be shipped honestly.

	The likely implementation checkpoint is a normalized sequence for common stages such as parse, validate, map, filter, enrich, and serialize so that highly similar data-processing pipelines can be compared across functions and files.

4. Duplicate error-handling should remain narrow unless broader repository-level evidence is added.

	The existing `duplicate_error_handler_block` rule is intentionally local and syntax-shaped. Broader promotion should require repeated wrapper structures across files or services, not just visually similar `try` and `except` blocks.

### Context, Robustness, And AI-Smell Follow-Ups

1. Boilerplate exception wrappers, toy-problem code, and "missing real-world context" should only be promoted around concrete boundary surfaces.

	Candidate anchors include:

	- network, filesystem, subprocess, or environment access with no visible failure handling
	- retry-worthy operations with repeated direct calls and no timeout or retry boundary in surrounding code
	- request, CLI, or job-entry functions that trust inputs entirely and skip obvious validation or fallback paths

	This work should avoid claiming that every simple helper needs logging, retries, or environment logic.

2. Overly descriptive variable names and slightly wrong variable names remain deliberately constrained.

	Future work may revisit these only if the rule can be reduced to objective structural evidence, such as repetitive boilerplate placeholder naming alongside duplicated AI-style scaffolding. Pure taste-based naming review should remain out of scope.

3. "Perfectly balanced tiny functions" and "textbook code with no personality" stay non-goals unless they collapse into another measurable smell.

	These concepts should not be promoted on tone alone. If they ever graduate, it should be because they can be restated as concrete duplication, over-abstraction, or missing-boundary evidence.

## Acceptance Criteria

- [x] The missing Python backlog from `python.md` is explicitly classified instead of being implied.
- [x] The roadmap names concrete advanced rule families for performance, architecture, duplication, AI-smell, and maintainability work.
- [x] Each proposed rule family has a conservative evidence policy and a clear implementation boundary.
- [x] The roadmap explicitly identifies which items require parser enrichment, repository-level evidence, or permanent deferral.
- [x] Phase 4 does not collapse subjective review taste into unsupported findings.

## Verification

- [x] Review this file against the backlog categories in `python.md` and confirm each item has a status.
- [x] Review `guides/python/index.md` and confirm the phase ordering and completion state remain accurate.
- [x] Verify that the required parser or repository-level evidence exists before implementing each shipped Phase 4 rule.
- [x] Require fixture-backed positive and negative coverage before adding each new rule to `guides/features-and-detections.md`.

## Document Update Obligations

- [x] Keep this file updated whenever the advanced Python backlog changes materially.
- [x] Update `guides/python/index.md` whenever a Phase 4 rule family moves from planned to implemented.
- [x] Update `guides/features-and-detections.md` when a Phase 4 rule becomes user-visible.
- [x] Update `guides/implementation-guide.md` if Phase 4 requires new shared analysis or repository-index abstractions.

## Risks And Open Questions

- Many backlog items blend static code smell, runtime performance, and style preference. Without explicit evidence boundaries, this phase could add a lot of noise quickly.
- Duplication detection can become expensive or unreadable if it starts as full clone detection instead of narrow, evidence-rich duplicate-block clustering.
- AI-smell detection is the easiest place to overfit personal taste. The roadmap must preserve explainable, reviewable evidence.
- Repository-level rules can pressure the current index and summary model. If those abstractions change, the design should stay language-scoped and incremental.
- Some items in the source backlog may remain permanent non-goals for static analysis, and the roadmap should be comfortable saying that directly.