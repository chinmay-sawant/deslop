import type { FindingsCoreDataset, FindingsQueryState, FindingsRecord, GraphEdge, GroupBy, GroupedRow } from '../types'

const severityRank: Record<string, number> = {
  error: 0,
  warning: 1,
  contextual: 2,
  info: 3,
}

const triageRank: Record<string, number> = {
  LIKELY_REAL: 0,
  REVIEW_NEEDED: 1,
  CONTEXT_DEPENDENT: 2,
  LIKELY_SUBJECTIVE: 3,
  LIKELY_FALSE_POSITIVE: 4,
}

const groupLabelMap: Record<Exclude<GroupBy, 'none'>, keyof FindingsRecord> = {
  rule: 'ruleId',
  family: 'ruleFamily',
  repo: 'sourceRepo',
  file: 'sourceDisplayPath',
}

export function filterFindings(records: FindingsRecord[], state: FindingsQueryState) {
  const search = state.search.trim().toLowerCase()

  return records.filter((record) => {
    if (search && !record.searchText.includes(search)) {
      return false
    }
    if (state.families.length > 0 && !state.families.includes(record.ruleFamily)) {
      return false
    }
    if (state.severities.length > 0 && !state.severities.includes(record.ruleSeverity)) {
      return false
    }
    if (state.statuses.length > 0 && !state.statuses.includes(record.ruleStatus)) {
      return false
    }
    if (state.languages.length > 0 && !state.languages.includes(record.language)) {
      return false
    }
    if (state.repos.length > 0 && !state.repos.includes(record.sourceRepo)) {
      return false
    }
    if (state.rules.length > 0 && !state.rules.includes(record.ruleId)) {
      return false
    }
    if (state.files.length > 0 && !state.files.includes(record.sourceDisplayPath)) {
      return false
    }
    return true
  })
}

export function sortFindings(records: FindingsRecord[], sortBy: FindingsQueryState['sortBy']) {
  const copy = [...records]
  copy.sort((left, right) => {
    switch (sortBy) {
      case 'severity':
        return (severityRank[left.ruleSeverity] ?? 99) - (severityRank[right.ruleSeverity] ?? 99) || left.ruleId.localeCompare(right.ruleId) || left.sourceDisplayPath.localeCompare(right.sourceDisplayPath) || left.line - right.line
      case 'rule':
        return left.ruleId.localeCompare(right.ruleId) || left.sourceDisplayPath.localeCompare(right.sourceDisplayPath) || left.line - right.line
      case 'file':
        return left.sourceDisplayPath.localeCompare(right.sourceDisplayPath) || left.line - right.line || left.ruleId.localeCompare(right.ruleId)
      case 'triage':
        return (triageRank[left.autoTriage] ?? 99) - (triageRank[right.autoTriage] ?? 99) || left.ruleId.localeCompare(right.ruleId) || left.sourceDisplayPath.localeCompare(right.sourceDisplayPath)
      case 'line':
        return left.line - right.line || left.sourceDisplayPath.localeCompare(right.sourceDisplayPath) || left.ruleId.localeCompare(right.ruleId)
      case 'count':
      default:
        return left.id - right.id
    }
  })
  return copy
}

export function groupFindings(records: FindingsRecord[], groupBy: GroupBy): GroupedRow[] {
  if (groupBy === 'none') {
    return []
  }

  const field = groupLabelMap[groupBy]
  const groups = new Map<string, FindingsRecord[]>()
  for (const record of records) {
    const value = String(record[field])
    const list = groups.get(value)
    if (list) {
      list.push(record)
    } else {
      groups.set(value, [record])
    }
  }

  return [...groups.entries()]
    .map(([label, findings]) => ({
      id: `${groupBy}:${label}`,
      label,
      groupBy,
      count: findings.length,
      findings,
    }))
    .sort((left, right) => right.count - left.count || left.label.localeCompare(right.label))
}

export function buildGraphView(
  dataset: FindingsCoreDataset,
  filtered: FindingsRecord[],
  options: {
    maxNodes: number
    selectedFindingId: number | null
  },
) {
  type FindingsGraphViewNode = {
    id: string
    nodeType: 'rule' | 'file' | 'finding'
    label: string
    count: number
    findingId?: number
    path?: string
    sourcePath?: string
  }

  const selectedRecord = options.selectedFindingId == null
    ? null
    : filtered.find((record) => record.id === options.selectedFindingId) ?? dataset.records.find((record) => record.id === options.selectedFindingId) ?? null

  const ruleCounter = new Map<string, number>()
  const fileCounter = new Map<string, number>()
  const pairCounter = new Map<string, number>()

  for (const record of filtered) {
    ruleCounter.set(record.ruleId, (ruleCounter.get(record.ruleId) ?? 0) + 1)
    fileCounter.set(record.sourceDisplayPath, (fileCounter.get(record.sourceDisplayPath) ?? 0) + 1)
    const pairKey = `${record.ruleId}:::${record.sourceDisplayPath}`
    pairCounter.set(pairKey, (pairCounter.get(pairKey) ?? 0) + 1)
  }

  const maxRules = Math.max(8, Math.floor(options.maxNodes * 0.25))
  const maxFiles = Math.max(12, Math.floor(options.maxNodes * 0.4))
  const topRules = [...ruleCounter.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0])).slice(0, maxRules)
  const topFiles = [...fileCounter.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0])).slice(0, maxFiles)
  const allowedRules = new Set(topRules.map(([ruleId]) => ruleId))
  const allowedFiles = new Set(topFiles.map(([file]) => file))

  const nodes: FindingsGraphViewNode[] = []
  const edges: Array<GraphEdge & { source: string; target: string }> = []

  for (const [ruleId, count] of topRules) {
    nodes.push({ id: `rule:${ruleId}`, nodeType: 'rule', label: ruleId, count })
  }
  for (const [filePath, count] of topFiles) {
    nodes.push({ id: `file:${filePath}`, nodeType: 'file', label: filePath.split('/').pop() ?? filePath, count, sourcePath: filePath, path: filePath })
  }

  for (const [pairKey, count] of [...pairCounter.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))) {
    const [ruleId, filePath] = pairKey.split(':::')
    if (!allowedRules.has(ruleId) || !allowedFiles.has(filePath)) {
      continue
    }
    edges.push({
      id: `edge:${ruleId}:${filePath}`,
      source: `rule:${ruleId}`,
      target: `file:${filePath}`,
      count,
    })
  }

  const leafCandidates = selectedRecord
    ? filtered.filter((record) => record.ruleId === selectedRecord.ruleId || record.sourceDisplayPath === selectedRecord.sourceDisplayPath).slice(0, 18)
    : filtered.slice(0, Math.max(10, Math.floor(options.maxNodes * 0.18)))

  for (const record of leafCandidates) {
    const findingNodeId = `finding:${record.id}`
    nodes.push({
      id: findingNodeId,
      nodeType: 'finding',
      label: `${record.ruleId}:${record.line}`,
      count: 1,
      findingId: record.id,
      path: record.sourceDisplayPath,
    })
    edges.push({
      id: `edge:file:${record.sourceDisplayPath}:finding:${record.id}`,
      source: `file:${record.sourceDisplayPath}`,
      target: findingNodeId,
      count: 1,
    })
  }

  return { nodes, edges, selectedRecord }
}
