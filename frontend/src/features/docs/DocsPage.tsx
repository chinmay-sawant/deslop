import { useState } from 'react'

import { principles, useCases } from '../../content/site-content'
import { Container } from '../../shared/ui/Container'
import { SectionIntro } from '../../shared/ui/SectionIntro'
import { FeatureGrid } from '../home/components/FeatureGrid'
import { PipelineTabs } from '../home/components/PipelineTabs'

const categories = [
  { id: 'overview', name: 'Overview' },
  { id: 'pipeline', name: 'Pipeline' },
  { id: 'use-cases', name: 'Use Cases' },
  { id: 'principles', name: 'Principles' },
]

export function DocsPage() {
  const [activeTab, setActiveTab] = useState('overview')

  return (
    <Container className="max-w-7xl mx-auto mt-4 flex flex-col md:flex-row gap-8 py-10 md:py-16">
      {/* Sidebar Drawer */}
      <aside className="w-full md:w-64 shrink-0">
        <nav className="sticky top-28 flex flex-col gap-2 bg-[var(--bg)]">
          <h3 className="text-xs font-semibold uppercase tracking-widest text-[var(--muted)] mb-3 px-3">
            Documentation
          </h3>
          {categories.map((cat) => (
            <button
              key={cat.id}
              onClick={() => setActiveTab(cat.id)}
              className={`text-left px-3 py-2.5 rounded-lg transition-colors duration-150 ${
                activeTab === cat.id
                  ? 'bg-[var(--accent-soft)] text-[var(--text-strong)] font-medium'
                  : 'text-[var(--muted)] hover:text-[var(--text)] hover:bg-[var(--accent-soft)]'
              }`}
            >
              {cat.name}
            </button>
          ))}
        </nav>
      </aside>

      {/* Content Area */}
      <main className="flex-1 min-w-0 pb-20">
        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
          {activeTab === 'overview' && (
            <div className="space-y-16">
              <SectionIntro
                eyebrow="Detection families"
                title="Signals across clarity, reliability, and risk."
                description="deslop groups findings into readable families so teams can scan naming, error handling, security, performance, and thin tests without digging through opaque scoring."
              />
              <FeatureGrid />
            </div>
          )}

          {activeTab === 'pipeline' && (
            <div className="space-y-16">
              <SectionIntro
                eyebrow="Pipeline"
                title="A local analysis pipeline built for speed and readable output."
                description="deslop discovers files, parses structure, builds a lightweight index, and runs explainable heuristics so scans stay quick and review output stays useful."
              />
              <PipelineTabs />
            </div>
          )}

          {activeTab === 'use-cases' && (
            <div className="space-y-16">
              <SectionIntro
                eyebrow="Use cases"
                title="Built for teams that need sharper review signals, not more dashboard noise."
                description="The most credible use cases in the docs are still lightweight ones: code review, local audits, narrow security passes, and automation that benefits from readable output rather than platform ceremony."
              />
              <div className="flex flex-col gap-16">
                {useCases.map((useCase) => (
                  <article key={useCase.title} className="max-w-3xl">
                    <h3 className="text-[2rem] leading-tight font-medium italic text-[var(--text-strong)]">
                      {useCase.title}
                    </h3>
                    <p className="mt-6 text-lg leading-relaxed text-[var(--muted)]">
                      {useCase.description}
                    </p>
                    <div className="mt-8 text-base leading-relaxed text-[var(--text-strong)] border-l-2 border-[var(--text-strong)] pl-5 py-1">
                      {useCase.outcome}
                    </div>
                  </article>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'principles' && (
            <div className="space-y-16">
              <SectionIntro
                eyebrow="Principles"
                title="Less visual noise, fewer promises, stronger positioning."
                description="deslop favors readable evidence, repository-local context, and conservative signals so teams can review findings quickly and make the final call with confidence."
              />
              <div className="mt-12 space-y-16">
                {principles.map((principle) => (
                  <article key={principle.title} className="max-w-xl">
                    <h3 className="text-[2rem] leading-tight font-medium italic">
                      {principle.title}
                    </h3>
                    <p className="mt-5 text-base leading-relaxed text-[var(--muted)] sm:text-lg">
                      {principle.description}
                    </p>
                  </article>
                ))}
              </div>
            </div>
          )}
        </div>
      </main>
    </Container>
  )
}
