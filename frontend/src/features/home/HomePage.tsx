import { useEffect } from 'react'
import { ArrowRightIcon } from '@heroicons/react/24/outline'
import { Link, useLocation } from 'react-router-dom'

import {
  footerLinks,
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
                    deslop ships as a Rust CLI for Go, Python, and Rust repositories with explainable static signals, repository-local symbol awareness, receiver-wrapper and context propagation checks for Go, readable output, Cargo installs, published binaries, and GitHub Actions support for automation.
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

      <footer className="border-t border-[var(--border)] pb-16 pt-14 sm:pb-20 sm:pt-18">
        <Container>
          {/* Main footer grid */}
          <div className="grid gap-12 lg:grid-cols-[1.5fr_1fr] items-start">
            {/* Left: Brand + description + language badges */}
            <div>
              <p className="font-['Newsreader'] italic text-[2.6rem] font-semibold leading-none tracking-[-0.03em] text-[var(--text-strong)]">
                deslop
              </p>
              <p className="mt-4 max-w-md font-['Newsreader'] italic text-[1.15rem] leading-[1.75] text-[var(--muted)]">
                Static analysis for low-context code, focused on readable, repository-aware findings across Go, Python, and Rust repositories.
              </p>

              {/* Language badges */}
              <div className="mt-6 flex flex-wrap gap-2">
                <span className="px-3 py-1 text-[0.7rem] font-medium font-['IBM_Plex_Mono'] tracking-[0.12em] uppercase border border-[var(--lang-go-badge)] text-[var(--lang-go)] bg-[var(--lang-go-soft)]">
                  Go
                </span>
                <span className="px-3 py-1 text-[0.7rem] font-medium font-['IBM_Plex_Mono'] tracking-[0.12em] uppercase border border-[var(--lang-python-badge)] text-[var(--lang-python)] bg-[var(--lang-python-soft)]">
                  Python
                </span>
                <span className="px-3 py-1 text-[0.7rem] font-medium font-['IBM_Plex_Mono'] tracking-[0.12em] uppercase border border-[var(--lang-rust-badge)] text-[var(--lang-rust)] bg-[var(--lang-rust-soft)]">
                  Rust
                </span>
              </div>
            </div>

            {/* Right: Navigation links */}
            <div className="flex flex-col gap-4 lg:items-end lg:pt-1">
              {footerLinks.map((link) => (
                <a
                  key={link.href}
                  href={link.href}
                  className="group flex items-center gap-2 text-sm text-[var(--muted)] transition-colors duration-150 hover:text-[var(--text-strong)]"
                >
                  <span>{link.label}</span>
                  <span className="inline-block transition-transform duration-150 group-hover:translate-x-0.5 opacity-40 group-hover:opacity-100">→</span>
                </a>
              ))}
            </div>
          </div>

          {/* Bottom bar */}
          <div className="mt-14 border-t border-[var(--border)] pt-8 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <p className="text-xs text-[var(--muted)] font-['IBM_Plex_Mono'] tracking-[0.06em]">
              © 2026 deslop · MIT License · Open-source from day one
            </p>
            <p className="text-sm text-[var(--muted)]">
              Built &amp; vibecoded by{' '}
              <a
                href="https://github.com/chinmay-sawant"
                target="_blank"
                rel="noreferrer"
                className="text-[var(--text)] underline decoration-[var(--border)] underline-offset-4 transition-colors duration-150 hover:decoration-[var(--text)]"
              >
                Chinmay Sawant
              </a>{' '}
              with ❤️
            </p>
          </div>
        </Container>
      </footer>
    </div>
  )
}