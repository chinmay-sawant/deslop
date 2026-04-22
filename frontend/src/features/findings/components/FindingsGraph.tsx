import type { FindingsRecord } from '../types'

export type FindingsGraphNode = {
  id: string
  nodeType: 'rule' | 'file' | 'finding'
  label: string
  count: number
  findingId?: number
  path?: string
}

export type FindingsGraphEdge = {
  id: string
  source: string
  target: string
  count: number
}

type FindingsGraphProps = {
  nodes: FindingsGraphNode[]
  edges: FindingsGraphEdge[]
  selectedFinding: FindingsRecord | null
  onNodeSelect: (node: FindingsGraphNode) => void
}

const WIDTH = 1180
const HEIGHT = 620
const GRAPH_LABEL_FONT_SIZE = 10
const GRAPH_LABEL_PADDING = 6

function polarToCartesian(cx: number, cy: number, radius: number, angle: number) {
  return {
    x: cx + radius * Math.cos(angle),
    y: cy + radius * Math.sin(angle),
  }
}

function positionNodes(nodes: FindingsGraphNode[]) {
  const ruleNodes = nodes.filter((node) => node.nodeType === 'rule')
  const fileNodes = nodes.filter((node) => node.nodeType === 'file')
  const findingNodes = nodes.filter((node) => node.nodeType === 'finding')
  const positions = new Map<string, { x: number; y: number }>()

  ruleNodes.forEach((node, index) => {
    const point = polarToCartesian(270, HEIGHT / 2, 235, -Math.PI / 2 + (index / Math.max(ruleNodes.length, 1)) * Math.PI)
    positions.set(node.id, point)
  })
  fileNodes.forEach((node, index) => {
    const point = polarToCartesian(845, HEIGHT / 2, 235, -Math.PI / 2 + (index / Math.max(fileNodes.length, 1)) * Math.PI)
    positions.set(node.id, point)
  })
  findingNodes.forEach((node, index) => {
    const cols = 4
    const col = index % cols
    const row = Math.floor(index / cols)
    positions.set(node.id, { x: 970 + col * 56, y: 92 + row * 58 })
  })

  return positions
}

function nodeRadius(node: FindingsGraphNode) {
  if (node.nodeType === 'finding') {
    return 8
  }
  return Math.min(28, 10 + Math.sqrt(node.count) * 2.5)
}

type LabelCandidate = {
  nodeId: string
  x: number
  y: number
  textAnchor: 'start' | 'middle' | 'end'
  text: string
  priority: number
  bbox: {
    left: number
    right: number
    top: number
    bottom: number
  }
}

function shortenLabel(text: string, maxLength: number) {
  if (text.length <= maxLength) {
    return text
  }
  return `${text.slice(0, maxLength - 1)}…`
}

function buildLabelCandidates(
  nodes: FindingsGraphNode[],
  positions: Map<string, { x: number; y: number }>,
  selectedFinding: FindingsRecord | null,
) {
  const candidates: LabelCandidate[] = []

  for (const node of nodes) {
    const point = positions.get(node.id)
    if (!point) {
      continue
    }

    const radius = nodeRadius(node)
    const isSelected = node.nodeType === 'finding' && node.findingId === selectedFinding?.id
    const text = node.nodeType === 'finding'
      ? `#${node.findingId}`
      : shortenLabel(node.label, node.nodeType === 'rule' ? 42 : 34)

    const textWidth = text.length * 6.1
    let x = point.x
    let y = point.y
    let textAnchor: 'start' | 'middle' | 'end' = 'middle'

    if (node.nodeType === 'rule') {
      x = point.x - radius - 12
      y = point.y + 4
      textAnchor = 'end'
    } else if (node.nodeType === 'file') {
      x = point.x + radius + 12
      y = point.y + 4
      textAnchor = 'start'
    } else {
      x = point.x
      y = point.y + 24
      textAnchor = 'middle'
    }

    const left = textAnchor === 'middle' ? x - textWidth / 2 - GRAPH_LABEL_PADDING : textAnchor === 'start' ? x - GRAPH_LABEL_PADDING : x - textWidth - GRAPH_LABEL_PADDING
    const right = textAnchor === 'middle' ? x + textWidth / 2 + GRAPH_LABEL_PADDING : textAnchor === 'start' ? x + textWidth + GRAPH_LABEL_PADDING : x + GRAPH_LABEL_PADDING
    const top = y - GRAPH_LABEL_FONT_SIZE - GRAPH_LABEL_PADDING
    const bottom = y + GRAPH_LABEL_PADDING

    const priority = isSelected
      ? 1000
      : node.nodeType === 'rule'
        ? 500 + node.count
        : node.nodeType === 'file'
          ? 300 + node.count
          : 100

    candidates.push({
      nodeId: node.id,
      x,
      y,
      textAnchor,
      text,
      priority,
      bbox: { left, right, top, bottom },
    })
  }

  candidates.sort((left, right) => right.priority - left.priority)
  const accepted: LabelCandidate[] = []

  for (const candidate of candidates) {
    if (
      accepted.some((current) =>
        !(candidate.bbox.right < current.bbox.left
          || candidate.bbox.left > current.bbox.right
          || candidate.bbox.bottom < current.bbox.top
          || candidate.bbox.top > current.bbox.bottom),
      )
    ) {
      continue
    }
    accepted.push(candidate)
  }

  return new Map(accepted.map((candidate) => [candidate.nodeId, candidate]))
}

export function FindingsGraph({ nodes, edges, selectedFinding, onNodeSelect }: FindingsGraphProps) {
  const positions = positionNodes(nodes)
  const labels = buildLabelCandidates(nodes, positions, selectedFinding)

  return (
    <section className="findings-panel findings-graph-panel">
      <div className="findings-panel-head">
        <div>
          <span className="eyebrow">Graph explorer</span>
          <h2 className="findings-section-title">Rule to file to finding, scoped for readability instead of brute-force rendering.</h2>
        </div>
        <div className="findings-graph-legend">
          <span><i className="findings-dot findings-dot-rule" /> Rules</span>
          <span><i className="findings-dot findings-dot-file" /> Files</span>
          <span><i className="findings-dot findings-dot-finding" /> Findings</span>
        </div>
      </div>

      <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} className="findings-graph-svg" role="img" aria-label="Rule to file to finding graph">
        <defs>
          <linearGradient id="findings-edge-gradient" x1="0" x2="1">
            <stop offset="0%" stopColor="rgba(236, 189, 119, 0.85)" />
            <stop offset="100%" stopColor="rgba(100, 181, 246, 0.4)" />
          </linearGradient>
        </defs>

        {edges.map((edge) => {
          const source = positions.get(edge.source)
          const target = positions.get(edge.target)
          if (!source || !target) {
            return null
          }
          const controlX = (source.x + target.x) / 2
          const controlY = source.y < target.y ? source.y - 42 : source.y + 42
          return (
            <path
              key={edge.id}
              d={`M ${source.x} ${source.y} Q ${controlX} ${controlY} ${target.x} ${target.y}`}
              className="findings-graph-edge"
              style={{ opacity: Math.min(0.85, 0.22 + edge.count / 12) }}
            />
          )
        })}

        {nodes.map((node) => {
          const point = positions.get(node.id)
          if (!point) {
            return null
          }
          const isSelected = node.nodeType === 'finding' && node.findingId === selectedFinding?.id
          return (
            <g key={node.id} transform={`translate(${point.x}, ${point.y})`} className="findings-graph-node-wrap">
              <circle
                r={nodeRadius(node)}
                className={`findings-graph-node findings-graph-node-${node.nodeType}${isSelected ? ' findings-graph-node-selected' : ''}`}
                onClick={() => onNodeSelect(node)}
              />
              {labels.has(node.id) ? (
                <text
                  className="findings-graph-label"
                  textAnchor={labels.get(node.id)?.textAnchor}
                  x={labels.get(node.id)?.x != null ? labels.get(node.id)!.x - point.x : 0}
                  y={labels.get(node.id)?.y != null ? labels.get(node.id)!.y - point.y : 0}
                >
                  {labels.get(node.id)?.text}
                </text>
              ) : null}
            </g>
          )
        })}
      </svg>
    </section>
  )
}
