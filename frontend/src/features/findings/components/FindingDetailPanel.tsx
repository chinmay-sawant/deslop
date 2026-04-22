import type { FindingsRecord } from '../types'

type FindingDetailPanelProps = {
  finding: FindingsRecord | null
  functionText: string
  isLoading: boolean
  copyStateLabel: string
  onCopy: () => void
}

function MetaRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="findings-meta-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  )
}

export function FindingDetailPanel({ finding, functionText, isLoading, copyStateLabel, onCopy }: FindingDetailPanelProps) {
  return (
    <aside className="findings-panel findings-detail-panel">
      <div className="findings-panel-head">
        <div>
          <span className="eyebrow">Detail panel</span>
          <h2 className="findings-section-title">Function context, metadata, and why this row is here.</h2>
        </div>
        <button type="button" className="findings-copy-button" onClick={onCopy} disabled={!finding}>
          {copyStateLabel}
        </button>
      </div>

      {!finding ? (
        <div className="findings-detail-scroll findings-detail-scroll-empty">
          <div className="findings-empty-state compact">
            <strong>Select a finding</strong>
            <p>The graph and the table both feed this panel. Pick any row or finding node to inspect its function context.</p>
          </div>
        </div>
      ) : (
        <div className="findings-detail-scroll">
          <div className="findings-detail-body">
            <div className="findings-detail-title-block">
              <strong>{finding.ruleId}</strong>
              <p>{finding.message}</p>
            </div>

            <div className="findings-detail-meta">
              <MetaRow label="Source" value={`${finding.sourceDisplayPath}:${finding.line}`} />
              <MetaRow label="Repo" value={finding.sourceRepo} />
              <MetaRow label="Severity" value={finding.ruleSeverity} />
              <MetaRow label="Family" value={finding.ruleFamily} />
              <MetaRow label="Status" value={finding.ruleStatus} />
              <MetaRow label="Triage" value={finding.autoTriage} />
            </div>

            <div className="findings-detail-note-block">
              <span>Rule description</span>
              <p>{finding.ruleDescription}</p>
            </div>

            <div className="findings-detail-note-block">
              <span>Auto triage note</span>
              <p>{finding.autoTriageNote}</p>
            </div>

            <div className="findings-detail-code-block">
              <div className="findings-detail-code-head">
                <strong>Function preview</strong>
                <span>
                  {finding.functionFound && finding.functionStart != null && finding.functionEnd != null
                    ? `lines ${finding.functionStart}-${finding.functionEnd}`
                    : 'Function not found'}
                </span>
              </div>
              {isLoading ? <p className="findings-code-loading">Loading function text…</p> : null}
              <pre>{functionText || finding.functionPreview || '[FUNCTION_NOT_FOUND]'}</pre>
            </div>
          </div>
        </div>
      )}
    </aside>
  )
}
