import { Tab, TabGroup, TabList, TabPanel, TabPanels } from '@headlessui/react'

import { pipelineStages } from '../../../content/site-content'

export function PipelineTabs() {
  return (
    <TabGroup className="mt-14">
      <TabList className="flex flex-wrap gap-3">
        {pipelineStages.map((stage, index) => (
          <Tab
            key={stage.name}
            className="cursor-pointer border-b border-transparent px-2 py-4 text-xl sm:text-2xl font-['Newsreader'] italic font-medium text-[var(--muted)] transition data-[hover]:text-[var(--text)] data-[selected]:border-[var(--text-strong)] data-[selected]:text-[var(--text-strong)]"
          >
            <span className="mr-3 text-sm not-italic font-['IBM_Plex_Mono'] uppercase tracking-[0.16em] opacity-60">0{index + 1}</span>
            <span>{stage.name}</span>
          </Tab>
        ))}
      </TabList>

      <TabPanels className="mt-8">
        {pipelineStages.map((stage) => (
          <TabPanel key={stage.name} className="py-12 lg:py-16">
            <div className="grid gap-16 xl:gap-24 lg:grid-cols-[1.1fr_0.9fr]">
              <div>
                <span className="font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.2em] text-[var(--muted)]">{stage.name} Phase</span>
                <h3 className="mt-8 max-w-3xl text-4xl leading-[1.1] font-medium sm:text-[3.5rem] text-[var(--text-strong)]">{stage.summary}</h3>
                <p className="mt-8 max-w-2xl text-lg leading-relaxed text-[var(--muted)]">{stage.detail}</p>
              </div>

              <div className="pt-8 lg:pt-0 lg:border-l lg:border-[var(--border)] lg:pl-16">
                <p className="font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.2em] text-[var(--muted)]">Stage details</p>
                <br />
                <ul className="mt-8 space-y-6">
                  {stage.bullets.map((bullet) => (
                    <li key={bullet} className="flex items-start gap-4 text-base leading-relaxed text-[var(--text-strong)] sm:text-lg">
                      <span className="mt-2.5 h-1.5 w-1.5 rounded-full bg-[var(--text-strong)]" aria-hidden="true" />
                      <span>{bullet}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </TabPanel>
        ))}
      </TabPanels>
    </TabGroup>
  )
}