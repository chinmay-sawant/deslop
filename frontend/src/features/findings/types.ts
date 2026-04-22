export type CountEntry = {
  key: string
  count: number
}

export type FindingsSummary = {
  totals: {
    findings: number
    rules: number
    files: number
    repos: number
    missingFiles: number
    missingFunctions: number
  }
  counts: {
    families: CountEntry[]
    severities: CountEntry[]
    statuses: CountEntry[]
    languages: CountEntry[]
    repos: CountEntry[]
    rules: CountEntry[]
    files: CountEntry[]
    autoTriage: CountEntry[]
  }
  topRules: CountEntry[]
  topFiles: CountEntry[]
  topRepos: CountEntry[]
}

export type FindingsRecord = {
  id: number
  sourcePath: string
  sourceDisplayPath: string
  sourceFile: string
  sourceRepo: string
  line: number
  ruleId: string
  ruleFamily: string
  ruleSeverity: string
  ruleStatus: string
  ruleLanguages: string
  ruleDescription: string
  message: string
  autoTriage: string
  autoTriageNote: string
  functionFound: boolean
  functionStart: number | null
  functionEnd: number | null
  functionPreview: string
  language: string
  tags: string[]
  rawFinding: string
  fileExists: boolean
  searchText: string
  detailShard?: string | null
  functionText?: string
}

export type GraphNode = {
  id: string
  nodeType: 'rule' | 'file'
  label: string
  count: number
  sourcePath?: string
}

export type GraphEdge = {
  id: string
  source: string
  target: string
  count: number
}

export type FindingsCoreDataset = {
  version: number
  records: FindingsRecord[]
  graph: {
    nodes: GraphNode[]
    edges: GraphEdge[]
    topRuleIds: string[]
    topFilePaths: string[]
  }
  summary: FindingsSummary
}

export type DetailShard = {
  key: string
  path: string
  start_id: number
  end_id: number
  count: number
}

export type FindingsManifest = {
  version: number
  generatedFrom: string
  totalFindings: number
  summaryOnly: boolean
  includesFunctionText: boolean
  summary: FindingsSummary
  filters: {
    families: string[]
    severities: string[]
    statuses: string[]
    languages: string[]
    repos: string[]
    rules: string[]
  }
  graph: {
    nodeCount: number
    edgeCount: number
    topRuleIds: string[]
    topFilePaths: string[]
  }
  files: {
    dataset?: string
    detailShards?: DetailShard[]
  }
}

export type DetailShardPayload = {
  version: number
  details: Array<{
    id: number
    functionText: string
  }>
}

export type GroupBy = 'none' | 'rule' | 'family' | 'repo' | 'file'
export type SortBy = 'severity' | 'rule' | 'file' | 'line' | 'triage' | 'count'

export type FindingsQueryState = {
  search: string
  families: string[]
  severities: string[]
  statuses: string[]
  languages: string[]
  repos: string[]
  rules: string[]
  files: string[]
  groupBy: GroupBy
  sortBy: SortBy
}

export type GroupedRow = {
  id: string
  label: string
  groupBy: Exclude<GroupBy, 'none'>
  count: number
  findings: FindingsRecord[]
}
