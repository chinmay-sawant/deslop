import { ArrowRightIcon } from '@heroicons/react/24/outline'

import { trustPillars } from '../../../content/site-content'
import { Container } from '../../../shared/ui/Container'

export function HeroSection() {
  return (
    <section id="top" className="section-anchor relative pt-24 pb-24 sm:pt-32 sm:pb-32 lg:pt-40 lg:pb-40">
      <Container className="max-w-5xl mx-auto text-center">
        <span className="eyebrow mx-auto mb-8">Static analysis. Human insight.</span>
        <div className="flex flex-col items-center justify-center text-center w-full">
          <h1 className="mt-6 text-[clamp(3rem,8vw,5.5rem)] leading-[0.95] font-medium tracking-[-0.03em] text-[var(--text-strong)] flex flex-col items-center justify-center w-full relative">
            {/* Line 1 */}
            <div className="relative inline-flex items-center justify-center overflow-hidden px-2 py-1 mb-2">
              <div className="absolute inset-x-0 inset-y-1 z-10 animate-sweep-box pointer-events-none" />
              <div className="absolute inset-0 flex items-center justify-center animate-sweep-text1 px-2 whitespace-nowrap">
                <span>Code moves fast.</span>
              </div>
              <div className="absolute inset-0 flex items-center justify-center animate-sweep-text2 px-2 whitespace-nowrap" aria-hidden="true">
                <span>Bridging the gap between</span>
              </div>
              <div className="opacity-0 pointer-events-none select-none px-2 whitespace-nowrap">
                <span>Bridging the gap between</span>
              </div>
            </div>

            {/* Line 2 */}
            <div className="relative inline-flex items-center justify-center overflow-hidden px-2 py-1 italic text-[var(--muted)]">
              <div className="absolute inset-x-0 inset-y-1 z-10 animate-sweep-box pointer-events-none" style={{ animationDelay: '150ms' }} />
              <div className="absolute inset-0 flex items-center justify-center animate-sweep-text1 px-2 whitespace-nowrap" style={{ animationDelay: '150ms' }}>
                <span>Context is left behind.</span>
              </div>
              <div className="absolute inset-0 flex items-center justify-center animate-sweep-text2 px-2 whitespace-nowrap" aria-hidden="true" style={{ animationDelay: '150ms' }}>
                <span>speed and clarity.</span>
              </div>
              <div className="opacity-0 pointer-events-none select-none px-2 whitespace-nowrap">
                <span>Context is left behind.</span>
              </div>
            </div>
          </h1>
          <p className="mt-10 mx-auto w-full max-w-2xl text-[1.15rem] leading-[1.8] text-[var(--muted)] sm:text-[1.35rem] text-center">
            Deslop helps teams surface patterns that show up when code is generated quickly and reviewed late: vague naming, brittle error handling, and thin tests. A tool built by engineers feeling the pain of synthetic code.
          </p>
        </div>

        <div className="mt-14 flex flex-col items-center justify-center gap-5 sm:flex-row">
          <a href="#install-run" className="button-primary px-8">
            Install and run
            <ArrowRightIcon className="h-4 w-4" aria-hidden="true" />
          </a>
          <a
            href="https://github.com/chinmay-sawant/deslop/releases/tag/v0.1.0"
            target="_blank"
            rel="noreferrer"
            className="button-secondary px-8"
          >
            Get v0.1.0 binaries
          </a>
        </div>

        <div className="mt-20 pt-16 border-t border-[var(--border)] max-w-4xl mx-auto grid gap-10 sm:grid-cols-3 text-left">
          {trustPillars.map((pillar) => (
            <div key={pillar} className="text-base leading-relaxed text-[var(--muted)] border-l border-[var(--border-strong)] pl-5">
              {pillar}
            </div>
          ))}
        </div>
      </Container>
    </section>
  )
}