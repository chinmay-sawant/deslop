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
  description: string
  command: string
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
  }
}

export const siteMetadata: SiteMetadata = {
  github: {
    owner: 'chinmay-sawant',
    repo: 'goslop',
    url: 'https://github.com/chinmay-sawant/goslop',
  },
}

export const navigation: NavItem[] = [
  { label: 'Signals', href: '#features' },
  { label: 'Pipeline', href: '#pipeline' },
  { label: 'Use cases', href: '#use-cases' },
  { label: 'Quick start', href: '#quickstart' },
  { label: 'Principles', href: '#principles' },
]

export const trustPillars = [
  'Explainable findings instead of opaque scoring',
  'Structured output for local workflows and automation',
  'Complements existing lints and linters instead of replacing them',
]

export const terminalFlow = [
  {
    prompt: './deslop scan . > results.txt',
    output: 'Scan the current repository and write a readable report to disk for a fast review loop.',
  },
  {
    prompt: './deslop scan --json . > results.json',
    output: 'Emit structured output when the same findings need to flow into scripts, CI, or dashboards.',
  },
  {
    prompt: './deslop bench --warmups 2 --repeats 5 .',
    output: 'Measure repeatable discovery, parse, index, heuristic, and total timings against the current repo.',
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
      'The current implementation uses tree-sitter-go and remains syntax tolerant. Even if a file is imperfect, deslop still tries to recover enough structure to keep signal flowing into the report.',
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
    label: 'Scan the current repository',
    description: 'Run deslop from the repository root and save a readable text report you can review or share.',
    command: './deslop scan . > results.txt',
  },
  {
    label: 'Export structured output',
    description: 'Use JSON when the same scan needs to flow into scripts, CI, or internal tooling.',
    command: './deslop scan --json . > results.json',
  },
  {
    label: 'Measure the pipeline',
    description: 'Benchmark the full local pass when you want repeatable timings for discovery, parsing, indexing, and heuristics.',
    command: './deslop bench --warmups 2 --repeats 5 .',
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
    value: 'Go repositories first',
    note: 'The public site stays broader, but the shipped analyzer currently targets Go.',
  },
  {
    label: 'Benchmark repository scale',
    value: '89 files / 702 functions',
    note: 'Measured as a full-repository static analysis pass.',
  },
]

export const footerLinks: NavItem[] = [
  { label: 'Back to top', href: '#top' },
  { label: 'Detection families', href: '#features' },
  { label: 'Pipeline', href: '#pipeline' },
  { label: 'Quick start', href: '#quickstart' },
]

export const footerSources = [
  'README for command surface and scan modes',
  'Feature guide for rule families and philosophy',
  'Implementation guide for pipeline and benchmark details',
]