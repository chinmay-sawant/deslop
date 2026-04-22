import { useCallback, useMemo, useState } from 'react'

import type { FindingsRecord, GroupBy, GroupedRow } from '../types'

type FindingsTableProps = {
  records: FindingsRecord[]
  groups: GroupedRow[]
  groupBy: GroupBy
  selectedFindingId: number | null
  onSelectFinding: (finding: FindingsRecord) => void
  onApplyGroupFilter: (groupBy: Exclude<GroupBy, 'none'>, value: string) => void
}

type RowModel =
  | { key: string; kind: 'finding'; finding: FindingsRecord }
  | { key: string; kind: 'group'; group: GroupedRow }

type RowMetric = {
  key: string
  top: number
  height: number
}

const VIEWPORT_HEIGHT = 620
const OVERSCAN_PX = 320

function severityClass(severity: string) {
  switch (severity) {
    case 'error':
      return 'findings-badge findings-badge-error'
    case 'warning':
      return 'findings-badge findings-badge-warning'
    case 'contextual':
      return 'findings-badge findings-badge-contextual'
    default:
      return 'findings-badge findings-badge-info'
  }
}

function estimateFindingHeight(finding: FindingsRecord) {
  const ruleLines = Math.ceil(finding.ruleId.length / 34)
  const messageLines = Math.ceil(finding.message.length / 52)
  const sourceLines = Math.ceil(finding.sourceDisplayPath.length / 42)
  const contentLines = Math.max(ruleLines + Math.min(messageLines, 4), sourceLines + 1, 2)
  return 44 + contentLines * 22
}

function estimateGroupHeight(group: GroupedRow) {
  const labelLines = Math.ceil(group.label.length / 38)
  const exampleSource = group.findings[0]?.sourceDisplayPath ?? ''
  const sourceLines = Math.ceil(exampleSource.length / 42)
  return 48 + Math.max(labelLines, sourceLines, 2) * 22
}

function buildMetrics(rows: RowModel[], measuredHeights: Record<string, number>) {
  const metrics: RowMetric[] = []
  let top = 0

  for (const row of rows) {
    const estimatedHeight = row.kind === 'finding'
      ? estimateFindingHeight(row.finding)
      : estimateGroupHeight(row.group)
    const height = measuredHeights[row.key] ?? estimatedHeight
    metrics.push({ key: row.key, top, height })
    top += height
  }

  return {
    metrics,
    totalHeight: top,
  }
}

function findStartIndex(metrics: RowMetric[], scrollTop: number) {
  let low = 0
  let high = metrics.length - 1
  let answer = 0

  while (low <= high) {
    const mid = Math.floor((low + high) / 2)
    const item = metrics[mid]
    if (item.top + item.height < scrollTop) {
      low = mid + 1
    } else {
      answer = mid
      high = mid - 1
    }
  }

  return answer
}

export function FindingsTable({
  records,
  groups,
  groupBy,
  selectedFindingId,
  onSelectFinding,
  onApplyGroupFilter,
}: FindingsTableProps) {
  const [scrollTop, setScrollTop] = useState(0)
  const [measuredHeights, setMeasuredHeights] = useState<Record<string, number>>({})

  const rows = useMemo<RowModel[]>(() => {
    if (groupBy === 'none') {
      return records.map((finding) => ({ key: `finding:${finding.id}`, kind: 'finding', finding }))
    }
    return groups.map((group) => ({ key: group.id, kind: 'group', group }))
  }, [groupBy, groups, records])

  const { metrics, totalHeight } = useMemo(() => buildMetrics(rows, measuredHeights), [rows, measuredHeights])

  const visibleRange = useMemo(() => {
    if (rows.length === 0) {
      return { startIndex: 0, endIndex: 0 }
    }
    const startIndex = Math.max(0, findStartIndex(metrics, Math.max(0, scrollTop - OVERSCAN_PX)))
    let endIndex = startIndex
    const cutoff = scrollTop + VIEWPORT_HEIGHT + OVERSCAN_PX
    while (endIndex < metrics.length && metrics[endIndex].top < cutoff) {
      endIndex += 1
    }
    return { startIndex, endIndex }
  }, [metrics, rows.length, scrollTop])

  const visibleRows = rows.slice(visibleRange.startIndex, visibleRange.endIndex)

  const measureRow = useCallback((rowKey: string, node: HTMLDivElement | HTMLButtonElement | null) => {
    if (!node) {
      return
    }
    const nextHeight = Math.ceil(node.getBoundingClientRect().height)
    setMeasuredHeights((current) => {
      if (current[rowKey] === nextHeight) {
        return current
      }
      return { ...current, [rowKey]: nextHeight }
    })
  }, [])

  return (
    <section className="findings-panel findings-table-panel">
      <div className="findings-panel-head">
        <div>
          <span className="eyebrow">Virtualized table</span>
          <h2 className="findings-section-title">Dense rows for scan review, grouped when you need higher-level triage.</h2>
        </div>
        <span className="findings-table-count">{rows.length.toLocaleString()} rows</span>
      </div>

      <div className="findings-table-header-grid">
        {groupBy === 'none' ? (
          <>
            <span>Rule</span>
            <span>Source</span>
            <span>Severity</span>
            <span>Triage</span>
          </>
        ) : (
          <>
            <span>Group</span>
            <span>Example finding</span>
            <span>Total</span>
            <span>Action</span>
          </>
        )}
      </div>

      <div
        className="findings-table-viewport"
        style={{ height: VIEWPORT_HEIGHT }}
        onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)}
      >
        <div style={{ height: totalHeight, position: 'relative' }}>
          {visibleRows.map((row, offset) => {
            const index = visibleRange.startIndex + offset
            const metric = metrics[index]
            if (!metric) {
              return null
            }

            if (row.kind === 'finding') {
              const finding = row.finding
              return (
                <button
                  key={row.key}
                  ref={(node) => measureRow(row.key, node)}
                  type="button"
                  className={finding.id === selectedFindingId ? 'findings-table-row findings-table-row-selected' : 'findings-table-row'}
                  style={{ top: metric.top, minHeight: metric.height }}
                  onClick={() => onSelectFinding(finding)}
                >
                  <div>
                    <strong>{finding.ruleId}</strong>
                    <p>{finding.message}</p>
                  </div>
                  <div>
                    <strong>{finding.sourceDisplayPath}</strong>
                    <p>line {finding.line} • {finding.language}</p>
                  </div>
                  <div>
                    <span className={severityClass(finding.ruleSeverity)}>{finding.ruleSeverity}</span>
                  </div>
                  <div>
                    <strong>{finding.autoTriage}</strong>
                    <p>{finding.language}</p>
                  </div>
                </button>
              )
            }

            const group = row.group
            const example = group.findings[0]
            return (
              <div
                key={row.key}
                ref={(node) => measureRow(row.key, node)}
                className="findings-table-row findings-table-group-row"
                style={{ top: metric.top, minHeight: metric.height }}
              >
                <div>
                  <strong>{group.label}</strong>
                  <p>{group.groupBy}</p>
                </div>
                <div>
                  <strong>{example?.ruleId ?? 'No example'}</strong>
                  <p>{example?.sourceDisplayPath ?? '—'}</p>
                </div>
                <div>
                  <strong>{group.count.toLocaleString()}</strong>
                  <p>findings</p>
                </div>
                <div>
                  <button type="button" className="findings-inline-button" onClick={() => onApplyGroupFilter(group.groupBy, group.label)}>
                    Filter to group
                  </button>
                </div>
              </div>
            )
          })}
        </div>
      </div>
    </section>
  )
}
