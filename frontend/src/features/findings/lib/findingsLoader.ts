import { sitePath } from '../../../shared/lib/sitePath'
import type { DetailShardPayload, FindingsCoreDataset, FindingsManifest, FindingsRecord } from '../types'

let manifestPromise: Promise<FindingsManifest> | null = null
let datasetPromise: Promise<FindingsCoreDataset> | null = null
const shardPromises = new Map<string, Promise<Map<number, string>>>()

async function fetchJson<T>(path: string): Promise<T> {
  const response = await fetch(sitePath(path))
  if (!response.ok) {
    throw new Error(`Failed to load ${path}: ${response.status}`)
  }
  return response.json() as Promise<T>
}

export function loadFindingsManifest() {
  if (!manifestPromise) {
    manifestPromise = fetchJson<FindingsManifest>('findings/manifest.json')
  }
  return manifestPromise
}

export async function loadFindingsDataset() {
  if (!datasetPromise) {
    datasetPromise = loadFindingsManifest().then((manifest) => {
      if (!manifest.files.dataset) {
        throw new Error('Findings manifest does not include a dataset file.')
      }
      return fetchJson<FindingsCoreDataset>(manifest.files.dataset)
    })
  }
  return datasetPromise
}

function findShardPath(manifest: FindingsManifest, finding: FindingsRecord) {
  if (!finding.detailShard || !manifest.files.detailShards) {
    return null
  }
  return manifest.files.detailShards.find((shard) => shard.key === finding.detailShard)?.path ?? null
}

async function loadShard(path: string) {
  let shardPromise = shardPromises.get(path)
  if (!shardPromise) {
    shardPromise = fetchJson<DetailShardPayload>(path).then((payload) => {
      const map = new Map<number, string>()
      for (const detail of payload.details) {
        map.set(detail.id, detail.functionText)
      }
      return map
    })
    shardPromises.set(path, shardPromise)
  }
  return shardPromise
}

export async function loadFindingFunctionText(manifest: FindingsManifest, finding: FindingsRecord) {
  if (typeof finding.functionText === 'string') {
    return finding.functionText
  }

  const shardPath = findShardPath(manifest, finding)
  if (!shardPath) {
    return ''
  }

  const shard = await loadShard(shardPath)
  return shard.get(finding.id) ?? ''
}
