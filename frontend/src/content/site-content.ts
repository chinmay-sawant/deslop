import type { ComponentType, SVGProps } from 'react'
import {
  BoltIcon,
  CircleStackIcon,
  CodeBracketSquareIcon,
  CpuChipIcon,
  ExclamationTriangleIcon,
  ShieldCheckIcon,
} from '@heroicons/react/24/outline'

import releaseAssetsData from './release-assets.json'

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

export type ReleaseAsset = {
  id: string
  label: string
  platform: string
  arch: string
  fileName: string
  url: string
}

export type ReleaseAssetManifest = {
  version: string
  releasePage: string
  actionRef: string
  assets: ReleaseAsset[]
}

export type ReleaseCatalogManifest = {
  currentVersion: string
  releases: ReleaseAssetManifest[]
}

export const releaseCatalog = releaseAssetsData as ReleaseCatalogManifest

export const releaseHistory = releaseCatalog.releases

export const currentRelease =
  releaseHistory.find((release) => release.version === releaseCatalog.currentVersion) ?? releaseHistory[0]

export const getReleaseByVersion = (version: string) =>
  releaseHistory.find((release) => release.version === version)

const linuxReleaseAsset =
  currentRelease.assets.find((asset) => asset.id === 'linux-x86_64') ?? currentRelease.assets[0]

export const siteMetadata: SiteMetadata = {
  github: {
    owner: 'chinmay-sawant',
    repo: 'deslop',
    url: 'https://github.com/chinmay-sawant/deslop',
    releaseUrl: currentRelease.releasePage,
  },
  crates: {
    url: 'https://crates.io/crates/deslop',
  },
}

export const navigation: NavItem[] = [
  { label: 'Install and run', href: '#install-run' },
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
    prompt: `curl -L ${linuxReleaseAsset.url} -o ${linuxReleaseAsset.fileName}`,
    output: `Pull the ${currentRelease.version} release asset directly when you want a prebuilt binary instead of a Cargo install.`,
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
      'Surface vague naming, overdescribed helpers, and weakly typed interfaces before they spread.',
    rules: ['Generic naming', 'Overlong identifiers', 'Weak typing'],
    icon: CodeBracketSquareIcon,
  },
  {
    title: 'Reliability',
    description:
      'Catch failure-handling shortcuts that make code look complete while hiding operational risk.',
    rules: ['Dropped errors', 'Panic-first branches', 'Weak wrapping'],
    icon: ExclamationTriangleIcon,
  },
  {
    title: 'Security',
    description:
      'Highlight secrets, weak crypto, and query-construction patterns that deserve a closer look.',
    rules: ['Secret literals', 'Weak crypto', 'Unsafe query strings'],
    icon: ShieldCheckIcon,
  },
  {
    title: 'Coordination',
    description:
      'Flag shutdown, cancellation, and blocking patterns that look harmless until systems are under load.',
    rules: ['Receiver wrapper propagation', 'Derived-context goroutines', 'Busy waiting'],
    icon: BoltIcon,
  },
  {
    title: 'Performance',
    description:
      'Flag repeated work, duplicate decoding, ORM loop waste, and request-path allocation churn before it hardens into defaults.',
    rules: ['Duplicate decode work', 'GORM loop churn', 'Body rewind waste', 'Handler batch gaps'],
    icon: CpuChipIcon,
  },
  {
    title: 'Tests and repo-local context',
    description:
      'Distinguish tests that only gesture at safety from code paths referencing symbols the project cannot resolve.',
    rules: ['Placeholder tests', 'Happy-path-only tests', 'Repo-local symbol misses'],
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
      'Skips vendor and common generated files',
      'Keeps discovery separate from parsing work',
    ],
  },
  {
    name: 'Parse',
    summary: 'Parse source structure, declared symbols, and call patterns without forcing a heavy semantic stack.',
    detail:
      'The current implementation uses tree-sitter-go, tree-sitter-python, and tree-sitter-rust while remaining syntax tolerant. Even if a file is imperfect, deslop still tries to recover enough structure to keep signal flowing into the report.',
    bullets: [
      'Package names, imports, and declared symbols',
      'Call sites plus loop and request-path clues',
      'Function-, class-, and module-level evidence for later heuristics',
    ],
  },
  {
    name: 'Index',
    summary: 'Build a lightweight repository-local symbol index keyed by language plus package or module and directory context.',
    detail:
      'This stage is intentionally modest. It improves same-package, same-module, and import-qualified checks without pretending to replace full semantic analysis or type resolution.',
    bullets: [
      'Functions, methods, and declared symbol counts',
      'Language-scoped matching for mixed repositories',
      'Import context reused by hallucination checks',
    ],
  },
  {
    name: 'Heuristics',
    summary: 'Run explainable rule families that emit rule IDs, severity, messages, and evidence.',
    detail:
      'The heuristics layer is designed for human review rather than opaque scoring. Findings stay readable by default, while the Go semantic pack adds nested-loop correlation when it is enabled for the repository or run.',
    bullets: [
      'Compact text output by default',
      'Optional `--details` adds per-function fingerprints',
      'JSON output for pipeline integration',
      'Optional semantic Go loop correlation when enabled',
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
    label: 'Download release binaries',
    channel: 'Binary',
    description:
      'Choose the latest release by default, or switch back to the previous tag when you want to compare assets or keep an older install path available.',
    snippet: [
      'Select a release tag below.',
      ...releaseHistory.map((release) => `${release.version} -> ${release.assets.length} assets`),
    ],
    linkLabel: 'Open latest GitHub release assets',
    linkHref: siteMetadata.github.releaseUrl,
  },
  {
    label: 'Run it in GitHub Actions',
    channel: 'CI',
    description: 'Use the composite action to download the correct release binary for the runner and execute a scan inside your workflow.',
    snippet: [
      '- uses: actions/checkout@v4',
      `- uses: ${currentRelease.actionRef}`,
      '  with:',
      '    path: .',
    ],
    linkLabel: 'See action usage in the README',
    linkHref: 'https://github.com/chinmay-sawant/deslop#github-action',
  },
  {
    label: 'Scan the current repository',
    channel: 'CLI',
    description: 'Run deslop from the repository root and save a readable report you can review locally or attach to CI output. Add `--enable-semantic` when you want the deeper Go loop heuristics forced on for that run.',
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
    value: '1906.20 ms',
    note: 'Mean total benchmark over 5 repeats after 1 warmup against gopdfsuit on April 3, 2026.',
  },
  {
    label: 'Current implementation',
    value: 'Go, Python, and Rust',
    note: 'The shipped analyzer supports standalone and mixed-language repositories across all three backends.',
  },
  {
    label: 'Benchmark repository scale',
    value: '125 files / 876 functions',
    note: 'Measured as a full-repository pass on the gopdfsuit corpus target.',
  },
]

export const footerLinks: NavItem[] = [
  { label: 'Back to top', href: '#top' },
  { label: 'Documentation', href: '/docs' },
  { label: 'Install and run', href: '#install-run' },
  { label: 'GitHub', href: 'https://github.com/chinmay-sawant/deslop' },
]

export const footerSources = [
  // 'README for command surface and GitHub Action usage',
  // 'GitHub Releases for the published v0.1.0 binaries',
  // 'crates.io for the cargo install path',
]
