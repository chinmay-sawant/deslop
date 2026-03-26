import { ArrowRightIcon } from '@heroicons/react/24/outline'

import {
  footerLinks,
  footerSources,
  metrics,
  principles,
  useCases,
} from '../../content/site-content'
import type { Theme } from '../../shared/lib/useTheme'
import { Container } from '../../shared/ui/Container'
import { SectionIntro } from '../../shared/ui/SectionIntro'
import { FeatureGrid } from './components/FeatureGrid'
import { Header } from './components/Header'
import { HeroSection } from './components/HeroSection'
import { PipelineTabs } from './components/PipelineTabs'
import { QuickStart } from './components/QuickStart'

type HomePageProps = {
  theme: Theme
  onToggleTheme: () => void
}

export function HomePage({ theme, onToggleTheme }: HomePageProps) {
  return (
    <div className="relative">
      <Header theme={theme} onToggleTheme={onToggleTheme} />

      <main>
        <HeroSection />

        <section id="features" className="section-anchor py-24 sm:py-32 lg:py-40">
          <Container>
            <SectionIntro
              eyebrow="Detection families"
              title="Signals across clarity, reliability, and risk."
              description="deslop groups findings into readable families so teams can scan naming, error handling, security, performance, and thin tests without digging through opaque scoring."
            />
            <FeatureGrid />
          </Container>
        </section>

        <section id="pipeline" className="section-anchor py-24 sm:py-32 lg:py-40">
          <Container>
            <SectionIntro
              eyebrow="Pipeline"
              title="A local analysis pipeline built for speed and readable output."
              description="deslop discovers files, parses structure, builds a lightweight index, and runs explainable heuristics so scans stay quick and review output stays useful."
            />
            <PipelineTabs />
          </Container>
        </section>

        <section id="use-cases" className="section-anchor py-24 sm:py-32 lg:py-40">
          <Container>
            <SectionIntro
              eyebrow="Use cases"
              title="Built for teams that need sharper review signals, not more dashboard noise."
              description="The most credible use cases in the docs are still lightweight ones: code review, local audits, narrow security passes, and automation that benefits from readable output rather than platform ceremony."
            />

            <div className="mt-20 flex flex-col gap-24">
              {useCases.map((useCase) => (
                <article key={useCase.title} className="max-w-3xl">
                  <h3 className="text-[2.2rem] leading-tight font-medium italic text-[var(--text-strong)]">{useCase.title}</h3>
                  <p className="mt-6 text-lg leading-relaxed text-[var(--muted)]">{useCase.description}</p>
                  <div className="mt-8 text-base leading-relaxed text-[var(--text-strong)] border-l-2 border-[var(--text-strong)] pl-5 py-1">
                    {useCase.outcome}
                  </div>
                </article>
              ))}
            </div>
          </Container>
        </section>

        <section id="install-run" className="section-anchor py-18 sm:py-22 lg:py-28">
          <Container>
            <SectionIntro
              eyebrow="Install and run"
              title="Install deslop with Cargo, release binaries, or GitHub Actions"
              description="Wire deslop into GitHub Actions, install from crates.io, grab a prebuilt binary, or run a scan directly from your repository root."
            />
            <QuickStart />
          </Container>
        </section>

        <section id="principles" className="section-anchor py-24 sm:py-32 lg:py-40">
          <Container className="grid gap-16 lg:grid-cols-[minmax(0,0.85fr)_minmax(0,1.15fr)] lg:items-start xl:gap-24">
            <div>
              <SectionIntro
                eyebrow="Principles"
                title="Less visual noise, fewer promises, stronger positioning."
                description="deslop favors readable evidence, repository-local context, and conservative signals so teams can review findings quickly and make the final call with confidence."
              />

              <div className="mt-16 space-y-16">
                {principles.map((principle) => (
                  <article key={principle.title} className="max-w-xl">
                    <h3 className="text-[2rem] leading-tight font-medium italic">{principle.title}</h3>
                    <p className="mt-5 text-base leading-relaxed text-[var(--muted)] sm:text-lg">{principle.description}</p>
                  </article>
                ))}
              </div>
            </div>

            <div className="py-8 sm:py-10">
              <p className="eyebrow text-[var(--accent)]">Founder Note</p>
              <h3 className="mt-6 max-w-none text-[clamp(2.5rem,3.8vw,3.5rem)] leading-tight font-medium">Why we built this internally.</h3>
              <p className="mt-6 max-w-2xl text-lg leading-relaxed text-[var(--muted)]">
                The tooling wasn’t capturing the messy reality of code written under pressure or produced by rapid AI prompting. We needed signals that developers could read without a manual, grounded directly in the local repository context.
              </p>
              <p className="mt-4 max-w-2xl text-lg leading-relaxed text-[var(--muted)]">
                The implementation guide includes a representative Go repository baseline so teams can understand scan cost and coverage at practical project scale.
              </p>

              <div className="mt-16 grid gap-10 md:grid-cols-2 xl:grid-cols-3">
                {metrics.map((metric) => (
                  
                  <article key={metric.label} className="border-l border-[var(--border-strong)] pl-6">
                    
                    <p className="font-['IBM_Plex_Mono'] text-[0.7rem] uppercase tracking-[0.2em] text-[var(--muted)]">{metric.label}</p>
                    <p className="mt-5 text-[1.65rem] font-medium text-[var(--text-strong)]">{metric.value}</p>
                    <p className="mt-4 text-[0.95rem] leading-relaxed text-[var(--muted)]">{metric.note}</p>
                  </article>
                ))}
              </div>

            </div>
          </Container>
        </section>

        <section className="py-24 sm:py-32 lg:py-40">
          <Container>
            <div className="py-12 sm:py-16">
              <div className="max-w-4xl">
                <div>
                  <span className="eyebrow">Open source from day one</span>
                  <h2 className="mt-6 max-w-4xl text-4xl leading-tight font-bold sm:text-6xl">
                    Install it fast. Keep the workflow local. Keep the findings readable.
                  </h2>
                  <p className="mt-5 max-w-3xl text-base leading-8 text-[var(--muted)] sm:text-lg">
                    deslop ships as a Rust CLI for Go repositories with explainable static signals, readable output, Cargo installs, published binaries, and GitHub Actions support for automation.
                  </p>
                </div>

                <div className="flex flex-col gap-3 sm:flex-row lg:flex-col">
                  <a href="#install-run" className="button-primary">
                    Install and run
                    <ArrowRightIcon className="h-4 w-4" aria-hidden="true" />
                  </a>
                  <a href="#features" className="button-secondary">
                    Browse the categories
                  </a>
                </div>
              </div>
            </div>
          </Container>
        </section>
      </main>

      <footer className="border-t border-[var(--border)] pb-12 pt-10 sm:pb-14">
        <Container className="grid gap-8 lg:grid-cols-[1.1fr_0.9fr]">
          <div>
            <p className="font-['Space_Grotesk'] text-2xl font-bold tracking-[-0.05em] text-[var(--text)]">deslop</p>
            <p className="mt-4 max-w-2xl text-sm leading-8 text-[var(--muted)] sm:text-base">
              Static analysis for low-context code, focused on readable findings across Go repositories today.
            </p>

            <p className="mt-5 text-sm leading-7 text-[var(--text-strong)] sm:text-base">
             Built & Vibecoded by{' '}
              <a
                href="https://github.com/chinmay-sawant/deslop"
                target="_blank"
                rel="noreferrer"
                className="font-medium text-[var(--text)] underline decoration-[var(--border)] underline-offset-4 transition hover:decoration-[var(--text)]"
              >
                Chinmay Sawant
              </a>{' '}
              with ❤️
            </p>

            <div className="mt-6 flex flex-wrap gap-2">
              {footerLinks.map((link) => (
                <a key={link.href} href={link.href} className="stat-pill rounded-full px-4 py-2 text-sm hover:text-white">
                  {link.label}
                </a>
              ))}
            </div>
          </div>

          <div className="grid gap-4 sm:grid-cols-3">
            {footerSources.map((source) => (
              <div key={source} className="grid-panel p-5">
                <p className="font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.18em] text-[var(--text)]">Source</p>
                <p className="mt-3 text-sm leading-7 text-[var(--text-strong)]">{source}</p>
              </div>
            ))}
          </div>
        </Container>
      </footer>
    </div>
  )
}