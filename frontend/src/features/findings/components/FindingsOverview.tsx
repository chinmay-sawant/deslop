import type { FindingsSummary } from '../types'

function MetricCard({ label, value, note }: { label: string; value: string; note: string }) {
  return (
    <article className="findings-metric-card">
      <span className="findings-metric-label">{label}</span>
      <strong className="findings-metric-value">{value}</strong>
      <p className="findings-metric-note">{note}</p>
    </article>
  )
}

function BucketList({ title, entries }: { title: string; entries: FindingsSummary['topRules'] }) {
  const max = entries[0]?.count ?? 1

  return (
    <section className="findings-panel">
      <div className="findings-panel-head">
        <h3>{title}</h3>
      </div>
      <div className="findings-bars">
        {entries.map((entry) => (
          <div key={entry.key} className="findings-bar-row">
            <div className="findings-bar-labels">
              <span className="findings-bar-key" title={entry.key}>{entry.key}</span>
              <strong>{entry.count}</strong>
            </div>
            <div className="findings-bar-track">
              <div className="findings-bar-fill" style={{ width: `${(entry.count / max) * 100}%` }} />
            </div>
          </div>
        ))}
      </div>
    </section>
  )
}

export function FindingsOverview({ summary }: { summary: FindingsSummary }) {
  return (
    <section className="findings-overview">
      <div className="findings-metrics-grid">
        <MetricCard label="Findings" value={summary.totals.findings.toLocaleString()} note="Structured records ready for search, filters, grouping, and graphing." />
        <MetricCard label="Rules" value={summary.totals.rules.toLocaleString()} note="Distinct detectors represented in the current scan bundle." />
        <MetricCard label="Files" value={summary.totals.files.toLocaleString()} note="Unique files carrying at least one finding." />
        <MetricCard label="Repos" value={summary.totals.repos.toLocaleString()} note="Top-level source repositories inferred from the finding paths." />
      </div>

      <div className="findings-overview-grid">
        <BucketList title="Top rules" entries={summary.topRules} />
        <BucketList title="Hot files" entries={summary.topFiles} />
        <BucketList title="Repo hotspots" entries={summary.topRepos} />
      </div>
    </section>
  )
}
