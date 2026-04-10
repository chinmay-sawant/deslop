import { currentRelease } from '../../content/site-content'

// ─── Types ────────────────────────────────────────────────────────────────────

type Language = 'go' | 'python' | 'rust' | 'common'
type SectionId =
  | 'overview'
  | 'detection-rules'
  | 'cli-commands'
  | 'pipeline'
  | 'limitations'
  | 'why-this-exists'
  | 'about'

interface NavSection {
  id: SectionId
  label: string
  icon: string
}

interface CliCommand {
  cmd: string
  desc: string
}

interface GitHubActionInput {
  name: string
  description: string
}

// ─── Static Data ──────────────────────────────────────────────────────────────

const languages: { id: Language; label: string }[] = [
  { id: 'go', label: 'Go' },
  { id: 'python', label: 'Python' },
  { id: 'rust', label: 'Rust' },
  { id: 'common', label: 'Common' },
]

const sections: NavSection[] = [
  { id: 'overview', label: 'Overview', icon: '◈' },
  { id: 'detection-rules', label: 'Detection Rules', icon: '⊹' },
  { id: 'cli-commands', label: 'CLI Commands', icon: '❯' },
  { id: 'pipeline', label: 'Pipeline', icon: '◎' },
  { id: 'limitations', label: 'Limitations', icon: '△' },
  { id: 'why-this-exists', label: 'Why This Exists', icon: '✦' },
  { id: 'about', label: 'About', icon: '♡' },
]

// ─── Content Data ────────────────────────────────────────────────────────────

// GENERATED_RULES_START
// Rule catalog now lives in generated JSON under ./generated/rule-manifest.json
// and ./generated/rules/<language>/<family>.json.
// GENERATED_RULES_END

// ─── CLI commands by language ─────────────────────────────────────────────────

const cliCommands: Record<Language, CliCommand[]> = {
  go: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Scan a Go repository and print a compact finding summary.' },
    { cmd: 'cargo run -- scan --details /path/to/repo', desc: 'Include full per-function fingerprint details.' },
    { cmd: 'cargo run -- scan --json /path/to/repo', desc: 'Emit structured JSON output for pipeline integration.' },
    { cmd: 'cargo run -- scan --json --details /path/to/repo', desc: 'Combine JSON output with full per-function fingerprints.' },
    { cmd: 'cargo run -- scan --enable-semantic /path/to/repo', desc: 'Force the deeper semantic Go pack on for nested-loop allocation, concat, and stronger N+1 correlation.' },
    { cmd: 'cargo run -- scan --ignore dropped_error,panic_on_error /path/to/repo', desc: 'Ignore selected Go rule IDs for one run without changing repository config.' },
    { cmd: 'cargo run -- scan /path/to/repo > results.txt', desc: 'Write the text report directly to a file.' },
    { cmd: 'cargo run -- scan --no-ignore /path/to/repo', desc: 'Scan without .gitignore filtering.' },
    { cmd: 'cargo run -- bench /path/to/repo', desc: 'Benchmark the full pipeline against a local repository.' },
    { cmd: 'cargo run -- bench --enable-semantic /path/to/repo', desc: 'Benchmark the Go pipeline with the deeper semantic pack forced on.' },
    { cmd: 'cargo run -- bench --warmups 2 --repeats 5 /path/to/repo', desc: 'Benchmark with explicit warmup and repeat counts.' },
    { cmd: 'cargo run -- bench --json /path/to/repo', desc: 'Emit benchmarking data as JSON.' },
  ],
  python: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Auto-detect and scan Python files alongside any Go or Rust files in the repository.' },
    { cmd: 'cargo run -- scan --details /path/to/repo', desc: 'Include full Python per-function fingerprint breakdown.' },
    { cmd: 'cargo run -- scan --json /path/to/repo', desc: 'Emit findings for Python files in structured JSON.' },
    { cmd: 'cargo run -- scan --json --details /path/to/repo', desc: 'Combine JSON output with full Python per-function fingerprints.' },
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
    { cmd: 'cargo run -- scan --json --details /path/to/repo', desc: 'Combine JSON output with full Rust per-function fingerprints.' },
    { cmd: 'cargo run -- scan --ignore rust_async_std_mutex_await,rust_lock_across_await /path/to/repo', desc: 'Ignore specific rule IDs for one scan invocation without changing repository config.' },
    { cmd: 'cargo run -- scan /path/to/repo > results.txt', desc: 'Save the Rust scan report to a file.' },
    { cmd: 'cargo run -- scan --no-ignore /path/to/repo', desc: 'Override .gitignore filtering when scanning Rust projects.' },
    { cmd: 'cargo run -- bench /path/to/repo', desc: 'Benchmark discovery, parse, index, heuristic, and total runtime stages.' },
    { cmd: 'cargo run -- bench --warmups 2 --repeats 5 /path/to/repo', desc: 'Benchmark with explicit warmup and repeat counts.' },
    { cmd: 'cargo run -- bench --json /path/to/repo', desc: 'Emit benchmarking data as JSON.' },
  ],
  common: [
    { cmd: 'cargo run -- scan /path/to/repo', desc: 'Scan any supported repository to trigger the shared heuristic layer.' },
    { cmd: 'cargo run -- scan --ignore hardcoded_secret,generic_name /path/to/repo', desc: 'Ignore specific shared rules for the current scan.' },
  ],
}

// GENERATED_ACTION_INPUTS_START
const githubActionInputs: GitHubActionInput[] = [
  { name: 'version', description: 'Release tag to install, for example v0.1.0. Defaults to the current action ref when it is a full release tag, otherwise latest. Optional.' },
  { name: 'command', description: 'Subcommand to run. Supported values are scan and bench. Defaults to scan. Optional.' },
  { name: 'path', description: 'Path to the repository to analyze. Defaults to .. Optional.' },
  { name: 'json', description: 'Emit JSON output. Defaults to false. Optional.' },
  { name: 'details', description: 'Include full per-function fingerprint details in scan output. Applies only to the scan command. Defaults to false. Optional.' },
  { name: 'no-ignore', description: 'Scan without respecting .gitignore. Defaults to false. Optional.' },
  { name: 'enable-semantic', description: 'Enable the opt-in deeper semantic Go heuristics. Defaults to false. Optional.' },
  { name: 'fail-on-findings', description: 'Exit with a non-zero status code when scan findings are present. Applies only to the scan command. Defaults to true. Optional.' },
  { name: 'repeats', description: 'Benchmark repeat count. Applies only to the bench command. Defaults to 5. Optional.' },
  { name: 'warmups', description: 'Benchmark warmup count. Applies only to the bench command. Defaults to 1. Optional.' },
]
// GENERATED_ACTION_INPUTS_END

// GENERATED_ACTION_EXAMPLES_START
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
// GENERATED_ACTION_EXAMPLES_END

const repositoryConfigExample = `go_semantic_experimental = true
rust_async_experimental = true
disabled_rules = ["panic_macro_leftover"]
suppressed_paths = ["tests/fixtures"]

[severity_overrides]
unwrap_in_non_test_code = "error"
missing_context_propagation = "error"`

const overviewContent = {
  go: {
    title: 'Go Analysis',
    lead: 'Go support is the broadest deslop surface: repository-aware parsing, a lightweight local index, and explainable rules for concurrency, request paths, performance, security, and repo-local structure.',
    bullets: [
      '.gitignore-aware repository walk',
      'tree-sitter-go parsing plus a local package index',
      'Explainable Go rules for concurrency, request paths, performance, and security',
      'Mixed-language support alongside Python and Rust',
    ],
  },
  python: {
    title: 'Python Analysis',
    lead: 'Python support covers async misuse, framework hot paths, maintainability smells, and repository-local hallucination checks with the same explainable output style.',
    bullets: [
      'tree-sitter-python parsing and local symbol extraction',
      'Framework, async, and maintainability heuristics',
      'Language-scoped index for repository-local hallucination checks',
      'Works in mixed-language repositories',
    ],
  },
  rust: {
    title: 'Rust Analysis',
    lead: 'Rust support focuses on hygiene leftovers, crate-local hallucination checks, async/runtime hazards, performance smells, and unsafe-soundness hot spots.',
    bullets: [
      'tree-sitter-rust parsing and crate-local indexing',
      'Async, unsafe, performance, and domain-modeling heuristics',
      'Repository-local hallucination checks for Rust imports',
      'Works in mixed-language repositories',
    ],
  },
  common: {
    title: 'Shared Heuristics',
    lead: 'deslop includes a shared layer of cross-language heuristics for naming quality, commentary hygiene, repository-local hallucination checks, and a small set of common anti-patterns.',
    bullets: [
      'Naming quality checks',
      'Repository-local hallucination detection',
      'Commentary and test hygiene signals',
      'A small shared security/performance layer',
    ],
  },
}

const goEcosystemSupport = [
  'Gin request paths',
  'GORM and SQL access',
  'Redis',
  'gRPC',
  'Structured logging',
  'Config and env access',
  'Prometheus',
  'AWS SDK',
  'Cobra',
]

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
    summary: 'Run explainable rule families that emit rule IDs, severity, messages, and evidence.',
    detail: 'Each finding includes a rule ID, severity level, file path, line number, and an evidence payload written for human review. The --details flag adds full per-function fingerprint breakdowns. JSON output is available for pipeline integration.',
  },
]

const limitations = {
  go: [
    'No authoritative Go type checking. Heuristics use structural patterns, not go/types.',
    'No full interprocedural or type-aware context propagation. Wrapper-chain reasoning stays repository-local and conservative.',
    'No proof of goroutine leaks, N+1 queries, or runtime performance regressions — only pattern signals.',
    'Package-method and local-symbol checks are repository-local; external packages are not indexed.',
    'The deeper semantic Go pack is still heuristic: it correlates nested-loop structure but does not prove asymptotic complexity or schema-aware DB cost.',
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
  common: [
    'Shared heuristics are strictly syntax-driven and do not perform deep interprocedural data-flow or points-to analysis.',
    'No cross-language semantic bridge (e.g., deslop does not model Go-to-Python FFI calling conventions).',
    'Naming and commentary hygiene checks are suggestive and do not account for project-specific jargon or acronyms.',
    'Hallucination checks are strictly repository-local and scoped by language to prevent false-positive cross-pollination.',
    'General performance signals like unbuffered I/O are pattern-based and do not account for OS-level buffering or hardware-specific optimizations.',
  ],
}

export {
  cliCommands,
  goEcosystemSupport,
  githubActionInputs,
  githubActionBenchExample,
  githubActionJsonExample,
  githubActionWorkflow,
  languages,
  limitations,
  overviewContent,
  pipelineStages,
  repositoryConfigExample,
  sections,
}

export type { CliCommand, GitHubActionInput, Language, NavSection, SectionId }
