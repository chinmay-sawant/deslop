import { ArrowRightIcon } from '@heroicons/react/24/outline'

import { terminalFlow, trustPillars } from '../../../content/site-content'
import { Container } from '../../../shared/ui/Container'

export function HeroSection() {
  return (
    <section id="top" className="section-anchor relative pt-14 pb-16 sm:pt-18 sm:pb-20 lg:pt-20 lg:pb-24">
      <Container className="relative grid items-start gap-10 lg:grid-cols-[minmax(0,1.02fr)_minmax(0,0.98fr)] lg:gap-16 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)] xl:gap-18">
        <div>
          <span className="eyebrow">Static analysis, without the noise</span>
          <h1 className="mt-6 max-w-[10.5ch] text-[clamp(3.3rem,6.8vw,4.95rem)] leading-[0.93] font-bold tracking-[-0.065em]">
            An easier way to review low-context code.
          </h1>
          <p className="mt-7 max-w-3xl text-lg leading-8 text-[var(--muted)] sm:text-xl">
            Deslop helps teams surface the patterns that show up when code is generated quickly and reviewed late: vague naming, brittle error handling, security smells, and thin tests. We are launching with deep support for Go today, with a roadmap built to extend that same review lens across the wider polyglot stack.
          </p>

          <div className="mt-10 flex flex-col gap-3 sm:flex-row">
            <a href="#quickstart" className="button-primary">
              View quick start
              <ArrowRightIcon className="h-4 w-4" aria-hidden="true" />
            </a>
            <a href="#pipeline" className="button-secondary">
              See the pipeline
            </a>
          </div>

          <div className="mt-10 grid max-w-4xl gap-4 md:grid-cols-3">
            {trustPillars.map((pillar) => (
              <div key={pillar} className="surface-inset rounded-[1.6rem] px-5 py-5 text-sm leading-7">
                {pillar}
              </div>
            ))}
          </div>
        </div>

        <div className="glass-panel rounded-[2.2rem] p-6 sm:p-7 xl:p-8">

          <div className="mt-6 space-y-4 text-sm sm:text-[0.95rem]">
            {terminalFlow.map((item) => (
              <div key={item.prompt} className="surface-inset space-y-3 rounded-[1.7rem] p-5 sm:p-6">
                <div className="terminal-line font-['IBM_Plex_Mono'] text-[0.82rem] sm:text-[0.9rem]">
                  <span className="terminal-prompt">$</span>
                  <span className="terminal-copy break-words">{item.prompt}</span>
                </div>
                <p className="pl-6 leading-7 text-[var(--muted)]">{item.output}</p>
              </div>
            ))}
          </div>

          <div className="mt-7 grid gap-4 border-t border-[var(--border)] pt-6 lg:grid-cols-2 2xl:grid-cols-3">
            <div className="surface-inset rounded-[1.45rem] p-5">
              <p className="font-['IBM_Plex_Mono'] text-[0.7rem] uppercase tracking-[0.18em] text-[var(--muted)]">Scope</p>
              <p className="mt-2 text-sm leading-7 text-[var(--text-strong)]">Today the analyzer targets Go repositories and their local project context.</p>
            </div>
            <div className="surface-inset rounded-[1.45rem] p-5">
              <p className="font-['IBM_Plex_Mono'] text-[0.7rem] uppercase tracking-[0.18em] text-[var(--muted)]">Output</p>
              <p className="mt-2 text-sm leading-7 text-[var(--text-strong)]">Readable findings first, detailed output only when you ask for it.</p>
            </div>
            <div className="surface-inset rounded-[1.45rem] p-5 lg:col-span-2 2xl:col-span-1">
              <p className="font-['IBM_Plex_Mono'] text-[0.7rem] uppercase tracking-[0.18em] text-[var(--muted)]">Intent</p>
              <p className="mt-2 text-sm leading-7 text-[var(--text-strong)]">More useful review signals, not a fake promise of perfect proof.</p>
            </div>
          </div>
        </div>
      </Container>
    </section>
  )
}