import { useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'

import type { FindingsQueryState, GroupBy, SortBy } from '../types'

const defaultState: FindingsQueryState = {
  search: '',
  families: [],
  severities: [],
  statuses: [],
  languages: [],
  repos: [],
  rules: [],
  files: [],
  groupBy: 'none',
  sortBy: 'severity',
}

function parseList(value: string | null) {
  if (!value) {
    return []
  }
  return value.split(',').map((item) => item.trim()).filter(Boolean)
}

function normalizeGroupBy(value: string | null): GroupBy {
  switch (value) {
    case 'rule':
    case 'family':
    case 'repo':
    case 'file':
      return value
    default:
      return 'none'
  }
}

function normalizeSortBy(value: string | null): SortBy {
  switch (value) {
    case 'rule':
    case 'file':
    case 'line':
    case 'triage':
    case 'count':
      return value
    default:
      return 'severity'
  }
}

export function useFindingsQueryState() {
  const [searchParams, setSearchParams] = useSearchParams()

  const state = useMemo<FindingsQueryState>(() => ({
    search: searchParams.get('q') ?? defaultState.search,
    families: parseList(searchParams.get('family')),
    severities: parseList(searchParams.get('severity')),
    statuses: parseList(searchParams.get('status')),
    languages: parseList(searchParams.get('language')),
    repos: parseList(searchParams.get('repo')),
    rules: parseList(searchParams.get('rule')),
    files: parseList(searchParams.get('file')),
    groupBy: normalizeGroupBy(searchParams.get('group')),
    sortBy: normalizeSortBy(searchParams.get('sort')),
  }), [searchParams])

  const updateState = (patch: Partial<FindingsQueryState>) => {
    const next = { ...state, ...patch }
    const params = new URLSearchParams()

    if (next.search) params.set('q', next.search)
    if (next.families.length > 0) params.set('family', next.families.join(','))
    if (next.severities.length > 0) params.set('severity', next.severities.join(','))
    if (next.statuses.length > 0) params.set('status', next.statuses.join(','))
    if (next.languages.length > 0) params.set('language', next.languages.join(','))
    if (next.repos.length > 0) params.set('repo', next.repos.join(','))
    if (next.rules.length > 0) params.set('rule', next.rules.join(','))
    if (next.files.length > 0) params.set('file', next.files.join(','))
    if (next.groupBy !== 'none') params.set('group', next.groupBy)
    if (next.sortBy !== 'severity') params.set('sort', next.sortBy)

    setSearchParams(params, { replace: true })
  }

  return {
    state,
    updateState,
    resetState: () => setSearchParams(new URLSearchParams(), { replace: true }),
  }
}
