import type { CountEntry, FindingsManifest, FindingsQueryState } from '../types'

type FindingsFilterPanelProps = {
  manifest: FindingsManifest
  state: FindingsQueryState
  onChange: (patch: Partial<FindingsQueryState>) => void
  onReset: () => void
}

const filterConfig: Array<{
  label: string
  stateKey: keyof Pick<FindingsQueryState, 'families' | 'severities' | 'statuses' | 'languages' | 'repos'>
  manifestKey: keyof FindingsManifest['summary']['counts']
}> = [
  { label: 'Family', stateKey: 'families', manifestKey: 'families' },
  { label: 'Severity', stateKey: 'severities', manifestKey: 'severities' },
  { label: 'Status', stateKey: 'statuses', manifestKey: 'statuses' },
  { label: 'Language', stateKey: 'languages', manifestKey: 'languages' },
  { label: 'Repo', stateKey: 'repos', manifestKey: 'repos' },
]

function ToggleChip({ active, label, onClick }: { active: boolean; label: string; onClick: () => void }) {
  return (
    <button
      type="button"
      className={active ? 'findings-chip findings-chip-active' : 'findings-chip'}
      onClick={onClick}
    >
      {label}
    </button>
  )
}

function entryLabel(entry: CountEntry) {
  return `${entry.key} (${entry.count})`
}

export function FindingsFilterPanel({ manifest, state, onChange, onReset }: FindingsFilterPanelProps) {
  return (
    <section className="findings-panel findings-filters-panel">
      <div className="findings-filter-head">
        <div>
          <span className="eyebrow">Query surface</span>
          <h2 className="findings-section-title">Slice the dataset before we draw or render anything expensive.</h2>
        </div>
        <button type="button" className="button-secondary findings-reset-button" onClick={onReset}>
          Reset filters
        </button>
      </div>

      <div className="findings-toolbar-grid">
        <label className="findings-search-box">
          <span>Search path, rule, message, or function preview</span>
          <input
            type="search"
            value={state.search}
            onChange={(event) => onChange({ search: event.target.value })}
            placeholder="public_api_missing_type_hints, swarms/utils, print debugging…"
          />
        </label>

        <label className="findings-select-box">
          <span>Group rows</span>
          <select value={state.groupBy} onChange={(event) => onChange({ groupBy: event.target.value as FindingsQueryState['groupBy'] })}>
            <option value="none">No grouping</option>
            <option value="rule">Rule</option>
            <option value="family">Family</option>
            <option value="repo">Repo</option>
            <option value="file">File</option>
          </select>
        </label>

        <label className="findings-select-box">
          <span>Sort rows</span>
          <select value={state.sortBy} onChange={(event) => onChange({ sortBy: event.target.value as FindingsQueryState['sortBy'] })}>
            <option value="severity">Severity</option>
            <option value="rule">Rule</option>
            <option value="file">File</option>
            <option value="line">Line</option>
            <option value="triage">Triage</option>
            <option value="count">Input order</option>
          </select>
        </label>
      </div>

      <div className="findings-filter-groups">
        {filterConfig.map((config) => {
          const entries = manifest.summary.counts[config.manifestKey] as CountEntry[]
          const activeValues = state[config.stateKey] as string[]

          return (
            <div key={config.label} className="findings-filter-group">
              <div className="findings-filter-group-head">
                <span>{config.label}</span>
                <span>{activeValues.length === 0 ? 'All' : `${activeValues.length} active`}</span>
              </div>
              <div className="findings-chip-wrap">
                {entries.slice(0, config.stateKey === 'repos' ? 18 : 12).map((entry) => {
                  const active = activeValues.includes(entry.key)
                  return (
                    <ToggleChip
                      key={entry.key}
                      active={active}
                      label={entryLabel(entry)}
                      onClick={() => {
                        const nextValues = active
                          ? activeValues.filter((value) => value !== entry.key)
                          : [...activeValues, entry.key]
                        onChange({ [config.stateKey]: nextValues } as Partial<FindingsQueryState>)
                      }}
                    />
                  )
                })}
              </div>
            </div>
          )
        })}
      </div>
    </section>
  )
}
