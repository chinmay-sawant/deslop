import { useEffect } from 'react'
import { ArrowRightIcon } from '@heroicons/react/24/outline'
import { Link, useLocation } from 'react-router-dom'

import { sitePath } from '../../shared/lib/sitePath'
import { Container } from '../../shared/ui/Container'
import { SectionIntro } from '../../shared/ui/SectionIntro'
import { FeatureGrid } from './components/FeatureGrid'
import { Footer } from './components/Footer'
import { HeroSection } from './components/HeroSection'
import { MetricsBar } from './components/MetricsBar'
import { PipelineTabs } from './components/PipelineTabs'
import { QuickStart } from './components/QuickStart'

export function HomePage() {
  const location = useLocation()

  useEffect(() => {
    if (location.hash) {
      const id = location.hash.substring(1)
      // Small timeout ensures the DOM has completed its layout rendering before scrolling
      setTimeout(() => {
        document.getElementById(id)?.scrollIntoView({ behavior: 'smooth' })
      }, 100)
    }
  }, [location])

  return (
    <div className="relative">
      <main id="main-content">
        {/* 1. Hero */}
        <HeroSection />

        {/* 2. Metrics — quick credibility numbers */}
        <MetricsBar />

        {/* 3. What it finds — detection families */}
        <section className="py-24 sm:py-32 lg:py-40">
          <Container>
            <SectionIntro
              eyebrow="Detection families"
              title="Six rule families covering the patterns that matter."
              description="deslop surfaces suspicious signals across reliability, security, performance, code clarity, coordination, and test quality—each with explainable evidence, not opaque scores."
            />
            <FeatureGrid />
          </Container>
        </section>

        {/* 4. How it works — pipeline stages */}
        <section className="border-t border-[var(--border)] py-24 sm:py-32 lg:py-40">
          <Container>
            <SectionIntro
              eyebrow="How it works"
              title="A four-stage local pipeline built for speed."
              description="Discover, parse, index, and run heuristics—each stage is independently composable and designed to stay fast on real repositories."
            />
            <PipelineTabs />
          </Container>
        </section>

        {/* 5. Install and run — conversion */}
        <section id="install-run" className="section-anchor border-t border-[var(--border)] py-24 sm:py-32 lg:py-40">
          <Container>
            <SectionIntro
              eyebrow="Install and run"
              title="Install deslop with Cargo, release binaries, or GitHub Actions."
              description="Wire deslop into GitHub Actions, install from crates.io, grab a prebuilt binary, or run a scan directly from your repository root."
            />
            <QuickStart />
          </Container>
        </section>

        {/* 6. Closing CTA */}
        <section className="border-t border-[var(--border)] py-24 sm:py-32 lg:py-40">
          <Container>
            <div className="max-w-4xl">
              <span className="eyebrow">Open source from day one</span>
              <h2 className="mt-6 max-w-4xl text-4xl leading-tight font-bold sm:text-6xl">
                Install it fast. Keep the workflow local. Keep the findings readable.
              </h2>
              <p className="mt-5 max-w-3xl text-base leading-8 text-[var(--muted)] sm:text-lg">
                deslop ships as a Rust CLI with explainable static signals and repository-local symbol awareness. It covers Go request-path packs for GORM and Gin, duplicate decode and multipart upload churn, repeated split and strconv work, loop-local URL and time parsing, looped GORM CRUD and DB churn, receiver-wrapper and context propagation checks—plus Cargo installs, prebuilt release binaries, and GitHub Actions support for automation.
              </p>

              <div className="mt-8 flex flex-col gap-3 sm:flex-row">
                <a
                  href={sitePath('#install-run')}
                  className="button-primary"
                  onClick={(e) => {
                    const el = document.getElementById('install-run')
                    if (el) {
                      e.preventDefault()
                      el.scrollIntoView({ behavior: 'smooth' })
                      window.history.pushState(null, '', sitePath('#install-run'))
                    }
                  }}
                >
                  Install and run
                  <ArrowRightIcon className="h-4 w-4" aria-hidden="true" />
                </a>
                <Link to="/docs" className="button-secondary">
                  Browse the categories
                </Link>
              </div>
            </div>
          </Container>
        </section>
      </main>

      <Footer />
    </div>
  )
}