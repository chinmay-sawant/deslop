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

        <section id="features" className="section-anchor py-18 sm:py-22 lg:py-28">
          <Container>
            <SectionIntro
              eyebrow="Detection families"
              title="The homepage should feel broader than the current parser target"
              description="The product can be presented as a static-analysis layer for low-context code while still being honest about the current implementation. These categories keep the framing generic, with implementation detail left to the guides and CLI docs."
            />
            <FeatureGrid />
          </Container>
        </section>

        <section id="pipeline" className="section-anchor py-18 sm:py-22 lg:py-28">
          <Container>
            <SectionIntro
              eyebrow="Pipeline"
              title="A staged pipeline designed for clarity, speed, and future extension"
              description="deslop is still intentionally simple at the product surface: discover, parse, index, and evaluate. That structure makes the implementation feel credible, and it gives the website a cleaner story than dumping every rule on the page."
            />
            <PipelineTabs />
          </Container>
        </section>

        <section id="use-cases" className="section-anchor py-18 sm:py-22 lg:py-28">
          <Container>
            <SectionIntro
              eyebrow="Use cases"
              title="Built for teams that need sharper review signals, not more dashboard noise"
              description="The most credible use cases in the docs are still lightweight ones: code review, local audits, narrow security passes, and automation that benefits from readable output rather than platform ceremony."
            />

            <div className="mt-14 grid gap-5 lg:grid-cols-2">
              {useCases.map((useCase) => (
                <article key={useCase.title} className="glass-panel rounded-[2rem] p-7 sm:p-8">
                  <h3 className="text-[2rem] leading-tight font-bold">{useCase.title}</h3>
                  <p className="mt-4 text-base leading-8 text-[var(--muted)]">{useCase.description}</p>
                  <div className="surface-inset mt-8 rounded-[1.5rem] px-5 py-5 text-sm leading-7 sm:text-base">
                    {useCase.outcome}
                  </div>
                </article>
              ))}
            </div>
          </Container>
        </section>

        <section id="quickstart" className="section-anchor py-18 sm:py-22 lg:py-28">
          <Container>
            <SectionIntro
              eyebrow="Quick start"
              title="The public framing can stay broad while the commands stay exact"
              description="Everything in this section stays tied to the README and implementation guide. That keeps the marketing cleaner while avoiding the usual trap of inventing features the repository does not actually expose."
            />
            <QuickStart />
          </Container>
        </section>

        <section id="principles" className="section-anchor py-18 sm:py-22 lg:py-28">
          <Container className="grid gap-8 lg:grid-cols-[minmax(0,0.8fr)_minmax(0,1.2fr)] lg:items-start xl:gap-10">
            <div>
              <SectionIntro
                eyebrow="Principles"
                title="Less visual noise, fewer promises, stronger positioning"
                description="The docs are already careful about scope and limitations. The site should match that tone: calm, sharp, and comfortable leaving some detail to the guides instead of shouting every capability at once."
              />

              <div className="mt-10 space-y-5">
                {principles.map((principle) => (
                  <article key={principle.title} className="glass-panel rounded-[1.8rem] p-7">
                    <h3 className="text-[1.85rem] leading-tight font-bold">{principle.title}</h3>
                    <p className="mt-4 text-sm leading-8 text-[var(--muted)] sm:text-base">{principle.description}</p>
                  </article>
                ))}
              </div>
            </div>

            <div className="glass-panel rounded-[2.4rem] p-8 sm:p-10 lg:p-11 xl:p-12">
              <p className="eyebrow">Benchmark reference</p>
              <h3 className="mt-6 max-w-none text-[clamp(3rem,4.2vw,4.35rem)] leading-[0.95] font-bold">Enough proof to feel real, without making the page feel crowded.</h3>
              <p className="mt-6 max-w-3xl text-base leading-8 text-[var(--muted)] sm:text-lg">
                The implementation guide documents a preferred baseline against a realistic local Go repository. The numbers belong here as evidence,
                not as a universal promise for every codebase shape.
              </p>

              <div className="mt-10 grid gap-5 md:grid-cols-2 xl:grid-cols-3">
                {metrics.map((metric) => (
                  <article key={metric.label} className="grid-panel rounded-[1.7rem] p-6 sm:p-7">
                    <p className="font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.2em] text-[var(--muted)]">{metric.label}</p>
                    <p className="mt-4 text-2xl font-bold text-[var(--text)]">{metric.value}</p>
                    <p className="mt-3 text-sm leading-7 text-[var(--muted)]">{metric.note}</p>
                  </article>
                ))}
              </div>

              <div className="surface-inset mt-8 rounded-[1.8rem] p-6 sm:p-7">
                <p className="font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.2em] text-[var(--text)]">What this page will not claim</p>
                <p className="mt-4 max-w-2xl text-sm leading-8 sm:text-base">
                  No authoritative Go type checking. No interprocedural proof. No guarantee that every flagged issue is wrong. The value is in the speed,
                  coverage, and clarity of the evidence you get back.
                </p>
              </div>
            </div>
          </Container>
        </section>

        <section className="pb-20 pt-8 sm:pb-24 lg:pb-28">
          <Container>
            <div className="glass-panel rounded-[2.4rem] p-8 sm:p-12 lg:p-14">
              <div className="grid gap-10 lg:grid-cols-[1fr_auto] lg:items-end">
                <div>
                  <span className="eyebrow">Open source from day one</span>
                  <h2 className="mt-6 max-w-4xl text-4xl leading-tight font-bold sm:text-6xl">
                    Keep the workflow local. Keep the findings readable. Keep the homepage restrained.
                  </h2>
                  <p className="mt-5 max-w-3xl text-base leading-8 text-[var(--muted)] sm:text-lg">
                    deslop is already structured for extension, but the current landing page stays faithful to what the repo actually ships today:
                    a Rust CLI for Go repositories with explainable static signals and repeatable benchmarks.
                  </p>
                </div>

                <div className="flex flex-col gap-3 sm:flex-row lg:flex-col">
                  <a href="#quickstart" className="button-primary">
                    Run the commands
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
              A static-analysis product story for low-context code, anchored to a current implementation that targets Go repositories and keeps the evidence readable.
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
              <div key={source} className="grid-panel rounded-[1.6rem] p-5">
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