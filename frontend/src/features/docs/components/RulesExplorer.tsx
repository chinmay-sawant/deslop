import { startTransition, useDeferredValue, useEffect, useState } from 'react'
import { ArrowPathIcon, MagnifyingGlassIcon, SparklesIcon } from '@heroicons/react/24/outline'

import type { Language } from '../docs-content'
import {
  formatConfigurability,
  formatRuleSeverity,
  formatRuleStatus,
  loadRuleFamily,
  ruleCatalog,
  type RuleFamilyChunk,
} from '../rule-catalog'

interface RulesExplorerProps {
  activeLang: Language
}

const ALL_FAMILY_ID = '__all__'

interface BrowserRule {
  id: string
  label: string
  defaultSeverity: 'contextual' | 'error' | 'info' | 'warning'
  status: 'experimental' | 'research' | 'stable'
  familyId: string
  familyLabel: string
}

export function RulesExplorer({ activeLang }: RulesExplorerProps) {
  const languageCatalog = ruleCatalog.languages[activeLang]
  const families = languageCatalog.families
  const allRules: BrowserRule[] = families.flatMap((family) =>
    family.rules.map((rule) => ({
      ...rule,
      familyId: family.id,
      familyLabel: family.label,
    })),
  )
  const firstAllRuleId = allRules[0]?.id ?? ''

  const [activeFamilyId, setActiveFamilyId] = useState(ALL_FAMILY_ID)
  const [selectedRuleId, setSelectedRuleId] = useState(firstAllRuleId)
  const [query, setQuery] = useState('')
  const deferredQuery = useDeferredValue(query)
  const [familyCache, setFamilyCache] = useState<Record<string, RuleFamilyChunk>>({})
  const [loadingKey, setLoadingKey] = useState<string | null>(null)
  const [loadError, setLoadError] = useState<string | null>(null)

  // Reset state when language changes (using startTransition to avoid cascading renders)
  useEffect(() => {
    startTransition(() => {
      setActiveFamilyId(ALL_FAMILY_ID)
      setSelectedRuleId(firstAllRuleId)
      setQuery('')
      setLoadError(null)
    })
  }, [activeLang, firstAllRuleId])

  const activeFamily =
    activeFamilyId === ALL_FAMILY_ID
      ? null
      : families.find((family) => family.id === activeFamilyId) ?? families[0] ?? null
  const normalizedQuery = deferredQuery.trim().toLowerCase()
  const rulePool: BrowserRule[] = activeFamily
    ? activeFamily.rules.map((rule) => ({
        ...rule,
        familyId: activeFamily.id,
        familyLabel: activeFamily.label,
      }))
    : allRules
  const filteredRules = rulePool.filter((rule) => {
    if (!normalizedQuery) {
      return true
    }

    const searchTarget = `${rule.label} ${rule.id} ${rule.familyLabel}`.toLowerCase()
    return searchTarget.includes(normalizedQuery)
  })

  // Derive valid selectedRuleId - ensure it's in filteredRules
  const validSelectedRuleId = (() => {
    if (!filteredRules.length) {
      return ''
    }
    if (filteredRules.some((rule) => rule.id === selectedRuleId)) {
      return selectedRuleId
    }
    return filteredRules[0].id
  })()

  // Sync selectedRuleId when derived value changes
  useEffect(() => {
    if (validSelectedRuleId !== selectedRuleId) {
      startTransition(() => {
        setSelectedRuleId(validSelectedRuleId)
      })
    }
  }, [validSelectedRuleId, selectedRuleId])

  const selectedRuleSummary = filteredRules.find((rule) => rule.id === validSelectedRuleId) ?? filteredRules[0] ?? null
  const detailCacheKey = selectedRuleSummary ? `${activeLang}/${selectedRuleSummary.familyId}` : ''
  const activeChunk = detailCacheKey ? familyCache[detailCacheKey] ?? null : null

  useEffect(() => {
    if (!selectedRuleSummary || activeChunk) {
      return
    }

    let cancelled = false

    // Start loading asynchronously
    Promise.resolve().then(() => {
      if (cancelled) {
        return
      }
      setLoadingKey(detailCacheKey)
      setLoadError(null)
    })

    loadRuleFamily(activeLang, selectedRuleSummary.familyId)
      .then((chunk) => {
        if (cancelled) {
          return
        }

        setFamilyCache((current) => ({
          ...current,
          [detailCacheKey]: chunk,
        }))
      })
      .catch((error: unknown) => {
        if (cancelled) {
          return
        }

        setLoadError(error instanceof Error ? error.message : 'Unable to load rule explanations.')
      })
      .finally(() => {
        if (cancelled) {
          return
        }

        setLoadingKey((current) => (current === detailCacheKey ? null : current))
      })

    return () => {
      cancelled = true
    }
  }, [activeChunk, activeLang, detailCacheKey, selectedRuleSummary])

  const selectedRule = selectedRuleSummary
    ? activeChunk?.rules.find((rule) => rule.id === selectedRuleSummary.id) ?? null
    : null
  const isLoading = loadingKey === detailCacheKey && !activeChunk

  const handleFamilySelect = (familyId: string, nextSelectedRuleId: string) => {
    startTransition(() => {
      setActiveFamilyId(familyId)
      setSelectedRuleId(nextSelectedRuleId)
      setQuery('')
      setLoadError(null)
    })
  }

  const listTitle = activeFamily ? activeFamily.label : 'All families'
  const listSummary = activeFamily
    ? activeFamily.summary
    : `Every ${activeLang} rule in one view. Use search when you know the rule name, or narrow to a family when you want a tighter slice.`
  const listCountLabel = normalizedQuery
    ? `${filteredRules.length} shown`
    : `${activeFamily ? activeFamily.ruleCount : languageCatalog.ruleCount} rules`

  return (
    <section className="rules-explorer">
      <div className="rules-explorer-toolbar">
        <div>
          <p className="rules-explorer-kicker">Grouped by language family</p>
          <p className="docs-p rules-explorer-copy">
            Use All for the full catalog or narrow to a family, then click a rule to read the explanation and fix guidance.
          </p>
        </div>

        <label className="rules-search" htmlFor="rule-search">
          <MagnifyingGlassIcon className="h-4 w-4" aria-hidden="true" />
          <input
            id="rule-search"
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={activeFamily ? `Filter ${activeFamily.label} rules` : 'Filter all rules'}
          />
        </label>
      </div>

      <div className="rules-family-strip" aria-label={`${activeLang} rule families`}>
        <button
          type="button"
          className={`rules-family-chip${activeFamilyId === ALL_FAMILY_ID ? ` active lang-${activeLang}` : ''}`}
          onClick={() => handleFamilySelect(ALL_FAMILY_ID, firstAllRuleId)}
        >
          <span className="rules-family-chip-label">All</span>
          <span className="rules-family-chip-count">{languageCatalog.ruleCount}</span>
        </button>

        {families.map((family) => (
          <button
            key={family.id}
            type="button"
            className={`rules-family-chip${family.id === activeFamily?.id ? ` active lang-${activeLang}` : ''}`}
            onClick={() => handleFamilySelect(family.id, family.rules[0]?.id ?? '')}
          >
            <span className="rules-family-chip-label">{family.label}</span>
            <span className="rules-family-chip-count">{family.ruleCount}</span>
          </button>
        ))}
      </div>

      <div className="rules-browser">
        <div
          className={`rules-browser-panel rules-browser-list${filteredRules.length ? '' : ' rules-browser-list-empty'}`}
        >
          {activeFamily || activeFamilyId === ALL_FAMILY_ID ? (
            <>
              <div className="rules-panel-head">
                <div>
                  <p className="rules-panel-label">{listTitle}</p>
                  <p className="rules-panel-summary">{listSummary}</p>
                </div>
                <span className="rules-panel-count">{listCountLabel}</span>
              </div>

              {filteredRules.length ? (
                <div className="rules-list" role="list">
                  {filteredRules.map((rule) => (
                    <button
                      key={rule.id}
                      type="button"
                      className={`rules-list-item${rule.id === selectedRuleId ? ' active' : ''}`}
                      onClick={() => setSelectedRuleId(rule.id)}
                    >
                      <span className="rules-list-title">{rule.label}</span>
                      <span className="rules-list-meta">
                        <code>{rule.id}</code>
                        {!activeFamily && <span>{rule.familyLabel}</span>}
                        <span>{formatRuleSeverity(rule.defaultSeverity)}</span>
                        {rule.status !== 'stable' && <span>{formatRuleStatus(rule.status)}</span>}
                      </span>
                    </button>
                  ))}
                </div>
              ) : (
                <div className="rules-empty-state">
                  <p>No rules matched that filter.</p>
                  <p>Try a shorter word or switch to another family.</p>
                </div>
              )}
            </>
          ) : (
            <div className="rules-empty-state">
              <p>No rules are available for this language yet.</p>
            </div>
          )}
        </div>

        <div className="rules-browser-panel rules-browser-detail">
          {loadError ? (
            <div className="rules-detail-state">
              <p className="rules-detail-state-title">Unable to load this explanation pack.</p>
              <p className="rules-detail-state-copy">{loadError}</p>
            </div>
          ) : isLoading ? (
            <div className="rules-detail-state">
              <ArrowPathIcon className="h-5 w-5 animate-spin" aria-hidden="true" />
              <p className="rules-detail-state-title">Loading rule explanations…</p>
              <p className="rules-detail-state-copy">
                The selected family is stored as a smaller generated chunk and loads on demand.
              </p>
            </div>
          ) : selectedRule ? (
            <>
              <div className="rules-panel-head">
                <div>
                  <p className="rules-panel-label">Rule detail</p>
                  <h3 className="rules-detail-title">{selectedRule.label}</h3>
                </div>
                <span className="rule-tag shared">{selectedRuleSummary?.familyLabel ?? activeChunk?.label}</span>
              </div>

              <p className="rules-detail-id">
                <code>{selectedRule.id}</code>
              </p>

              <div className="rules-detail-badges">
                <span className={`rule-tag lang-${activeLang}`}>{formatRuleSeverity(selectedRule.defaultSeverity)}</span>
                <span className="rule-tag shared">{formatRuleStatus(selectedRule.status)}</span>
                {selectedRule.configurability.map((config) => (
                  <span key={config} className="rule-tag shared">
                    {formatConfigurability(config)}
                  </span>
                ))}
              </div>

              <div className="rules-detail-block">
                <span className="rules-detail-label">What it catches</span>
                <p className="rules-detail-copy">{selectedRule.description}</p>
              </div>

              <div className="rules-detail-block">
                <span className="rules-detail-label">Why this matters</span>
                <p className="rules-detail-copy">{selectedRule.explanation}</p>
              </div>

              <div className="rules-detail-block">
                <span className="rules-detail-label">What to do instead</span>
                <p className="rules-detail-copy">{selectedRule.fix}</p>
              </div>

            </>
          ) : (
            <div className="rules-detail-state">
              <SparklesIcon className="h-5 w-5" aria-hidden="true" />
              <p className="rules-detail-state-title">Pick a rule to inspect it.</p>
              <p className="rules-detail-state-copy">
                The right panel shows the short explanation once a rule is selected.
              </p>
            </div>
          )}
        </div>
      </div>
    </section>
  )
}
