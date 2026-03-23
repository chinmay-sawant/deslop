# Phase 4: Advanced Python Rule Families And Backlog Expansion

## Overview

This phase exists because the attached Python backlog in `python.md` is much broader than the currently shipped Python support. Phases 1 through 3 established a conservative backend, parser contract, first rule pack, and rollout baseline. They do not fully cover advanced Python performance smells, architectural and design smells, duplication, AI-generated-code signals, or the larger maintainability backlog.

The goal of this phase is not to promise that every item from the attached notes can be shipped as a low-noise static rule. The goal is to turn the remaining backlog into an explicit implementation contract with clear triage boundaries:

- what can be implemented conservatively with the current architecture
- what needs additional parser or repository-level evidence
- what likely needs a new shared abstraction or cross-file index support
- what should remain deferred because it depends on runtime profiling, framework semantics, or subjective review judgment

This phase should treat the user-supplied backlog as a rule-family roadmap, not as permission to add noisy detectors quickly.

## In Scope

- Expanding the Python roadmap to explicitly cover the missing backlog from `python.md`
- Grouping the missing backlog into rule families with implementable evidence policies
- Defining which advanced Python rules belong in shared heuristics versus `src/heuristics/python/`
- Planning parser and analysis-model extensions required for the next Python rule families
- Planning repository-level evidence needed for duplication and coupling-oriented signals
- Defining a conservative policy for AI-generated-code smell detection so subjective signals do not turn into arbitrary noise
- Defining which backlog items should remain permanently non-goals for static analysis
- Establishing fixture and verification expectations for each new Python rule family

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

	- implementable now with mostly syntactic evidence
	- implementable after modest parser enrichment
	- implementable only after repository-level or cross-file evidence exists
	- intentionally deferred because they need runtime profiling, framework semantics, or reviewer judgment

	This checkpoint is required because the missing backlog spans everything from obvious `== None` misuse to profiling-only query and container-efficiency claims.

2. Expand the Python performance backlog with conservative static policies.

	The current roadmap already covers:

	- `string_concat_in_loop`
	- `blocking_sync_io_in_async`
	- `full_dataset_load`

	The missing performance backlog should be triaged explicitly as follows.

	Good Phase 4 candidates with conservative evidence policies:

	- using `list(...) [0]` or equivalent eager materialization where `next(iter(...))` would avoid building a list
	- repeated temporary list or dict construction inside obvious hot loops
	- recursion on clearly deep traversal helpers when the function shape suggests unbounded nesting risk
	- using list membership checks inside loops when the looked-up value is stable and the container is obviously list-like
	- repeated `len()` calls inside loops only when the receiver is obviously unchanged and the pattern is directly local
	- not using `collections.deque` for repeated queue-style `pop(0)` or `insert(0, ...)` operations

	Candidates that likely need stronger evidence before shipping:

	- list-versus-set or dict lookup recommendations when the value flow is unclear
	- recomputing the same value repeatedly instead of caching
	- temporary-object churn in loops when the loop is not obviously hot
	- overusing `try` and `except` in loops where context may matter

	Candidates that should stay deferred or documentation-only because static analysis cannot justify them honestly:

	- ignoring database indexes in the general case
	- N+1 queries without recognizable query-call evidence beyond the current generic data-access rules
	- pandas-versus-pure-Python recommendations based on dataset size or workload shape
	- overuse of globals as a performance claim rather than a design smell

	Evidence rule:

	- no Phase 4 performance rule should claim a measured regression; messages must stay at the level of likely hot-path risk or avoidable allocation pattern.

3. Define the advanced architecture and design-smell family.

	The current roadmap mentions monolithic `__init__.py`, god functions, and oversized classes only as backlog. This phase should turn them into explicit candidates with evidence thresholds.

	Candidate families:

	- god functions based on line span, branching density, local-name churn, call-site fan-out, and mixed responsibility signals in a single function
	- god classes based on method count, responsibility breadth, and instance-variable fan-out rather than a single raw threshold
	- classes with excessive instance variables using `self.` assignment counts with constructor and method evidence
	- deep inheritance hierarchies when base-class chains are locally visible and repository-scoped
	- monolithic modules including oversized `__init__.py` files, especially when they mix exports, business logic, I/O, and configuration loading
	- hardcoded business rules or magic decision tables that should likely be externalized
	- mixing concerns such as HTTP, persistence, and business logic in one function when the call and import evidence is locally obvious
	- constructors that create unrelated collaborators eagerly inside `__init__`
	- misleading names that mask responsibility only when the mismatch is supported by structural evidence rather than taste alone
	- over-fetching or overly broad data-return shapes only when the query or serialization evidence is visible locally
	- tight coupling between modules if import fan-in and direct constructor usage can be summarized conservatively
	- over-abstraction patterns only when there is a low-noise structural signature, not because a design feels "too abstract"

	Threshold rule:

	- design-smell thresholds must be explicitly documented and justified with fixtures so they do not become arbitrary style preferences.

4. Define the duplication strategy instead of leaving it as a vague non-goal.

	The attached backlog calls out duplication explicitly and the current roadmap does not provide a dedicated plan. This phase should define a duplication strategy with narrow initial scope.

	Recommended initial scope:

	- repeated error-handling blocks with highly similar AST shapes in the same file
	- repeated validation pipelines within the same module or directory
	- repeated string literals or query fragments above a configurable threshold
	- duplicate utility logic split between tests and production code when function fingerprints and token shapes are very close

	Recommended non-goals for the first duplication pass:

	- whole-repository clone detection
	- semantic equivalence across renamed variables or reordered statements
	- cross-language duplication

	Implementation guidance:

	- start from function and block fingerprints already used elsewhere in deslop
	- prefer near-duplicate block clusters over line-based similarity percentages
	- require reviewer-readable evidence such as matched snippet summaries, block counts, and normalized token-shape counts

5. Define an AI-generated-code smell policy with hard false-positive boundaries.

	The current phases mention low-noise AI-smell detection only as backlog. This phase should make the policy explicit.

	Potential candidates with moderate static signal if kept conservative:

	- inconsistent naming conventions within one file, such as mixed `snake_case` and `camelCase`, when the file is otherwise coherent enough to compare local symbols
	- imports from obviously unrelated heavy ecosystems when usage evidence shows most imported modules are unused or irrelevant
	- boilerplate try and except wrappers repeated around operations that do not raise the handled class in any visible way
	- over-commenting of obvious operations when the comment-style heuristics and statement shape strongly agree
	- emojis or highly enthusiastic comments in production code
	- suspiciously textbook docstrings on tiny helpers when docstrings restate the code without adding domain context

	Candidates that should stay deferred unless a very strong evidence policy is defined:

	- overly descriptive variable names by taste alone
	- perfectly balanced function decomposition as a finding by itself
	- "zero personality" or "textbook" structure without a measurable proxy
	- variable names that merely feel slightly off

	Policy rule:

	- an AI-smell finding must always point to a concrete structural signal that would still matter in a non-AI-authored file.

6. Expand the maintainability backlog into explicit rule candidates.

	The current shipped Python rules only cover a subset of the maintainability list. This phase should define the missing maintainability family explicitly.

	Strong static candidates:

	- `== None` and `!= None` comparisons instead of `is None` and `is not None`
	- loop-based reductions or searches that should use obvious built-ins such as `any`, `all`, `sum`, `max`, or `min`
	- list comprehensions used only for side effects
	- explicit `return None` in functions that otherwise just fall through
	- hardcoded file-system paths where `pathlib` or configuration indirection is more appropriate
	- commented-out dead code blocks that look intentionally disabled rather than explanatory comments
	- broad `except Exception:` handlers that suppress or obscure failure beyond the currently shipped swallowed-exception rule
	- missing context-manager usage for files, locks, or other obviously closable resources when the open/close lifecycle is visible locally
	- overuse of `*args` and `**kwargs` on public APIs when signatures become opaque without a strong forwarding reason
	- mixed sync and async boundaries inside one module or function family when call and declaration evidence is strong enough

	Candidates that need careful policy work:

	- public APIs missing type hints, because repository style and Python-version targets vary
	- ignoring virtual environments or `pyproject.toml`, because those are repository-level conventions rather than per-file heuristics

	Compatibility rule:

	- where a maintainability signal is honestly cross-language in meaning, prefer extending shared heuristics instead of duplicating Python-only logic.

7. Define the parser and shared-model extensions required for Phase 4 rule families.

	Potential parser additions:

	- richer assignment-shape capture for `self.` fields, constructor collaborators, and magic constants
	- lightweight loop-body summaries for queue operations, temporary allocations, repeated length checks, and side-effect-only comprehensions
	- richer call-target normalization for file I/O, HTTP, DB, and cache-like APIs
	- class inheritance summaries and local base-resolution when repository-local evidence exists
	- comment and docstring summaries that can distinguish explanatory context from restated code
	- string-literal aggregation for repeated literal and query-fragment detection

	Shared-model rule:

	- do not bloat shared analysis types with Python-only detail unless another language can use the same concept honestly.

8. Define repository-level evidence needs before adding cross-file rules.

	Advanced Python rules that need more than one file should identify that dependency up front.

	Likely repository-level needs:

	- file and module summaries for monolithic module detection
	- symbol-to-module ownership for tight-coupling heuristics
	- normalized duplicate-block fingerprints for duplication rules
	- repeated literal and query-fragment frequency counts across a package or repository slice

	Boundary rule:

	- if a rule needs repository aggregation, that dependency must be explicit in code structure, tests, and documentation rather than hidden inside a heuristic helper.

9. Define rule-family fixture strategy before implementation.

	Required fixture coverage for every Phase 4 rule family:

	- positive and negative fixtures for each rule
	- mixed-rule fixtures that prove signals compose without duplicate spam
	- suppression fixtures for test code, CLI entrypoints, framework glue, and intentionally acceptable patterns
	- false-positive fixtures for library-style APIs, generated-looking but legitimate documentation, and repository conventions that resemble AI or duplication smells

	Duplication-specific verification:

	- verify that near-duplicate blocks produce one understandable finding cluster instead of one finding per repeated line.

10. Produce an explicit status map for every backlog item from the attached notes.

	This phase should end with every backlog item from the user-provided list classified as one of:

	- shipped already in Phases 1 through 3
	- targeted by a concrete Phase 4 rule family
	- deferred pending parser enrichment
	- deferred pending repository-level evidence
	- intentionally non-goal because static analysis would be misleading

	That status map is required so the roadmap can answer "do we cover this idea?" without rereading free-form prose.

## Backlog Status Map

This section maps the user-provided Python backlog to current roadmap status.

### Performance And Unoptimized Code

- using `+` for string concatenation in a loop: already shipped in Phases 1 through 3 as `string_concat_in_loop`
- looping over a list when a set or dict lookup would be more appropriate: Phase 4 target after modest parser enrichment and conservative local-container evidence
- using a list for membership checks instead of a set: Phase 4 target with tight local evidence only
- repeatedly calling `len()` inside a tight loop: Phase 4 target with unchanged-receiver evidence only
- overusing global variables as a performance claim: deferred as a pure performance rule; may be reconsidered later as a design smell instead
- blocking async code with sync I/O: already shipped in Phases 1 through 3 as `blocking_sync_io_in_async`
- using `list(...)[0]` instead of `next(iter(...))`: Phase 4 target
- creating temporary lists or dicts inside hot loops: Phase 4 target when the loop-local allocation pattern is obvious
- using recursion for deep structures: Phase 4 target with conservative shape-based evidence; not as a proof of recursion failure
- not using `collections.deque` for queue-style operations: Phase 4 target
- loading huge datasets into memory instead of streaming: partially shipped already through `full_dataset_load`; further streaming-specific expansion is a Phase 4 target
- using pandas for tiny data or pure Python for huge data: intentional non-goal for static analysis without workload evidence
- recomputing the same value repeatedly instead of caching: deferred pending stronger local data-flow evidence
- overusing `try` and `except` inside loops: deferred pending a low-noise evidence policy; likely Phase 4 only for narrow repeated-wrapper shapes
- ignoring database indexes or writing N+1 queries: N+1-style query work already exists generically in deslop, but Python-specific index-awareness remains deferred and likely framework-specific

### Architectural And Design Smells

- god classes or functions: Phase 4 target
- classes with 20 or more instance variables: Phase 4 target
- using classes where a simple function or dataclass would suffice: Phase 4 target under the over-abstraction family
- deep inheritance hierarchies: Phase 4 target with repository-local base-chain evidence
- monolithic `__init__.py` files or single huge modules: Phase 4 target
- hardcoded business logic instead of configuration: Phase 4 target with careful evidence thresholds
- mixing business logic, HTTP, and DB concerns in one function: Phase 4 target
- creating unrelated objects inside `__init__`: Phase 4 target
- misleading names that hide the real responsibility: Phase 4 target only when supported by structural evidence
- verb and subject reversal in naming such as `process_user` versus `user.process`: deferred because this is often too style- and codebase-dependent
- returning more data than needed: deferred pending stronger call-site and data-shape evidence
- tight coupling between modules: Phase 4 target with repository-level summaries
- magic numbers or strings without constants or enums: Phase 4 target
- reinventing the wheel such as a custom JSON parser instead of established libraries: deferred because library-choice judgment is hard to make honestly with local syntax alone
- over-abstraction patterns where a simple map or helper would do: Phase 4 target when the structural signature is clear

### Code Duplication

- copy-paste functions across files: Phase 4 target with repository-level duplicate fingerprints
- duplicate error-handling blocks repeated many times: Phase 4 target
- same validation logic across endpoints or services: Phase 4 target
- repeated string literals or query fragments: Phase 4 target
- duplicate data-transformation pipelines: Phase 4 target, likely after repository-level duplicate-block summaries exist
- same utility logic in tests and production code: Phase 4 target

### AI-Generated-Code Smells

- overly descriptive variable names: deferred because this is too taste-driven without a stronger evidence policy
- tiny functions with Wikipedia-style docstrings: Phase 4 target
- imports from unrelated ecosystems when only one small capability is used: Phase 4 target
- over-commenting the obvious: Phase 4 target
- perfectly balanced tiny functions that should be merged: intentional non-goal unless a future structural proxy proves low-noise enough
- boilerplate try and except everywhere: Phase 4 target
- emojis or overly enthusiastic comments: Phase 4 target
- structurally flawless code that lacks real-world context such as logging, retries, or environment handling: deferred because absence-of-context is too subjective without framework-aware evidence
- toy-problem solutions that assume perfect input: deferred because robustness expectations vary by project and layer
- inconsistent naming in the same file: Phase 4 target
- unexplained magic numbers like `17` or `2.718`: Phase 4 target
- classes for what should be two lines of code: Phase 4 target under over-abstraction
- variable names that are just slightly off: intentional non-goal without a measurable policy
- feels like textbook code with no personality: intentional non-goal unless it can be reduced to a concrete structural smell

### Maintainability And Readability

- using `== None` instead of `is None`: Phase 4 target
- writing loops instead of clear built-ins such as `any`, `all`, `sum`, or `max`: Phase 4 target
- unnecessary list comprehensions for side effects: Phase 4 target
- returning `None` explicitly in every function: Phase 4 target
- using `eval()` or `exec()`: already shipped in Phases 1 through 3 as `eval_exec_usage`
- hardcoding file paths instead of using `pathlib` plus configuration: Phase 4 target
- ignoring virtual environments and `pyproject.toml`: deferred as a repository-convention check rather than a per-file code smell
- print debugging in production code: already shipped in Phases 1 through 3 as `print_debugging_leftover`
- no type hints on public APIs: Phase 4 target only with configurable or clearly documented thresholds
- overusing `**kwargs` and `*args` to avoid interface design: Phase 4 target
- commenting out dead code instead of deleting it: Phase 4 target
- huge `try`/`except Exception:` blocks with `pass`: partially shipped already through `exception_swallowed`; broader obscuring-exception coverage is a Phase 4 target
- not using context managers for files, locks, or DB connections: Phase 4 target
- mixing sync and async in the same codebase without clear boundaries: Phase 4 target with careful module- and call-shape evidence

## Acceptance Criteria

- The missing Python backlog from `python.md` is explicitly classified instead of being implied.
- The roadmap names concrete advanced rule families for performance, architecture, duplication, AI-smell, and maintainability work.
- Each proposed rule family has a conservative evidence policy and a clear implementation boundary.
- The roadmap explicitly identifies which items require parser enrichment, repository-level evidence, or permanent deferral.
- Phase 4 does not collapse subjective review taste into unsupported findings.

## Verification

- Review this file against the backlog categories in `python.md` and confirm each item has a status.
- Review `guides/python/index.md` and confirm the phase ordering and completion state remain accurate.
- Before implementing any Phase 4 rule, verify that the required parser or repository-level evidence exists or extend the earlier phase contracts first.
- When Phase 4 implementation starts, require fixture-backed positive and negative coverage before adding each new rule to `guides/features-and-detections.md`.

## Document Update Obligations

- Update this file whenever the advanced Python backlog changes materially.
- Update `guides/python/index.md` whenever a Phase 4 rule family moves from planned to implemented.
- Update `guides/features-and-detections.md` only when a Phase 4 rule becomes user-visible.
- Update `guides/implementation-guide.md` if Phase 4 requires new shared analysis or repository-index abstractions.

## Risks And Open Questions

- Many backlog items blend static code smell, runtime performance, and style preference. Without explicit evidence boundaries, this phase could add a lot of noise quickly.
- Duplication detection can become expensive or unreadable if it starts as full clone detection instead of narrow, evidence-rich duplicate-block clustering.
- AI-smell detection is the easiest place to overfit personal taste. The roadmap must preserve explainable, reviewable evidence.
- Repository-level rules can pressure the current index and summary model. If those abstractions change, the design should stay language-scoped and incremental.
- Some items in the source backlog may remain permanent non-goals for static analysis, and the roadmap should be comfortable saying that directly.