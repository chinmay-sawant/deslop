import type { ComponentType, SVGProps } from 'react'
import {
  BoltIcon,
  CircleStackIcon,
  CodeBracketSquareIcon,
  CpuChipIcon,
  ExclamationTriangleIcon,
  ShieldCheckIcon,
} from '@heroicons/react/24/outline'

type IconType = ComponentType<SVGProps<SVGSVGElement>>

export type NavItem = {
  label: string
  href: string
}

export type DetectionFamily = {
  title: string
  description: string
  rules: string[]
  icon: IconType
}

export type PipelineStage = {
  name: string
  summary: string
  detail: string
  bullets: string[]
}

export type UseCase = {
  title: string
  description: string
  outcome: string
}

export type QuickStartItem = {
  label: string
  channel: string
  description: string
  snippet: string[]
  showPrompt?: boolean
  linkLabel?: string
  linkHref?: string
}

export type Principle = {
  title: string
  description: string
}

export type Metric = {
  label: string
  value: string
  note: string
}

export type SiteMetadata = {
  github: {
    owner: string
    repo: string
    url: string
    releaseUrl: string
  }
  crates: {
    url: string
  }
}

export const siteMetadata: SiteMetadata = {
  github: {
    owner: 'chinmay-sawant',
    repo: 'deslop',
    url: 'https://github.com/chinmay-sawant/deslop',
    releaseUrl: 'https://github.com/chinmay-sawant/deslop/releases/tag/v0.1.0',
  },
  crates: {
    url: 'https://crates.io/crates/deslop',
  },
}

export const navigation: NavItem[] = [
  { label: 'Install and run', href: '/#install-run' },
  { label: 'Documentation', href: '/docs' },
]

export const trustPillars = [
  'Explainable findings instead of opaque scoring',
  'Structured output for local workflows and automation',
  'Installs through Cargo, prebuilt release binaries, or GitHub Actions',
]

export const terminalFlow = [
  {
    prompt: 'cargo install deslop',
    output: 'Install the CLI directly from crates.io when you want the fastest local setup path.',
  },
  {
    prompt: 'curl -L https://github.com/chinmay-sawant/deslop/releases/download/v0.1.0/deslop-linux-x86_64.tar.gz -o deslop-linux-x86_64.tar.gz',
    output: 'Pull the v0.1.0 release asset directly when you want a prebuilt binary instead of a Cargo install.',
  },
  {
    prompt: 'deslop scan . > results.txt',
    output: 'Run a repository scan locally and keep the report readable enough for fast human review.',
  },
]

export const detectionFamilies: DetectionFamily[] = [
  {
    title: 'Code clarity',
    description:
      'Surface vague naming, overdescribed helpers, and weakly signaled interfaces before they spread through a codebase.',
    rules: ['Generic naming', 'Overlong identifiers', 'Weak typing'],
    icon: CodeBracketSquareIcon,
  },
  {
    title: 'Reliability',
    description:
      'Catch the failure-handling shortcuts that make code look complete while hiding operational risk.',
    rules: ['Dropped errors', 'Panic-first branches', 'Weak wrapping'],
    icon: ExclamationTriangleIcon,
  },
  {
    title: 'Security',
    description:
      'Highlight secrets, weak crypto choices, and query-construction patterns that deserve a closer review.',
    rules: ['Secret literals', 'Weak crypto', 'Unsafe query strings'],
    icon: ShieldCheckIcon,
  },
  {
    title: 'Coordination',
    description:
      'Find shutdown, cancellation, and blocking decisions that often look harmless until systems are under load.',
    rules: ['Missing context', 'Missing cancel', 'Busy waiting'],
    icon: BoltIcon,
  },
  {
    title: 'Performance',
    description:
      'Flag repeated work inside loops, full-payload reads, and formatting-heavy hot paths before they harden into defaults.',
    rules: ['Allocation churn', 'Formatting hot paths', 'Full data loads'],
    icon: CpuChipIcon,
  },
  {
    title: 'Tests and local context',
    description:
      'Differentiate between tests that only gesture at safety and local code paths that appear to reference symbols the project cannot resolve.',
    rules: ['Placeholder tests', 'Happy-path-only tests', 'Local call misses'],
    icon: CircleStackIcon,
  },
]

export const pipelineStages: PipelineStage[] = [
  {
    name: 'Discover',
    summary: 'Walk the repository quickly, with normal developer ignore rules respected by default.',
    detail:
      'deslop starts with file selection only. It keeps discovery independent from later analysis so the pipeline stays composable and cheap to run.',
    bullets: [
      '.gitignore-aware by default',
      'Skips vendor and common generated files in the current implementation',
      'Keeps discovery separate from parsing',
    ],
  },
  {
    name: 'Parse',
    summary: 'Parse source structure, declared symbols, and call patterns without forcing a heavy semantic stack.',
    detail:
      'The current implementation uses tree-sitter-go, tree-sitter-python, and tree-sitter-rust while remaining syntax tolerant. Even if a file is imperfect, deslop still tries to recover enough structure to keep signal flowing into the report.',
    bullets: [
      'Package names, imports, and declared symbols',
      'Call sites, loop markers, and context clues',
      'Function-level fingerprints built for later heuristics',
    ],
  },
  {
    name: 'Index',
    summary: 'Build a lightweight repository-local symbol index keyed by package and directory context.',
    detail:
      'This stage is intentionally modest. It improves local selector and same-package checks without pretending to replace full Go type analysis.',
    bullets: [
      'Functions, methods, and declared symbol counts',
      'Package-plus-directory matching to reduce ambiguity',
      'Import context reused by hallucination heuristics',
    ],
  },
  {
    name: 'Heuristics',
    summary: 'Run explainable rule families that emit rule IDs, severity, messages, and evidence.',
    detail:
      'The heuristics layer is designed for human review rather than opaque scoring. Findings stay readable and conservative where deeper semantic proof does not exist yet.',
    bullets: [
      'Compact text output by default, details opt in',
      'JSON output for pipeline integration',
      'Readable evidence payloads instead of black-box scores',
    ],
  },
]

export const useCases: UseCase[] = [
  {
    title: 'Review AI-assisted changes',
    description:
      'Use deslop as a second pass when code looks plausible but lacks the domain texture, failure handling, or test intent you would expect from a mature change.',
    outcome: 'Shortens review time by surfacing the suspicious shapes first.',
  },
  {
    title: 'Run focused quality sweeps',
    description:
      'Use the tool as a narrow scanner for brittle error handling, thin tests, and structure that feels generated rather than grounded in the problem.',
    outcome: 'Gives teams a fast quality pass without a heavy platform rollout.',
  },
  {
    title: 'Add lightweight security review',
    description:
      'Weak crypto, secret literals, and string-built query paths are called out as explainable findings that can feed human security review.',
    outcome: 'Adds a narrow security lens without pretending to be a full audit suite.',
  },
  {
    title: 'Feed internal automation',
    description:
      'The CLI surface already supports JSON output and benchmarking, so the tool can sit in CI or local tooling without a database or background service.',
    outcome: 'Keeps adoption cheap for teams that prefer simple workflows.',
  },
]

export const quickStartItems: QuickStartItem[] = [
  {
    label: 'Install from crates.io',
    channel: 'Package',
    description: 'Use Cargo when you want the standard Rust CLI install flow and a clean upgrade path from crates.io.',
    snippet: ['cargo install deslop'],
    showPrompt: true,
    linkLabel: 'View crates.io package',
    linkHref: siteMetadata.crates.url,
  },
  {
    label: 'Download the v0.1.0 binaries',
    channel: 'Binary',
    description: 'Grab the already published Linux, macOS, or Windows release asset when you want a prebuilt binary immediately.',
    snippet: [
      'v0.1.0 release assets',
      'deslop-linux-x86_64.tar.gz',
      'deslop-macos-arm64.tar.gz',
      'deslop-macos-x86_64.tar.gz',
      'deslop-windows-x86_64.zip',
    ],
    linkLabel: 'Open GitHub release assets',
    linkHref: siteMetadata.github.releaseUrl,
  },
  {
    label: 'Run it in GitHub Actions',
    channel: 'CI',
    description: 'Use the composite action to download the correct release binary for the runner and execute a scan inside your workflow.',
    snippet: [
      '- uses: actions/checkout@v4',
      '- uses: chinmay-sawant/deslop@v0.1.0',
      '  with:',
      '    path: .',
    ],
    linkLabel: 'See action usage in the README',
    linkHref: 'https://github.com/chinmay-sawant/deslop#github-action',
  },
  {
    label: 'Scan the current repository',
    channel: 'CLI',
    description: 'Run deslop from the repository root and save a readable report you can review locally or attach to CI output.',
    snippet: ['deslop scan . > results.txt'],
    showPrompt: true,
  },
]

export const principles: Principle[] = [
  {
    title: 'Heuristics, not proof',
    description:
      'deslop is intentionally explicit about what it can and cannot prove. It surfaces suspicious patterns quickly and leaves the final judgment to engineers.',
  },
  {
    title: 'Repository-local context first',
    description:
      'The current index and hallucination checks stay local to the scanned repository. That keeps the tool fast and honest about its scope.',
  },
  {
    title: 'Readable evidence over black-box scoring',
    description:
      'Each finding is meant to be legible in a code review workflow: rule ID, message, severity, and the evidence needed to validate it.',
  },
]

export const metrics: Metric[] = [
  {
    label: 'Documented baseline',
    value: '180.80 ms',
    note: 'Preferred local benchmark documented against a realistic Go repository.',
  },
  {
    label: 'Current implementation',
    value: 'Go, Python, and Rust',
    note: 'The shipped analyzer supports standalone and mixed-language repositories across all three backends.',
  },
  {
    label: 'Benchmark repository scale',
    value: '89 files / 702 functions',
    note: 'Measured as a full-repository static analysis pass.',
  },
]

export const footerLinks: NavItem[] = [
  { label: 'Back to top', href: '/#top' },
  { label: 'Documentation', href: '/docs' },
  { label: 'Install and run', href: '/#install-run' },
  { label: 'GitHub', href: 'https://github.com/chinmay-sawant/deslop' },
]

export const footerSources = [
  // 'README for command surface and GitHub Action usage',
  // 'GitHub Releases for the published v0.1.0 binaries',
  // 'crates.io for the cargo install path',
]
