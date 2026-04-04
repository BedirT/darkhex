import type { TreeNode, TreeEdge } from './types'
import { layoutTree, collectVisibleNodes, collectVisibleEdges, DEFAULT_LAYOUT } from './TreeLayout'
import type { LayoutConfig } from './TreeLayout'
import { drawMiniBoard } from './MiniBoard'

// Style tokens matching the warm board-game palette
const CREAM = '#faf5ef'
const MAUVE_DARK = '#995a5a'
const MAUVE_LIGHT = '#e8d5d5'
const TEXT = '#4a3535'
const TEXT_MUTED = '#8a7070'
const GREEN = '#5a9a5e'
// green border available for future use: '#b5d8b7'
const COLLISION_COLOR = '#c75000'
const FONT = "'Nunito', -apple-system, BlinkMacSystemFont, sans-serif"

const NODE_RADIUS = 8            // rounded rect corner radius
const NODE_PADDING = 8           // padding around mini board
const EDGE_LABEL_FONT_SIZE = 11
const MIN_ZOOM = 0.1
const MAX_ZOOM = 3.0
const ZOOM_FACTOR = 1.08
const DOUBLE_CLICK_MS = 350

export interface TreeRendererCallbacks {
  onNodeClick: (node: TreeNode) => void
  onNodeDoubleClick: (node: TreeNode) => void
}

export class TreeRenderer {
  private canvas: HTMLCanvasElement
  private ctx: CanvasRenderingContext2D
  private root: TreeNode
  private rows: number
  private cols: number
  private layout: LayoutConfig
  private callbacks: TreeRendererCallbacks

  // Camera
  private camX = 0
  private camY = 0
  private camScale = 1

  // Interaction state
  private dragging = false
  private dragStartX = 0
  private dragStartY = 0
  private camStartX = 0
  private camStartY = 0
  private selectedNode: TreeNode | null = null
  private lastClickTime = 0
  private lastClickNode: TreeNode | null = null

  // Cached visible elements
  private visibleNodes: TreeNode[] = []
  private visibleEdges: Array<{ parent: TreeNode; edge: TreeEdge }> = []
  private treeWidth = 0
  private treeHeight = 0

  // RAF
  private _rafId = 0
  private _dirty = true

  // Hex cell size for tree node thumbnails
  private cellSize: number
  // Logical (CSS) dimensions — stable reference for layout/hit-testing
  private cssWidth = 0
  private cssHeight = 0

  constructor(
    canvas: HTMLCanvasElement,
    root: TreeNode,
    rows: number,
    cols: number,
    layoutConfig: LayoutConfig = DEFAULT_LAYOUT,
    callbacks: TreeRendererCallbacks,
  ) {
    this.canvas = canvas
    this.ctx = canvas.getContext('2d')!
    this.root = root
    this.rows = rows
    this.cols = cols
    this.layout = layoutConfig
    this.callbacks = callbacks

    // Scale hex cells to fit inside node box
    const maxDim = Math.max(rows, cols)
    this.cellSize = Math.max(4, Math.min(8, Math.floor((layoutConfig.nodeWidth - NODE_PADDING * 2) / (maxDim * 1.8))))

    this._resizeCanvas()
    this.relayout()

    // Event listeners
    this.canvas.addEventListener('wheel', this._onWheel, { passive: false })
    this.canvas.addEventListener('pointerdown', this._onPointerDown)
    this.canvas.addEventListener('pointermove', this._onPointerMove)
    this.canvas.addEventListener('pointerup', this._onPointerUp)
    this.canvas.addEventListener('pointerleave', this._onPointerUp)
    window.addEventListener('resize', this._onResize)

    this._rafId = requestAnimationFrame(this._animate)
  }

  // ── Public API ─────────────────────────────────────────────────────────────

  relayout(): void {
    const bounds = layoutTree(this.root, this.layout)
    this.treeWidth = bounds.width
    this.treeHeight = bounds.height
    this.visibleNodes = collectVisibleNodes(this.root)
    this.visibleEdges = collectVisibleEdges(this.root)
    this._dirty = true
  }

  selectNode(node: TreeNode | null): void {
    this.selectedNode = node
    this._dirty = true
  }

  fitToView(): void {
    if (this.treeWidth === 0 || this.treeHeight === 0) return
    const cw = this.cssWidth
    const ch = this.cssHeight
    const pad = 40
    const scaleX = (cw - pad * 2) / this.treeWidth
    const scaleY = (ch - pad * 2) / this.treeHeight
    this.camScale = Math.min(scaleX, scaleY, MAX_ZOOM)
    this.camScale = Math.max(this.camScale, MIN_ZOOM)
    this.camX = (cw - this.treeWidth * this.camScale) / 2
    this.camY = (ch - this.treeHeight * this.camScale) / 2 + pad / 2
    this._dirty = true
  }

  dispose(): void {
    cancelAnimationFrame(this._rafId)
    this.canvas.removeEventListener('wheel', this._onWheel)
    this.canvas.removeEventListener('pointerdown', this._onPointerDown)
    this.canvas.removeEventListener('pointermove', this._onPointerMove)
    this.canvas.removeEventListener('pointerup', this._onPointerUp)
    this.canvas.removeEventListener('pointerleave', this._onPointerUp)
    window.removeEventListener('resize', this._onResize)
  }

  // ── Render loop ────────────────────────────────────────────────────────────

  private _animate = (): void => {
    this._rafId = requestAnimationFrame(this._animate)
    if (!this._dirty) return
    this._dirty = false
    this._draw()
  }

  private _draw(): void {
    const ctx = this.ctx
    const dpr = window.devicePixelRatio || 1

    // Clear (in physical pixels, before DPR transform)
    ctx.setTransform(1, 0, 0, 1, 0, 0)
    ctx.fillStyle = CREAM
    ctx.fillRect(0, 0, this.canvas.width, this.canvas.height)

    // Apply DPR then camera transform (all subsequent draws in CSS pixel space)
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    ctx.translate(this.camX, this.camY)
    ctx.scale(this.camScale, this.camScale)

    // Draw edges first (behind nodes)
    for (const { parent, edge } of this.visibleEdges) {
      this._drawEdge(ctx, parent, edge)
    }

    // Draw nodes
    for (const node of this.visibleNodes) {
      this._drawNode(ctx, node)
    }

    ctx.restore()
  }

  private _drawNode(ctx: CanvasRenderingContext2D, node: TreeNode): void {
    const { nodeWidth, nodeHeight } = this.layout
    const x = node.x - nodeWidth / 2
    const y = node.y

    // Background
    const isSelected = node === this.selectedNode
    ctx.beginPath()
    this._roundRect(ctx, x, y, nodeWidth, nodeHeight, NODE_RADIUS)
    ctx.fillStyle = '#fff'
    ctx.fill()
    ctx.strokeStyle = isSelected ? GREEN : MAUVE_LIGHT
    ctx.lineWidth = isSelected ? 2.5 : 1.5
    ctx.stroke()

    // Mini board
    drawMiniBoard(
      ctx,
      node.x,
      y + nodeHeight / 2,
      node.boardView,
      this.rows,
      this.cols,
      this.cellSize,
      node.actions.length > 0
        ? { highlightActions: node.actions.map(([a]) => a) }
        : undefined,
    )

    // Terminal badge
    if (node.isTerminal) {
      const badgeW = 28
      const badgeH = 12
      const bx = x + nodeWidth - badgeW - 3
      const by = y + 3
      ctx.beginPath()
      this._roundRect(ctx, bx, by, badgeW, badgeH, 3)
      ctx.fillStyle = MAUVE_DARK
      ctx.fill()
      ctx.fillStyle = '#fff'
      ctx.font = `bold 8px ${FONT}`
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.fillText('END', bx + badgeW / 2, by + badgeH / 2)
    }

    // Collapse badge: show "+N" if collapsed and has children
    if (node.collapsed && node.children.length > 0) {
      const label = `+${node.subtreeSize}`
      ctx.font = `bold 10px ${FONT}`
      const tw = ctx.measureText(label).width
      const badgeW = tw + 8
      const badgeH = 16
      const bx = node.x - badgeW / 2
      const by = y + nodeHeight + 4
      ctx.beginPath()
      this._roundRect(ctx, bx, by, badgeW, badgeH, 4)
      ctx.fillStyle = MAUVE_LIGHT
      ctx.fill()
      ctx.strokeStyle = MAUVE_DARK
      ctx.lineWidth = 1
      ctx.stroke()
      ctx.fillStyle = TEXT
      ctx.textAlign = 'center'
      ctx.textBaseline = 'middle'
      ctx.fillText(label, node.x, by + badgeH / 2)
    }
  }

  private _drawEdge(ctx: CanvasRenderingContext2D, parent: TreeNode, edge: TreeEdge): void {
    const { nodeHeight } = this.layout
    const child = edge.child

    const x0 = parent.x
    const y0 = parent.y + nodeHeight
    const x1 = child.x
    const y1 = child.y

    // Bezier curve
    const cpY = (y0 + y1) / 2
    ctx.beginPath()
    ctx.moveTo(x0, y0)
    ctx.bezierCurveTo(x0, cpY, x1, cpY, x1, y1)

    if (edge.isCollision) {
      ctx.strokeStyle = COLLISION_COLOR
      ctx.setLineDash([4, 3])
    } else {
      ctx.strokeStyle = TEXT_MUTED
      ctx.setLineDash([])
    }
    ctx.lineWidth = 1.5
    ctx.stroke()
    ctx.setLineDash([])

    // Edge label at midpoint
    const mx = (x0 + x1) / 2
    const my = (y0 + y1) / 2
    const label = edge.isCollision
      ? `${edge.label}(c)`
      : `${edge.label}: ${edge.probability.toFixed(2)}`

    ctx.font = `${EDGE_LABEL_FONT_SIZE}px ${FONT}`
    const tw = ctx.measureText(label).width
    // Background pill
    ctx.fillStyle = '#fff'
    ctx.globalAlpha = 0.85
    ctx.beginPath()
    this._roundRect(ctx, mx - tw / 2 - 4, my - 8, tw + 8, 16, 4)
    ctx.fill()
    ctx.globalAlpha = 1.0
    // Text
    ctx.fillStyle = edge.isCollision ? COLLISION_COLOR : TEXT
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillText(label, mx, my)
  }

  private _roundRect(
    ctx: CanvasRenderingContext2D,
    x: number, y: number, w: number, h: number, r: number,
  ): void {
    ctx.moveTo(x + r, y)
    ctx.arcTo(x + w, y, x + w, y + h, r)
    ctx.arcTo(x + w, y + h, x, y + h, r)
    ctx.arcTo(x, y + h, x, y, r)
    ctx.arcTo(x, y, x + w, y, r)
  }

  // ── Hit testing ────────────────────────────────────────────────────────────

  private _hitTestNode(screenX: number, screenY: number): TreeNode | null {
    // Convert screen coords to tree coords
    const tx = (screenX - this.camX) / this.camScale
    const ty = (screenY - this.camY) / this.camScale
    const { nodeWidth, nodeHeight } = this.layout

    for (let i = this.visibleNodes.length - 1; i >= 0; i--) {
      const node = this.visibleNodes[i]
      const nx = node.x - nodeWidth / 2
      const ny = node.y
      if (tx >= nx && tx <= nx + nodeWidth && ty >= ny && ty <= ny + nodeHeight) {
        return node
      }
    }
    return null
  }

  // ── Event handlers ─────────────────────────────────────────────────────────

  private _resizeCanvas(): void {
    const rect = this.canvas.parentElement!.getBoundingClientRect()
    const dpr = window.devicePixelRatio || 1
    this.cssWidth = rect.width
    this.cssHeight = rect.height
    this.canvas.width = rect.width * dpr
    this.canvas.height = rect.height * dpr
    this.canvas.style.width = `${rect.width}px`
    this.canvas.style.height = `${rect.height}px`
    this._dirty = true
  }

  private _onResize = (): void => {
    this._resizeCanvas()
  }

  private _onWheel = (e: WheelEvent): void => {
    e.preventDefault()
    const rect = this.canvas.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top

    const oldScale = this.camScale
    const factor = e.deltaY < 0 ? ZOOM_FACTOR : 1 / ZOOM_FACTOR
    this.camScale = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, this.camScale * factor))

    // Zoom towards cursor
    this.camX = mx - (mx - this.camX) * (this.camScale / oldScale)
    this.camY = my - (my - this.camY) * (this.camScale / oldScale)
    this._dirty = true
  }

  private _onPointerDown = (e: PointerEvent): void => {
    const rect = this.canvas.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top

    // Check for node click (CSS pixel coords — camera is in CSS space)
    const node = this._hitTestNode(mx, my)

    if (node) {
      const now = Date.now()
      if (now - this.lastClickTime < DOUBLE_CLICK_MS && this.lastClickNode === node) {
        // Double-click: expand/collapse
        this.callbacks.onNodeDoubleClick(node)
        this.lastClickTime = 0
        this.lastClickNode = null
        return
      }
      this.lastClickTime = now
      this.lastClickNode = node
      this.callbacks.onNodeClick(node)
      return
    }

    // Start drag
    this.dragging = true
    this.dragStartX = e.clientX
    this.dragStartY = e.clientY
    this.camStartX = this.camX
    this.camStartY = this.camY
    this.canvas.style.cursor = 'grabbing'
  }

  private _onPointerMove = (e: PointerEvent): void => {
    if (!this.dragging) {
      // Hover cursor
      const rect = this.canvas.getBoundingClientRect()
      const node = this._hitTestNode(e.clientX - rect.left, e.clientY - rect.top)
      this.canvas.style.cursor = node ? 'pointer' : 'grab'
      return
    }
    const dx = e.clientX - this.dragStartX
    const dy = e.clientY - this.dragStartY
    this.camX = this.camStartX + dx
    this.camY = this.camStartY + dy
    this._dirty = true
  }

  private _onPointerUp = (): void => {
    if (this.dragging) {
      this.dragging = false
      this.canvas.style.cursor = 'grab'
    }
  }
}
