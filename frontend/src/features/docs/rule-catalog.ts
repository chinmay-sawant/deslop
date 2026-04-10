import manifestData from './generated/rule-manifest.json'
import type { Language } from './docs-content'

type RuleSeverity = 'contextual' | 'error' | 'info' | 'warning'
type RuleStatus = 'experimental' | 'research' | 'stable'
type RuleConfigurability = 'disable' | 'ignore' | 'severity_override'

interface RuleSummary {
  id: string
  label: string
  defaultSeverity: RuleSeverity
  status: RuleStatus
}

interface RuleFamilyManifest {
  id: string
  label: string
  summary: string
  ruleCount: number
  rules: RuleSummary[]
}

interface RuleLanguageManifest {
  ruleCount: number
  families: RuleFamilyManifest[]
}

interface RuleCatalogManifest {
  languages: Record<Language, RuleLanguageManifest>
}

interface RuleDoc extends RuleSummary {
  description: string
  configurability: RuleConfigurability[]
  explanation: string
  fix: string
}

interface RuleFamilyChunk {
  language: Language
  family: string
  label: string
  summary: string
  rules: RuleDoc[]
}

const ruleCatalog = manifestData as RuleCatalogManifest
const familyLoaders = import.meta.glob('./generated/rules/*/*.json')

export async function loadRuleFamily(language: Language, family: string): Promise<RuleFamilyChunk> {
  const path = `./generated/rules/${language}/${family}.json`
  const loader = familyLoaders[path]

  if (!loader) {
    throw new Error(`Missing generated rule family chunk for ${language}/${family}`)
  }

  const module = (await loader()) as { default: RuleFamilyChunk }
  return module.default
}

export function formatRuleSeverity(severity: RuleSeverity): string {
  if (severity === 'contextual') {
    return 'Contextual'
  }

  return severity.charAt(0).toUpperCase() + severity.slice(1)
}

export function formatRuleStatus(status: RuleStatus): string {
  return status.charAt(0).toUpperCase() + status.slice(1)
}

export function formatConfigurability(config: RuleConfigurability): string {
  if (config === 'severity_override') {
    return 'Severity override'
  }

  return config.charAt(0).toUpperCase() + config.slice(1)
}

export { ruleCatalog }
export type {
  RuleCatalogManifest,
  RuleConfigurability,
  RuleDoc,
  RuleFamilyChunk,
  RuleFamilyManifest,
  RuleLanguageManifest,
  RuleSeverity,
  RuleStatus,
  RuleSummary,
}
