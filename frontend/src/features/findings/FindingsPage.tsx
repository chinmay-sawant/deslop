import { useEffect, useMemo, useState } from 'react'

import { Container } from '../../shared/ui/Container'
import { FindingDetailPanel } from './components/FindingDetailPanel'
import { FindingsFilterPanel } from './components/FindingsFilterPanel'
import { FindingsGraph, type FindingsGraphNode } from './components/FindingsGraph'
import { FindingsOverview } from './components/FindingsOverview'
import { FindingsTable } from './components/FindingsTable'
import { loadFindingFunctionText, loadFindingsDataset, loadFindingsManifest } from './lib/findingsLoader'
import { buildGraphView, filterFindings, groupFindings, sortFindings } from './lib/findingsSelectors'
import { useFindingsQueryState } from './lib/useFindingsQueryState'
import type { FindingsCoreDataset, FindingsManifest, FindingsQueryState, FindingsRecord } from './types'

type LoadState = {
  manifest: FindingsManifest | null
  dataset: FindingsCoreDataset | null
  error: string | null
  loading: boolean
}

function EmptyDatasetState({ message }: { message: string }) {
  return (
    <section className="findings-empty-state">
      <strong>Findings dataset not available yet</strong>
      <p>{message}</p>
      <pre>{`python3 scripts/extract_function_context_json.py temp.txt \\
  --output-dir frontend/public/findings \\
  --include-function-text`}</pre>
    </section>
  )
}

function applyGroupFilterPatch(groupBy: Exclude<FindingsQueryState['groupBy'], 'none'>, value: string): Partial<FindingsQueryState> {
  switch (groupBy) {
    case 'rule':
      return { rules: [value], groupBy: 'none' }
    case 'family':
      return { families: [value], groupBy: 'none' }
    case 'repo':
      return { repos: [value], groupBy: 'none' }
    case 'file':
      return { files: [value], groupBy: 'none' }
  }
}

export function FindingsPage() {
  const [{ manifest, dataset, error, loading }, setLoadState] = useState<LoadState>({
    manifest: null,
    dataset: null,
    error: null,
    loading: true,
  })
  const { state, updateState, resetState } = useFindingsQueryState()
  const [selectedFindingId, setSelectedFindingId] = useState<number | null>(null)
  const [functionTextById, setFunctionTextById] = useState<Record<number, string>>({})
  const [copyStateLabel, setCopyStateLabel] = useState('Copy current finding')

  useEffect(() => {
    let cancelled = false

    Promise.all([loadFindingsManifest(), loadFindingsDataset()])
      .then(([nextManifest, nextDataset]) => {
        if (cancelled) {
          return
        }
        setLoadState({ manifest: nextManifest, dataset: nextDataset, error: null, loading: false })
      })
      .catch((loadError: Error) => {
        if (cancelled) {
          return
        }
        setLoadState({ manifest: null, dataset: null, error: loadError.message, loading: false })
      })

    return () => {
      cancelled = true
    }
  }, [])

  const filtered = useMemo(() => {
    if (!dataset) {
      return []
    }
    return sortFindings(filterFindings(dataset.records, state), state.sortBy)
  }, [dataset, state])

  const groups = useMemo(() => groupFindings(filtered, state.groupBy), [filtered, state.groupBy])

  const selectedFinding: FindingsRecord | null = useMemo(() => {
    if (filtered.length === 0) {
      return null
    }
    if (selectedFindingId == null) {
      return filtered[0]
    }
    return filtered.find((record) => record.id === selectedFindingId) ?? filtered[0]
  }, [filtered, selectedFindingId])

  useEffect(() => {
    let cancelled = false
    if (!manifest || !selectedFinding || functionTextById[selectedFinding.id] != null) {
      return
    }

    loadFindingFunctionText(manifest, selectedFinding)
      .then((text) => {
        if (!cancelled) {
          setFunctionTextById((current) => ({ ...current, [selectedFinding.id]: text }))
        }
      })

    return () => {
      cancelled = true
    }
  }, [functionTextById, manifest, selectedFinding])

  const detailLoading = Boolean(
    manifest
      && selectedFinding
      && manifest.includesFunctionText
      && functionTextById[selectedFinding.id] == null,
  )

  const graph = useMemo(() => {
    if (!dataset) {
      return { nodes: [], edges: [], selectedRecord: null }
    }
    return buildGraphView(dataset, filtered, { maxNodes: 54, selectedFindingId })
  }, [dataset, filtered, selectedFindingId])

  const selectedFunctionText = selectedFinding ? functionTextById[selectedFinding.id] ?? '' : ''

  const handleCopyCurrentView = async () => {
    if (!selectedFinding) {
      return
    }

    const payload = [
      'Virtualized Table',
      `Rule: ${selectedFinding.ruleId}`,
      `Message: ${selectedFinding.message}`,
      `Source: ${selectedFinding.sourceDisplayPath}:${selectedFinding.line}`,
      `Severity: ${selectedFinding.ruleSeverity}`,
      `Triage: ${selectedFinding.autoTriage}`,
      `Language: ${selectedFinding.language}`,
      '',
      'Detail Panel',
      `Repo: ${selectedFinding.sourceRepo}`,
      `Family: ${selectedFinding.ruleFamily}`,
      `Status: ${selectedFinding.ruleStatus}`,
      `Rule description: ${selectedFinding.ruleDescription}`,
      `Auto triage note: ${selectedFinding.autoTriageNote}`,
      '',
      'Function Preview',
      selectedFunctionText || selectedFinding.functionPreview || '[FUNCTION_NOT_FOUND]',
    ].join('\n')

    try {
      await navigator.clipboard.writeText(payload)
      setCopyStateLabel('Copied')
      window.setTimeout(() => setCopyStateLabel('Copy current finding'), 1600)
    } catch {
      setCopyStateLabel('Copy failed')
      window.setTimeout(() => setCopyStateLabel('Copy current finding'), 1600)
    }
  }

  if (loading) {
    return (
      <Container className="py-16">
        <EmptyDatasetState message="Loading the findings manifest and dataset…" />
      </Container>
    )
  }

  if (error || !manifest || !dataset) {
    return (
      <Container className="py-16">
        <EmptyDatasetState message={error ?? 'The dataset files were not found under frontend/public/findings.'} />
      </Container>
    )
  }

  return (
    <div className="findings-page-shell">
      <section className="findings-hero">
        <div className="findings-page-width findings-page-width-hero">
          <div className="findings-hero-copy">
            <span className="eyebrow">Findings visualizer</span>
            <h1>Obsidian-style exploration for large deslop scans.</h1>
            <p>
              Pre-aggregated JSON, scoped graph rendering, grouped review tables, and lazy function detail loading.
              The current bundle covers {manifest.totalFindings.toLocaleString()} findings from {manifest.summary.totals.repos.toLocaleString()} repositories.
            </p>
          </div>
        </div>
      </section>

      <main className="pb-24">
        <div className="findings-page-width findings-layout">
          <FindingsOverview summary={manifest.summary} />

          <FindingsFilterPanel
            manifest={manifest}
            state={state}
            onChange={updateState}
            onReset={resetState}
          />

          <FindingsGraph
            nodes={graph.nodes}
            edges={graph.edges}
            selectedFinding={selectedFinding}
            onNodeSelect={(node: FindingsGraphNode) => {
              if (node.nodeType === 'rule') {
                updateState({ rules: [node.label] })
                return
              }
              if (node.nodeType === 'file' && node.path) {
                updateState({ files: [node.path] })
                return
              }
              if (node.nodeType === 'finding' && node.findingId) {
                setSelectedFindingId(node.findingId)
              }
            }}
          />

          <div className="findings-review-grid">
            <FindingsTable
              records={filtered}
              groups={groups}
              groupBy={state.groupBy}
              selectedFindingId={selectedFinding?.id ?? null}
              onSelectFinding={(finding) => setSelectedFindingId(finding.id)}
              onApplyGroupFilter={(groupBy, value) => updateState(applyGroupFilterPatch(groupBy, value))}
            />

            <FindingDetailPanel
              finding={selectedFinding}
              functionText={selectedFunctionText}
              isLoading={detailLoading}
              copyStateLabel={copyStateLabel}
              onCopy={handleCopyCurrentView}
            />
          </div>
        </div>
      </main>
    </div>
  )
}
