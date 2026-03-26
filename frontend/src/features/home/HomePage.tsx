import { useEffect } from 'react'
import { ArrowRightIcon } from '@heroicons/react/24/outline'
import { Link, useLocation } from 'react-router-dom'

import {
  footerLinks,
  footerSources,
} from '../../content/site-content'
import { Container } from '../../shared/ui/Container'
import { SectionIntro } from '../../shared/ui/SectionIntro'
import { HeroSection } from './components/HeroSection'
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
      <main>
        <HeroSection />

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

                <div className="mt-8 flex flex-col gap-3 sm:flex-row lg:flex-row">
                  <a 
                    href="#install-run" 
                    className="button-primary"
                    onClick={(e) => {
                      const el = document.getElementById('install-run')
                      if (el) {
                        e.preventDefault()
                        el.scrollIntoView({ behavior: 'smooth' })
                        window.history.pushState(null, '', '/deslop/#install-run')
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
            </div>
          </Container>
        </section>
      </main>

      <footer className="border-t border-[var(--border)] pb-12 pt-10 sm:pb-14">
        <Container className="grid gap-8 lg:grid-cols-[1.1fr_0.9fr]">
          <div>
            <p className="font-['Newsreader'] italic text-3xl font-bold tracking-[-0.02em] text-[var(--text)]">deslop</p>
            <p className="mt-4 max-w-2xl font-['Newsreader'] italic text-xl leading-8 text-[var(--muted)] sm:text-2xl">
              Static analysis for low-context code, focused on readable findings across Go, Python, and Rust repositories.
            </p>

            <p className="mt-5 font-['IBM_Plex_Mono'] text-xs uppercase tracking-widest text-[var(--text-strong)] sm:text-sm">
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

            <div className="mt-8 flex flex-wrap gap-2">
              {footerLinks.map((link) => (
                <a 
                  key={link.href} 
                  href={link.href} 
                  className="stat-pill rounded-full px-4 py-2 text-sm hover:text-white"
                >
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