import {
  cliCommands,
  commonRules,
  goEcosystemSupport,
  githubActionInputs,
  githubActionBenchExample,
  githubActionJsonExample,
  githubActionWorkflow,
  goRules,
  languages,
  limitations,
  overviewContent,
  pipelineStages,
  pythonRules,
  repositoryConfigExample,
  rustRules,
  sections,
  type Language,
  type SectionId,
} from '../docs-content'
import { currentRelease } from '../../../content/site-content'
import { CodeBlock } from './CodeBlock'

interface DocsLayoutProps {
  activeLang: Language
  activeSection: SectionId
  onLangChange: (lang: Language) => void
  onSectionChange: (section: SectionId) => void
}

export function DocsLayout({
  activeLang,
  activeSection,
  onLangChange,
  onSectionChange,
}: DocsLayoutProps) {

  const langClass = `lang-${activeLang}`
  const overview = overviewContent[activeLang]
  const rules = activeLang === 'common' ? commonRules : activeLang === 'go' ? goRules : activeLang === 'python' ? pythonRules : rustRules
  const commands = cliCommands[activeLang]
  const limits = limitations[activeLang]

  const handleLangChange = (lang: Language) => {
    onLangChange(lang)
    onSectionChange('overview')
  }

  const setActiveSection = onSectionChange

  const ruleCounts: Record<Language, number> = {
    go: goRules.length,
    python: pythonRules.length,
    rust: rustRules.length,
    common: commonRules.length,
  }

  return (
    <div className="docs-layout">
      {/* ── Sidebar ─────────────────────────────────────────────────────── */}
      <aside className="docs-sidebar">
        {/* Language tabs */}
        <p className="docs-sidebar-section-label" style={{ marginTop: 0 }}>Language</p>
        <div className="docs-lang-tabs">
          {languages.map((lang) => (
            <button
              key={lang.id}
              className={`docs-lang-tab lang-${lang.id}${activeLang === lang.id ? ' active' : ''}`}
              onClick={() => handleLangChange(lang.id)}
              type="button"
            >
              <span className="docs-lang-tab-dot" />
              <span className="docs-lang-tab-label">{lang.label}</span>
              <span className="docs-lang-tab-count">{ruleCounts[lang.id]}</span>
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
            {activeLang === 'common' && 'Shared heuristics for all supported languages.'}
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

          {activeLang === 'go' && (
            <>
              <h2 className="docs-h2">Third-party coverage</h2>
              <p className="docs-p">
                Go support includes dedicated heuristics for common server and infrastructure stacks without turning the overview into a wall of text.
              </p>
              <div className="docs-pill-list">
                {goEcosystemSupport.map((item) => (
                  <span key={item} className="docs-pill">{item}</span>
                ))}
              </div>
            </>
          )}

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
          <p className="docs-p">Emit JSON, include per-function fingerprints, and keep the workflow green while you evaluate the report:</p>
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
              {githubActionInputs.map((input) => (
                <tr key={input.name}>
                  <td>{input.name}</td>
                  <td>{input.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {/* DETECTION RULES */}
        <div className={`docs-section${activeSection === 'detection-rules' ? ' active' : ''}`}>
          <div className={`docs-eyebrow ${langClass}`}>Detection rules</div>
          <h1 className="docs-h1">
            {activeLang === 'common'
              ? `${rules.length} generic rules.`
              : `${rules.length} rules for ${activeLang === 'go' ? 'Go' : activeLang === 'python' ? 'Python' : 'Rust'}.`}
          </h1>
          <p className="docs-lead">
            Each rule produces a finding with a rule ID, severity, file path, line number, and human-readable evidence.
            Findings are heuristics, not compile-time proof. deslop is conservative where full type information is missing.
          </p>

          <div className="docs-callout" style={{ borderLeftColor: `var(--lang-${activeLang})`, background: `var(--lang-${activeLang}-soft)` }}>
            <p>
              By default, <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem' }}>deslop scan</code> prints the standard finding set.
              Pass <code style={{ fontFamily: 'var(--mono-font)', fontSize: '0.8rem' }}>--details</code> when you want the per-function fingerprint breakdown alongside the normal findings.
            </p>
          </div>

          <h2 className="docs-h2">All {activeLang === 'go' ? 'Go' : activeLang === 'python' ? 'Python' : activeLang === 'rust' ? 'Rust' : 'Common'} rules</h2>
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
              <tr><td>--details</td><td>Include full per-function fingerprint details in scan output.</td></tr>
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
            A local benchmark recorded on April 3, 2026 averaged 1906.20 ms over 5 repeats after 1 warmup against gopdfsuit at 125 files and 876 functions.
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
            {activeLang === 'common' && ['Inter-language FFI/RPC bridge modeling', 'Global AST-normalized similarity indexing', 'Heuristic weight calibration', 'Support for JS/TS and C++'].map((item) => <span key={item} className="docs-pill">{item}</span>)}
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
