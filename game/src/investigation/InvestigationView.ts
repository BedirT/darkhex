import type { Policy, StrategyConfig } from '../strategy/types'
import type { InfoStateOps } from '../engine/InfoStateOps'
import type { TreeNode } from './types'
import { buildTree, cellLabel } from './TreeBuilder'
import { DEFAULT_LAYOUT } from './TreeLayout'
import { TreeRenderer } from './TreeRenderer'
import { drawMiniBoard, miniBoardSize } from './MiniBoard'

const FONT = "'Nunito', -apple-system, BlinkMacSystemFont, sans-serif"
const CREAM = '#faf5ef'
const MAUVE_DARK = '#995a5a'
const MAUVE_LIGHT = '#e8d5d5'
const TEXT = '#4a3535'
const TEXT_MUTED = '#8a7070'
const GREEN = '#5a9a5e'
const SIDEBAR_WIDTH = 280

/**
 * Full-screen investigation overlay.
 * Composes TreeRenderer (canvas) + detail sidebar + bottom bar.
 */
export class InvestigationView {
  private parent: HTMLElement
  private overlay: HTMLDivElement | null = null
  private renderer: TreeRenderer | null = null
  private detailEl: HTMLDivElement | null = null
  private _onKeyDown: ((e: KeyboardEvent) => void) | null = null

  private root: TreeNode | null = null
  private rows = 0
  private cols = 0

  constructor(parent: HTMLElement) {
    this.parent = parent
  }

  async run(policy: Policy, config: StrategyConfig, infoOps: InfoStateOps): Promise<void> {
    this.rows = config.rows
    this.cols = config.cols

    // Build tree
    this.root = buildTree(policy, infoOps, config)

    // Create overlay
    this.overlay = document.createElement('div')
    this.overlay.style.cssText = `
      position: fixed; top: 0; left: 0; width: 100%; height: 100%;
      background: ${CREAM}; z-index: 90; display: flex; flex-direction: column;
      font-family: ${FONT}; color: ${TEXT};
    `

    // ── Top bar ─────────────────────────────────────────────────────
    const topBar = document.createElement('div')
    topBar.style.cssText = `
      display: flex; align-items: center; gap: 12px; padding: 10px 16px;
      border-bottom: 1px solid ${MAUVE_LIGHT}; flex-shrink: 0;
      background: ${CREAM};
    `

    const escBadge = document.createElement('span')
    escBadge.textContent = 'Esc = menu'
    escBadge.style.cssText = `
      font-size: 12px; font-weight: 700; color: ${TEXT_MUTED};
      background: ${MAUVE_LIGHT}; padding: 4px 10px; border-radius: 6px;
    `
    topBar.appendChild(escBadge)

    const fitBtn = this._makeBtn('Fit to View', MAUVE_LIGHT, TEXT)
    fitBtn.addEventListener('click', () => this.renderer?.fitToView())
    topBar.appendChild(fitBtn)

    const spacer = document.createElement('div')
    spacer.style.flex = '1'
    topBar.appendChild(spacer)

    const playerName = config.player === 0 ? 'Black' : 'White'
    const playerColor = config.player === 0 ? '#506080' : MAUVE_DARK
    const summary = document.createElement('span')
    summary.style.cssText = `font-size: 14px; font-weight: 700; color: ${TEXT_MUTED};`
    summary.innerHTML = `
      <span style="color: ${playerColor}; font-weight: 800;">${playerName}</span>
      &nbsp;${config.rows}x${config.cols}
      &nbsp;&middot;&nbsp;${policy.size} info states
    `
    topBar.appendChild(summary)

    this.overlay.appendChild(topBar)

    // ── Main area: canvas + sidebar ─────────────────────────────────
    const main = document.createElement('div')
    main.style.cssText = 'display: flex; flex: 1; overflow: hidden;'

    const canvasWrap = document.createElement('div')
    canvasWrap.style.cssText = 'flex: 1; position: relative; overflow: hidden;'

    const canvas = document.createElement('canvas')
    canvas.style.cssText = 'width: 100%; height: 100%; display: block;'
    canvasWrap.appendChild(canvas)
    main.appendChild(canvasWrap)

    // Detail sidebar
    this.detailEl = document.createElement('div')
    this.detailEl.style.cssText = `
      width: ${SIDEBAR_WIDTH}px; flex-shrink: 0; overflow-y: auto;
      border-left: 1px solid ${MAUVE_LIGHT}; padding: 16px;
      display: none; flex-direction: column; gap: 12px;
      background: #fff;
    `
    main.appendChild(this.detailEl)

    this.overlay.appendChild(main)
    this.parent.appendChild(this.overlay)

    // ── Create renderer ─────────────────────────────────────────────
    this.renderer = new TreeRenderer(
      canvas, this.root, config.rows, config.cols, DEFAULT_LAYOUT,
      {
        onNodeClick: (node) => this._showDetail(node, config),
        onNodeDoubleClick: (node) => this._toggleCollapse(node),
      },
    )
    this.renderer.fitToView()

    // ── Keyboard ────────────────────────────────────────────────────
    return new Promise<void>((resolve) => {
      this._onKeyDown = (e: KeyboardEvent) => {
        if (e.key === 'Escape') {
          resolve()
        }
      }
      window.addEventListener('keydown', this._onKeyDown)
    })
  }

  dispose(): void {
    if (this._onKeyDown) window.removeEventListener('keydown', this._onKeyDown)
    this.renderer?.dispose()
    this.overlay?.remove()
    this.overlay = null
    this.renderer = null
  }

  // ── Detail sidebar ──────────────────────────────────────────────────────────

  private _showDetail(node: TreeNode, config: StrategyConfig): void {
    if (!this.detailEl) return
    this.renderer?.selectNode(node)
    this.detailEl.style.display = 'flex'
    this.detailEl.innerHTML = ''

    // Enlarged mini board
    const boardCanvas = document.createElement('canvas')
    const detailCellSize = 14
    const { w, h } = miniBoardSize(this.rows, this.cols, detailCellSize)
    const pad = 16
    boardCanvas.width = (w + pad) * 2
    boardCanvas.height = (h + pad) * 2
    boardCanvas.style.cssText = `width: ${w + pad}px; height: ${h + pad}px; display: block; margin: 0 auto;`
    const bctx = boardCanvas.getContext('2d')!
    bctx.scale(2, 2) // retina
    drawMiniBoard(
      bctx,
      (w + pad) / 2,
      (h + pad) / 2,
      node.boardView,
      this.rows,
      this.cols,
      detailCellSize,
      node.actions.length > 0
        ? { highlightActions: node.actions.map(([a]) => a) }
        : undefined,
    )
    this.detailEl.appendChild(boardCanvas)

    // Info state string
    const infoLabel = document.createElement('div')
    infoLabel.style.cssText = `font-size: 11px; font-weight: 700; color: ${TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px;`
    infoLabel.textContent = 'Info State'
    this.detailEl.appendChild(infoLabel)

    const infoStr = document.createElement('pre')
    infoStr.textContent = node.infoState
    infoStr.style.cssText = `
      font-size: 13px; font-family: 'SF Mono', 'Fira Code', monospace;
      background: ${CREAM}; border: 1px solid ${MAUVE_LIGHT}; border-radius: 6px;
      padding: 8px 10px; margin: 0; overflow-x: auto; white-space: pre;
      color: ${TEXT};
    `
    this.detailEl.appendChild(infoStr)

    // Actions table
    if (node.actions.length > 0) {
      const actLabel = document.createElement('div')
      actLabel.style.cssText = `font-size: 11px; font-weight: 700; color: ${TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px;`
      actLabel.textContent = 'Actions'
      this.detailEl.appendChild(actLabel)

      const table = document.createElement('div')
      table.style.cssText = `display: flex; flex-direction: column; gap: 4px;`
      for (const [action, prob] of node.actions) {
        const row = document.createElement('div')
        row.style.cssText = `
          display: flex; justify-content: space-between; align-items: center;
          padding: 6px 10px; background: ${CREAM}; border-radius: 6px;
          font-size: 14px;
        `
        const name = document.createElement('span')
        name.style.cssText = `font-weight: 700; color: ${GREEN};`
        name.textContent = cellLabel(action, config.cols)
        const val = document.createElement('span')
        val.style.cssText = `font-weight: 600; color: ${TEXT};`
        val.textContent = prob.toFixed(3)
        row.appendChild(name)
        row.appendChild(val)
        table.appendChild(row)
      }
      this.detailEl.appendChild(table)
    }

    // Metadata
    const metaLabel = document.createElement('div')
    metaLabel.style.cssText = `font-size: 11px; font-weight: 700; color: ${TEXT_MUTED}; text-transform: uppercase; letter-spacing: 0.5px; margin-top: 4px;`
    metaLabel.textContent = 'Details'
    this.detailEl.appendChild(metaLabel)

    const meta = document.createElement('div')
    meta.style.cssText = `font-size: 13px; color: ${TEXT_MUTED}; line-height: 1.6;`
    const items = [
      `Depth: ${node.depth}`,
      `Children: ${node.children.length}`,
      `Subtree: ${node.subtreeSize} nodes`,
      node.isTerminal ? `Terminal` : null,
    ].filter(Boolean)
    meta.textContent = items.join(' \u00B7 ')
    this.detailEl.appendChild(meta)

    // Expand/Collapse button (if node has children)
    if (node.children.length > 0) {
      const toggleBtn = this._makeBtn(
        node.collapsed ? 'Expand Subtree' : 'Collapse Subtree',
        MAUVE_LIGHT,
        TEXT,
      )
      toggleBtn.style.marginTop = '8px'
      toggleBtn.addEventListener('click', () => {
        this._toggleCollapse(node)
        this._showDetail(node, config) // refresh
      })
      this.detailEl.appendChild(toggleBtn)
    }
  }

  private _toggleCollapse(node: TreeNode): void {
    if (node.children.length === 0) return
    node.collapsed = !node.collapsed
    this.renderer?.relayout()
  }

  private _makeBtn(text: string, bg: string, fg: string): HTMLButtonElement {
    const btn = document.createElement('button')
    btn.textContent = text
    btn.style.cssText = `
      padding: 8px 14px; border-radius: 8px; cursor: pointer;
      font-family: ${FONT}; font-size: 13px; font-weight: 700;
      background: ${bg}; color: ${fg}; border: 1px solid ${MAUVE_LIGHT};
      transition: filter 0.12s;
    `
    btn.addEventListener('mouseover', () => { btn.style.filter = 'brightness(0.94)' })
    btn.addEventListener('mouseout', () => { btn.style.filter = '' })
    return btn
  }
}
