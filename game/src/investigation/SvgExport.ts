/**
 * Export the strategy tree as a publication-quality SVG.
 *
 * Walks the same TreeNode / TreeEdge structures used by TreeRenderer
 * and emits SVG elements with matching geometry and style tokens.
 */

import type { TreeNode } from './types'
import type { LayoutConfig } from './TreeLayout'
import { collectVisibleNodes, collectVisibleEdges } from './TreeLayout'

// ── Style tokens (same as TreeRenderer / InvestigationView) ─────────────────

const CREAM = '#faf5ef'
const MAUVE_DARK = '#995a5a'
const MAUVE_LIGHT = '#e8d5d5'
const TEXT = '#4a3535'
const TEXT_MUTED = '#8a7070'
const COLLISION_COLOR = '#c75000'
const FONT = "'Nunito', -apple-system, BlinkMacSystemFont, sans-serif"

const NODE_RADIUS = 8
const EDGE_LABEL_FONT_SIZE = 11

// MiniBoard colors
const MB_EMPTY = '#e0b8b8'
const MB_BLACK = '#506080'
const MB_WHITE = '#e8e0d8'
const MB_HIGHLIGHT = '#81c784'
const MB_OUTLINE = '#2a2a30'
const MB_EDGE_BLACK = '#506080'
const MB_EDGE_WHITE = '#c8beb4'

const SQRT3 = Math.sqrt(3)

// ── Hex math (duplicated from MiniBoard.ts — pure arithmetic) ───────────────

function hexCenter(row: number, col: number, r: number): [number, number] {
  const x = SQRT3 * r * col + (SQRT3 / 2) * r * row
  const y = 1.5 * r * row
  return [x, y]
}

function vtx(cx: number, cy: number, r: number, i: number): [number, number] {
  const angle = -Math.PI / 2 + (Math.PI / 3) * i
  return [cx + r * Math.cos(angle), cy + r * Math.sin(angle)]
}

function fmt(n: number): string {
  return n.toFixed(2)
}

// ── SVG mini board ──────────────────────────────────────────────────────────

function svgMiniBoard(
  cx: number,
  cy: number,
  boardView: Int8Array,
  rows: number,
  cols: number,
  cellSize: number,
  highlightActions?: number[],
): string {
  const highlightSet = highlightActions ? new Set(highlightActions) : null
  const r = cellSize * 0.92

  // Center the board at (cx, cy)
  const [firstX, firstY] = hexCenter(0, 0, cellSize)
  const [lastX, lastY] = hexCenter(rows - 1, cols - 1, cellSize)
  const ox = cx - (firstX + lastX) / 2
  const oy = cy - (firstY + lastY) / 2

  const parts: string[] = []

  // Hex cells
  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const idx = row * cols + col
      const [hx, hy] = hexCenter(row, col, cellSize)
      const px = hx + ox
      const py = hy + oy

      const cell = boardView[idx]
      let fill: string
      if (highlightSet?.has(idx)) fill = MB_HIGHLIGHT
      else if (cell === 1) fill = MB_BLACK
      else if (cell === 2) fill = MB_WHITE
      else fill = MB_EMPTY

      const points: string[] = []
      for (let i = 0; i < 6; i++) {
        const [vx, vy] = vtx(px, py, r, i)
        points.push(`${fmt(vx)},${fmt(vy)}`)
      }
      parts.push(
        `<polygon points="${points.join(' ')}" fill="${fill}" stroke="${MB_OUTLINE}" stroke-width="${fmt(Math.max(0.5, cellSize * 0.08))}"/>`,
      )
    }
  }

  const lw = fmt(Math.max(1.5, cellSize * 0.25))

  // North border (row 0) — Black
  for (let col = 0; col < cols; col++) {
    const [hx, hy] = hexCenter(0, col, cellSize)
    const px = hx + ox, py = hy + oy
    const [x5, y5] = vtx(px, py, r, 5)
    const [x0, y0] = vtx(px, py, r, 0)
    const [x1, y1] = vtx(px, py, r, 1)
    parts.push(
      `<polyline points="${fmt(x5)},${fmt(y5)} ${fmt(x0)},${fmt(y0)} ${fmt(x1)},${fmt(y1)}" fill="none" stroke="${MB_EDGE_BLACK}" stroke-width="${lw}"/>`,
    )
  }

  // South border (last row) — Black
  for (let col = 0; col < cols; col++) {
    const [hx, hy] = hexCenter(rows - 1, col, cellSize)
    const px = hx + ox, py = hy + oy
    const [x2, y2] = vtx(px, py, r, 2)
    const [x3, y3] = vtx(px, py, r, 3)
    const [x4, y4] = vtx(px, py, r, 4)
    parts.push(
      `<polyline points="${fmt(x2)},${fmt(y2)} ${fmt(x3)},${fmt(y3)} ${fmt(x4)},${fmt(y4)}" fill="none" stroke="${MB_EDGE_BLACK}" stroke-width="${lw}"/>`,
    )
  }

  // West border (col 0) — White
  for (let row = 0; row < rows; row++) {
    const [hx, hy] = hexCenter(row, 0, cellSize)
    const px = hx + ox, py = hy + oy
    const [x5, y5] = vtx(px, py, r, 5)
    const [x4, y4] = vtx(px, py, r, 4)
    parts.push(
      `<line x1="${fmt(x5)}" y1="${fmt(y5)}" x2="${fmt(x4)}" y2="${fmt(y4)}" stroke="${MB_EDGE_WHITE}" stroke-width="${lw}"/>`,
    )
  }

  // East border (last col) — White
  for (let row = 0; row < rows; row++) {
    const [hx, hy] = hexCenter(row, cols - 1, cellSize)
    const px = hx + ox, py = hy + oy
    const [x1, y1] = vtx(px, py, r, 1)
    const [x2, y2] = vtx(px, py, r, 2)
    parts.push(
      `<line x1="${fmt(x1)}" y1="${fmt(y1)}" x2="${fmt(x2)}" y2="${fmt(y2)}" stroke="${MB_EDGE_WHITE}" stroke-width="${lw}"/>`,
    )
  }

  return parts.join('\n')
}

// ── SVG export ──────────────────────────────────────────────────────────────

export function exportTreeSvg(
  root: TreeNode,
  rows: number,
  cols: number,
  layout: LayoutConfig,
): string {
  const nodes = collectVisibleNodes(root)
  const edges = collectVisibleEdges(root)
  const { nodeWidth, nodeHeight } = layout

  // Compute hex cell size (same logic as TreeRenderer)
  const maxDim = Math.max(rows, cols)
  const cellSize = Math.max(4, Math.min(8, Math.floor((nodeWidth - 8 * 2) / (maxDim * 1.8))))

  // Compute bounding box from laid-out nodes
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity
  for (const node of nodes) {
    const nx = node.x - nodeWidth / 2
    minX = Math.min(minX, nx)
    minY = Math.min(minY, node.y)
    maxX = Math.max(maxX, nx + nodeWidth)
    // Account for collapse badge below node
    const bottomExtra = (node.collapsed && node.children.length > 0) ? 24 : 0
    maxY = Math.max(maxY, node.y + nodeHeight + bottomExtra)
  }

  const pad = 40
  const svgW = maxX - minX + pad * 2
  const svgH = maxY - minY + pad * 2
  const offX = -minX + pad
  const offY = -minY + pad

  const parts: string[] = []

  // SVG header
  parts.push(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${fmt(svgW)} ${fmt(svgH)}" width="${fmt(svgW)}" height="${fmt(svgH)}">`,
  )

  // Embedded style
  parts.push(`<style>
  text { font-family: ${FONT}; }
  .edge-label { font-size: ${EDGE_LABEL_FONT_SIZE}px; }
  .terminal-badge { font-size: 8px; font-weight: bold; fill: #fff; }
  .collapse-badge { font-size: 10px; font-weight: bold; fill: ${TEXT}; }
</style>`)

  // Background
  parts.push(`<rect width="100%" height="100%" fill="${CREAM}"/>`)

  // Offset group
  parts.push(`<g transform="translate(${fmt(offX)},${fmt(offY)})">`)

  // ── Edges ──
  parts.push('<g class="edges">')
  for (const { parent, edge } of edges) {
    const x0 = parent.x
    const y0 = parent.y + nodeHeight
    const x1 = edge.child.x
    const y1 = edge.child.y
    const cpY = (y0 + y1) / 2

    const stroke = edge.isCollision ? COLLISION_COLOR : TEXT_MUTED
    const dash = edge.isCollision ? ' stroke-dasharray="4,3"' : ''

    parts.push(
      `<path d="M ${fmt(x0)} ${fmt(y0)} C ${fmt(x0)} ${fmt(cpY)}, ${fmt(x1)} ${fmt(cpY)}, ${fmt(x1)} ${fmt(y1)}" fill="none" stroke="${stroke}" stroke-width="1.5"${dash}/>`,
    )

    // Edge label at midpoint
    const mx = (x0 + x1) / 2
    const my = (y0 + y1) / 2
    const label = edge.isCollision
      ? `${edge.label}(c)`
      : `${edge.label}: ${edge.probability.toFixed(2)}`

    // Approximate text width (rough: 6.5px per char at 11px font)
    const tw = label.length * 6.5
    const pillW = tw + 8
    const pillH = 16

    parts.push(
      `<rect x="${fmt(mx - pillW / 2)}" y="${fmt(my - pillH / 2)}" width="${fmt(pillW)}" height="${fmt(pillH)}" rx="4" fill="#fff" opacity="0.85"/>`,
    )
    parts.push(
      `<text x="${fmt(mx)}" y="${fmt(my)}" text-anchor="middle" dominant-baseline="central" class="edge-label" fill="${edge.isCollision ? COLLISION_COLOR : TEXT}">${escapeXml(label)}</text>`,
    )
  }
  parts.push('</g>')

  // ── Nodes ──
  parts.push('<g class="nodes">')
  for (const node of nodes) {
    const x = node.x - nodeWidth / 2
    const y = node.y

    // Node background
    parts.push(
      `<rect x="${fmt(x)}" y="${fmt(y)}" width="${nodeWidth}" height="${nodeHeight}" rx="${NODE_RADIUS}" fill="#fff" stroke="${MAUVE_LIGHT}" stroke-width="1.5"/>`,
    )

    // Mini board
    parts.push(
      `<g>`,
      svgMiniBoard(
        node.x,
        y + nodeHeight / 2,
        node.boardView,
        rows,
        cols,
        cellSize,
        node.actions.length > 0 ? node.actions.map(([a]) => a) : undefined,
      ),
      `</g>`,
    )

    // Terminal badge
    if (node.isTerminal) {
      const badgeW = 28
      const badgeH = 12
      const bx = x + nodeWidth - badgeW - 3
      const by = y + 3
      parts.push(
        `<rect x="${fmt(bx)}" y="${fmt(by)}" width="${badgeW}" height="${badgeH}" rx="3" fill="${MAUVE_DARK}"/>`,
        `<text x="${fmt(bx + badgeW / 2)}" y="${fmt(by + badgeH / 2)}" text-anchor="middle" dominant-baseline="central" class="terminal-badge">END</text>`,
      )
    }

    // Missing policy badge
    if (node.isMissing) {
      const badgeW = 12
      const badgeH = 12
      const bx = x + nodeWidth - badgeW - 3
      const by = y + 3
      parts.push(
        `<rect x="${fmt(bx)}" y="${fmt(by)}" width="${badgeW}" height="${badgeH}" rx="3" fill="#d4920a"/>`,
        `<text x="${fmt(bx + badgeW / 2)}" y="${fmt(by + badgeH / 2)}" text-anchor="middle" dominant-baseline="central" style="font-size:9px;font-weight:bold;fill:#fff">?</text>`,
      )
    }

    // Collapse badge
    if (node.collapsed && node.children.length > 0) {
      const label = `+${node.subtreeSize}`
      const tw = label.length * 7
      const badgeW = tw + 8
      const badgeH = 16
      const bx = node.x - badgeW / 2
      const by = y + nodeHeight + 4
      parts.push(
        `<rect x="${fmt(bx)}" y="${fmt(by)}" width="${fmt(badgeW)}" height="${badgeH}" rx="4" fill="${MAUVE_LIGHT}" stroke="${MAUVE_DARK}" stroke-width="1"/>`,
        `<text x="${fmt(node.x)}" y="${fmt(by + badgeH / 2)}" text-anchor="middle" dominant-baseline="central" class="collapse-badge">${escapeXml(label)}</text>`,
      )
    }
  }
  parts.push('</g>')

  // Close offset group and SVG
  parts.push('</g>')
  parts.push('</svg>')

  return parts.join('\n')
}

// ── Download helper ─────────────────────────────────────────────────────────

export function downloadSvg(svgString: string, filename: string): void {
  const blob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

// ── Utility ─────────────────────────────────────────────────────────────────

function escapeXml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}
