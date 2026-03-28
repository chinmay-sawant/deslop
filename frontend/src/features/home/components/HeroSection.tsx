import { useState, useEffect } from 'react'
import { ArrowRightIcon } from '@heroicons/react/24/outline'

import { trustPillars } from '../../../content/site-content'
import { Container } from '../../../shared/ui/Container'

export function HeroSection() {
  const [isReady, setIsReady] = useState(false)

  useEffect(() => {
    // Delay animation start by 3 seconds after mount
    const timer = setTimeout(() => setIsReady(true), 3000)
    return () => clearTimeout(timer)
  }, [])

  return (
    <section id="top" className="section-anchor relative pt-24 pb-24 sm:pt-32 sm:pb-32 lg:pt-40 lg:pb-40">
      <Container className="max-w-5xl mx-auto text-center">
        <span className="eyebrow mx-auto mb-8">Static analysis. Human insight.</span>
        <div className="flex flex-col items-center justify-center text-center w-full">
          <h1 className="mt-6 text-[clamp(2.5rem,7vw,5rem)] leading-[0.95] font-medium tracking-[-0.03em] text-[var(--text-strong)] flex flex-col items-center justify-center w-full relative">
            {/* Line 1 */}
            <div className="relative inline-flex items-center justify-center px-2 mb-2 h-[1.3em]">
              <div className={`absolute inset-x-0 inset-y-0 z-10 pointer-events-none ${isReady ? 'animate-sweep-box' : ''}`} />
              <div className="overflow-hidden h-full">
                <div className={`flex flex-col ${isReady ? 'animate-sweep-text-col' : ''}`}>
                  <span className="h-[1.3em] flex items-center justify-center whitespace-nowrap px-1">Modern problems</span>
                  <span className="h-[1.3em] flex items-center justify-center whitespace-nowrap px-1">Write code for the machine</span>
                  <span className="h-[1.3em] flex items-center justify-center whitespace-nowrap px-1">Works alongside your linters</span>
                </div>
              </div>
            </div>

            {/* Line 2 */}
            <div className="relative inline-flex items-center justify-center px-2 italic text-[var(--muted)] h-[1.3em]">
              <div 
                className={`absolute inset-x-0 inset-y-0 z-10 pointer-events-none ${isReady ? 'animate-sweep-box' : ''}`} 
                style={{ animationDelay: isReady ? '150ms' : '0ms' }} 
              />
              <div className="overflow-hidden h-full">
                <div 
                  className={`flex flex-col ${isReady ? 'animate-sweep-text-col' : ''}`} 
                  style={{ animationDelay: isReady ? '150ms' : '0ms' }}
                >
                  <span className="h-[1.3em] flex items-center justify-center whitespace-nowrap px-1">Requires modern solutions.</span>
                  <span className="h-[1.3em] flex items-center justify-center whitespace-nowrap px-1">Optimize it for the human.</span>
                  <span className="h-[1.3em] flex items-center justify-center whitespace-nowrap px-1">Flagging the context they miss.</span>
                </div>
              </div>
            </div>
          </h1>
          <br/>
          <p className="mt-10 mx-auto w-full max-w-2xl text-[1.15rem] leading-[1.8] text-[var(--muted)] sm:text-[1.35rem] text-center">
            A lightning-fast static analyzer that flags AI-generated code smells across Python, Rust, and Go. It surfaces suspicious patterns with readable evidence, giving engineers absolute confidence when reviewing synthetic code.
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