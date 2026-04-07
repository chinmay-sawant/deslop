import { useState } from 'react'
import { Tab, TabGroup, TabList, TabPanel, TabPanels } from '@headlessui/react'

import {
  currentRelease,
  getReleaseByVersion,
  quickStartItems,
  releaseHistory,
  siteMetadata,
} from '../../../content/site-content'

// Display order: GitHub Actions → crates.io → Binary → Scan
const TAB_ORDER = [2, 0, 1, 3]

export function QuickStart() {
  const orderedItems = TAB_ORDER.map((i) => quickStartItems[i])
  const [selectedReleaseVersion, setSelectedReleaseVersion] = useState(currentRelease.version)
  const selectedBinaryRelease = getReleaseByVersion(selectedReleaseVersion) ?? currentRelease

  return (
    <TabGroup defaultIndex={0} className="mt-14">
      <TabList className="flex flex-wrap gap-3">
        {orderedItems.map((item, idx) => (
          <Tab
            key={item.label}
            className="cursor-pointer border-b border-transparent px-2 py-4 text-xl sm:text-2xl font-['Newsreader'] italic font-medium text-[var(--muted)] transition data-[hover]:text-[var(--text)] data-[selected]:border-[var(--text-strong)] data-[selected]:text-[var(--text-strong)]"
          >
            <span className="font-['IBM_Plex_Mono'] text-[0.68rem] uppercase tracking-[0.16em] not-italic">
              {String(idx + 1).padStart(2, '0')}
            </span>
            <span className="ml-2">{item.label}</span>
          </Tab>
        ))}
      </TabList>
      <TabPanels className="mt-8">
        {/* Tabs 0–2: standard layout */}
        {orderedItems.slice(0, 3).map((item) => (
          <TabPanel key={item.label} className="py-12 lg:py-16">
            {item.channel === 'Binary' ? (
              <div className="grid gap-10 lg:grid-cols-[1.15fr_0.85fr] lg:items-start">
                <div>
                  <span className="eyebrow">{item.channel}</span>
                  <h3 className="mt-6 text-4xl leading-tight font-bold sm:text-[3rem]">{item.label}</h3>
                  <p className="mt-5 max-w-2xl text-base leading-8 text-[var(--muted)] sm:text-lg">
                    {item.description}
                  </p>
                  <div className="release-selector mt-8">
                    <label className="release-selector-label" htmlFor="binary-release-version">
                      Release tag
                    </label>
                    <select
                      id="binary-release-version"
                      className="release-select"
                      value={selectedReleaseVersion}
                      onChange={(event) => setSelectedReleaseVersion(event.target.value)}
                    >
                      {releaseHistory.map((release) => (
                        <option key={release.version} value={release.version}>
                          {release.version}
                        </option>
                      ))}
                    </select>
                    <p className="release-selector-note">
                      Latest defaults to {currentRelease.version}. Keep {releaseHistory[1]?.version ?? 'the previous tag'} available when you want rollback-friendly downloads or a side-by-side comparison.
                    </p>
                  </div>
                  <a
                    href={selectedBinaryRelease.releasePage}
                    target="_blank"
                    rel="noreferrer"
                    className="button-secondary mt-8 inline-flex"
                  >
                    Open {selectedBinaryRelease.version} release
                  </a>
                </div>

                <div className="pt-8 lg:pt-0 lg:border-l lg:border-[var(--border)] lg:pl-16">
                  <p className="font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.2em] text-[var(--muted)]">
                    {selectedBinaryRelease.version} assets
                  </p>
                  <div className="mt-6 grid gap-3">
                    {selectedBinaryRelease.assets.map((asset) => (
                      <a
                        key={`${selectedBinaryRelease.version}-${asset.id}`}
                        href={asset.url}
                        target="_blank"
                        rel="noreferrer"
                        className="block border border-[var(--border)] bg-[var(--accent-soft)] px-4 py-4 transition hover:border-[var(--border-strong)] hover:bg-[color-mix(in_srgb,var(--accent-soft)_70%,var(--bg-elevated))]"
                      >
                        <p className="font-['IBM_Plex_Mono'] text-[0.68rem] uppercase tracking-[0.16em] text-[var(--muted)]">
                          {asset.label}
                        </p>
                        <p className="mt-2 font-['IBM_Plex_Mono'] text-[0.8rem] leading-6 text-[var(--text)] break-all">
                          {asset.fileName}
                        </p>
                      </a>
                    ))}
                  </div>
                </div>
              </div>
            ) : (
              <div className="grid gap-10 lg:grid-cols-[1.15fr_0.85fr] lg:items-start">
                <div>
                  <span className="eyebrow">{item.channel}</span>
                  <h3 className="mt-6 text-4xl leading-tight font-bold sm:text-[3rem]">{item.label}</h3>
                  <p className="mt-5 max-w-2xl text-base leading-8 text-[var(--muted)] sm:text-lg">
                    {item.description}
                  </p>
                  {item.linkHref && (
                    <a
                      href={item.linkHref}
                      target="_blank"
                      rel="noreferrer"
                      className="button-secondary mt-8 inline-flex"
                    >
                      {item.linkLabel}
                    </a>
                  )}
                </div>

                <div className="pt-8 lg:pt-0 lg:border-l lg:border-[var(--border)] lg:pl-16">
                  <p className="font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.2em] text-[var(--muted)]">
                    {item.channel} setup
                  </p>
                  <div className="mt-6">
                    {item.showPrompt && item.snippet.length === 1 ? (
                      <div className="terminal-line font-['IBM_Plex_Mono'] text-[0.82rem] leading-7">
                        <span className="terminal-prompt">$</span>
                        <span className="terminal-copy break-all">{item.snippet[0]}</span>
                      </div>
                    ) : (
                      <pre className="overflow-x-auto whitespace-pre-wrap break-words font-['IBM_Plex_Mono'] text-[0.78rem] leading-7 text-[var(--text)]">
                        {item.snippet.join('\n')}
                      </pre>
                    )}
                  </div>
                </div>
              </div>
            )}
          </TabPanel>
        ))}

        {/* Tab 3: Scan — shows scan command + install quick-refs */}
        {(() => {
          const scanItem = orderedItems[3]
          const cratesItem = quickStartItems[0]
          return (
            <TabPanel key={scanItem.label} className="py-12 lg:py-16">
              <div className="grid gap-16 xl:gap-24 lg:grid-cols-[1.1fr_0.9fr] lg:items-start">
                {/* Left: description */}
                <div>
                  <span className="eyebrow">{scanItem.channel}</span>
                  <h3 className="mt-6 text-4xl leading-tight font-bold sm:text-[3rem]">{scanItem.label}</h3>
                  <p className="mt-5 max-w-2xl text-base leading-8 text-[var(--muted)] sm:text-lg">
                    {scanItem.description}
                  </p>
                  <p className="mt-4 text-sm leading-7 text-[var(--muted)]">
                    Make sure deslop is installed first — install via Cargo or grab a prebuilt binary.
                  </p>
                </div>

                {/* Right: scan command + install quick-refs */}
                <div className="flex flex-col gap-4">
                  {/* Scan command */}
                  <div className="border-t border-[var(--border-strong)] pt-6">
                    <p className="font-['IBM_Plex_Mono'] text-xs uppercase tracking-[0.2em] text-[var(--muted)]">
                      Scan command
                    </p>
                    <div className="terminal-line mt-5 font-['IBM_Plex_Mono'] text-[0.82rem] leading-7">
                      <span className="terminal-prompt">$</span>
                      <span className="terminal-copy">{scanItem.snippet[0]}</span>
                    </div>
                  </div>

                  {/* Install quick-refs */}
                  <div className="grid grid-cols-2 gap-3">
                    <div className="border-t border-[var(--border)] pt-4">
                      <p className="font-['IBM_Plex_Mono'] text-[0.66rem] uppercase tracking-[0.16em] text-[var(--muted)]">
                        Via Cargo
                      </p>
                      <div className="terminal-line mt-3 font-['IBM_Plex_Mono'] text-[0.72rem] leading-6">
                        <span className="terminal-prompt">$</span>
                        <span className="terminal-copy break-all">{cratesItem.snippet[0]}</span>
                      </div>
                    </div>
                    <div className="border-t border-[var(--border)] pt-4">
                      <p className="font-['IBM_Plex_Mono'] text-[0.66rem] uppercase tracking-[0.16em] text-[var(--muted)]">
                        Via Binary
                      </p>
                      <a
                        href={siteMetadata.github.releaseUrl}
                        target="_blank"
                        rel="noreferrer"
                        className="mt-3 block font-['IBM_Plex_Mono'] text-[0.72rem] leading-6 text-[var(--text)] underline underline-offset-2 transition hover:text-[var(--accent-strong)]"
                      >
                        {currentRelease.version} release assets →
                      </a>
                      <p className="mt-1 text-[0.68rem] leading-5 text-[var(--muted)]">
                        {currentRelease.assets.slice(0, 2).map((asset) => asset.fileName).join(', ')}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </TabPanel>
          )
        })()}
      </TabPanels>
    </TabGroup>
  )
}