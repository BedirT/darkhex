import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'
import { BoardLayout3D } from '../board/BoardLayout'
import { HexTile3D } from '../board/HexTile'
import { boardCenter, PALETTE } from '../board/IsometricHex'
import { GameEngine, type Player } from '../engine/GameEngine'
import { InfoStateOps } from '../engine/InfoStateOps'
import { createOutlineComposer } from '../postprocess/OutlinePostProcess'
import { StrategyGenerator } from '../strategy/StrategyGenerator'
import { SetupPanel } from '../ui/SetupPanel'
import { ActionPanel } from '../ui/ActionPanel'
import { InfoPanel } from '../ui/InfoPanel'

const BOARD_ROWS = 4
const BOARD_COLS = 3
const PLAYER_LABEL: Record<Player, string> = { 0: 'Black', 1: 'White' }

type Mode = 'play' | 'strategy'

/**
 * Main 3D scene: orthographic camera at isometric angle,
 * warm lighting to match the mauve / board-game aesthetic.
 */
export class BoardScene {
  private renderer: THREE.WebGLRenderer
  private scene: THREE.Scene
  private camera: THREE.OrthographicCamera
  private controls: OrbitControls

  private engine!: GameEngine
  private board!: BoardLayout3D

  private outlineRender!: ReturnType<typeof createOutlineComposer>

  private raycaster = new THREE.Raycaster()
  private pointer = new THREE.Vector2()
  private hoveredTile: HexTile3D | null = null
  private lastMoveIndex: number | null = null

  private clock = new THREE.Clock()
  private _rafId = 0

  private statusEl!: HTMLDivElement
  private infoEl!: HTMLDivElement
  private hudTop!: HTMLDivElement
  private hudBottom!: HTMLDivElement

  // Strategy mode
  private mode: Mode = 'play'
  private stratGen: StrategyGenerator | null = null
  private setupPanel!: SetupPanel
  private actionPanel!: ActionPanel
  private infoPanel!: InfoPanel
  private selectedTiles: Map<number, number> = new Map() // cellIndex → 1 (tracked for selection)
  private probOverlay!: HTMLDivElement // container for probability labels
  private probLabels: Map<number, HTMLDivElement> = new Map()
  private _lastClickTime = 0
  private _lastClickTile = -1

  constructor(private container: HTMLElement) {
    // ── Renderer ──────────────────────────────────────────────────────────
    this.renderer = new THREE.WebGLRenderer({ antialias: true })
    this.renderer.setPixelRatio(window.devicePixelRatio)
    this.renderer.setSize(container.clientWidth, container.clientHeight)
    this.renderer.setClearColor(PALETTE.bg)
    this.renderer.shadowMap.enabled = true
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap
    this.renderer.toneMapping = THREE.NoToneMapping
    container.appendChild(this.renderer.domElement)

    // ── Scene ─────────────────────────────────────────────────────────────
    this.scene = new THREE.Scene()

    // ── Camera ────────────────────────────────────────────────────────────
    const aspect = container.clientWidth / container.clientHeight
    const frustum = 5.5
    this.camera = new THREE.OrthographicCamera(
      -frustum * aspect, frustum * aspect,
      frustum, -frustum,
      0.1, 100,
    )

    const [cx, , cz] = boardCenter(BOARD_ROWS, BOARD_COLS)
    // Isometric view matching the reference image angle
    const dist = 14
    this.camera.position.set(cx + dist * 0.55, dist * 0.7, cz + dist * 0.55)
    this.camera.lookAt(cx, 0, cz)

    // ── Orbit controls ────────────────────────────────────────────────────
    this.controls = new OrbitControls(this.camera, this.renderer.domElement)
    this.controls.target.set(cx, 0, cz)
    this.controls.enablePan = false
    this.controls.enableZoom = true
    this.controls.minZoom = 0.5
    this.controls.maxZoom = 3
    this.controls.minPolarAngle = 0
    this.controls.maxPolarAngle = Math.PI / 2  // allow fully horizontal
    this.controls.enableDamping = true
    this.controls.dampingFactor = 0.08
    this.controls.update()

    // ── Lights (warm, soft) ───────────────────────────────────────────────
    this._setupLights(cx, cz)

    // ── Post-processing (screen-space outlines) ──────────────────────────
    this.outlineRender = createOutlineComposer(this.renderer, this.scene, this.camera, {
      outlineColor: PALETTE.outline,
      depthThreshold: 2.0,     // only big silhouettes (object IDs handle tile boundaries)
      normalThreshold: 0.45,   // top/side crease edges
      thickness: 2,
    })

    // ── HUD ───────────────────────────────────────────────────────────────
    this._buildHud()

    // ── Events ────────────────────────────────────────────────────────────
    this.renderer.domElement.addEventListener('pointermove', this._onPointerMove)
    this.renderer.domElement.addEventListener('pointerdown', this._onPointerDown)
    window.addEventListener('resize', this._onResize)
    window.addEventListener('keydown', this._onKeyDown)
  }

  async init(): Promise<void> {
    this.engine = await GameEngine.create(BOARD_ROWS, BOARD_COLS)
    this.board = new BoardLayout3D(this.scene, BOARD_ROWS, BOARD_COLS)
    this.setupPanel = new SetupPanel(this.container)
    this.actionPanel = new ActionPanel(this.container)
    this.infoPanel = new InfoPanel(this.container)

    // Probability overlay container (positioned over canvas)
    this.probOverlay = document.createElement('div')
    this.probOverlay.style.cssText = 'position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 10;'
    this.container.appendChild(this.probOverlay)

    // Wire up strategy mode callbacks
    this.actionPanel.onConfirm((actions, probs) => this._handleConfirm(actions, probs))
    this.actionPanel.onUndo(() => this._strategyUndo())
    this.actionPanel.onRestart(() => this._strategyRestart())
    this.actionPanel.onRnd(() => this._handleStrategyAction('r'))
    this.actionPanel.onProbChange((probs) => this._syncProbOverlaysFromInputs(probs))

    this._updateHud()
    this._animate()
  }

  // ── Render loop ─────────────────────────────────────────────────────────

  private _animate = (): void => {
    this._rafId = requestAnimationFrame(this._animate)
    const dt = this.clock.getDelta()
    this.controls.update()
    if (this.board) this.board.tickAnimations(dt)
    if (this.mode === 'strategy') this._updateProbLabelPositions()
    this.outlineRender.render()
  }

  dispose(): void {
    cancelAnimationFrame(this._rafId)
    this.renderer.domElement.removeEventListener('pointermove', this._onPointerMove)
    this.renderer.domElement.removeEventListener('pointerdown', this._onPointerDown)
    window.removeEventListener('resize', this._onResize)
    window.removeEventListener('keydown', this._onKeyDown)
    this.controls.dispose()
    this.renderer.dispose()
  }

  // ── Lights ──────────────────────────────────────────────────────────────

  private _setupLights(cx: number, cz: number): void {
    // Strong neutral ambient — dominates so colors render close to their hex values
    this.scene.add(new THREE.AmbientLight(0xffffff, 1.2))

    // Gentle directional — just enough to create soft shadows and slight shading
    const key = new THREE.DirectionalLight(0xffffff, 0.4)
    key.position.set(cx + 6, 14, cz - 4)
    key.target.position.set(cx, 0, cz)
    key.castShadow = true
    key.shadow.mapSize.setScalar(2048)
    key.shadow.camera.left = -10
    key.shadow.camera.right = 10
    key.shadow.camera.top = 10
    key.shadow.camera.bottom = -10
    key.shadow.camera.near = 1
    key.shadow.camera.far = 40
    key.shadow.bias = -0.002
    this.scene.add(key)
    this.scene.add(key.target)
  }

  // ── Raycaster ─────────────────────────────────────────────────────────

  private _updatePointer(e: PointerEvent): void {
    const rect = this.renderer.domElement.getBoundingClientRect()
    this.pointer.x = ((e.clientX - rect.left) / rect.width) * 2 - 1
    this.pointer.y = -((e.clientY - rect.top) / rect.height) * 2 + 1
  }

  private _raycastTile(): HexTile3D | null {
    if (!this.board) return null
    this.raycaster.setFromCamera(this.pointer, this.camera)
    const hits = this.raycaster.intersectObjects(this.board.meshes, false)
    if (hits.length === 0) return null
    return this.board.tileFromMesh(hits[0].object)
  }

  private _onPointerMove = (e: PointerEvent): void => {
    this._updatePointer(e)
    const tile = this._raycastTile()
    if (tile !== this.hoveredTile) {
      if (this.hoveredTile) this.hoveredTile.setHovered(false)
      if (tile) tile.setHovered(true)
      this.hoveredTile = tile
    }
  }

  private _onPointerDown = (e: PointerEvent): void => {
    this._updatePointer(e)
    const tile = this._raycastTile()
    if (!tile) return

    if (this.mode === 'strategy') {
      if (!this.stratGen) return
      if (tile.state !== 'empty') return

      const now = Date.now()
      const isDoubleClick = (now - this._lastClickTime < 400) && tile.cellIndex === this._lastClickTile
      this._lastClickTime = now
      this._lastClickTile = tile.cellIndex

      if (isDoubleClick) {
        // Double-click: instant deterministic action
        this._clearSelection()
        this._handleConfirm([tile.cellIndex], [1.0])
        return
      }

      // Toggle tile selection
      if (this.selectedTiles.has(tile.cellIndex)) {
        this.selectedTiles.delete(tile.cellIndex)
        tile.setSelected(false)
      } else {
        this.selectedTiles.set(tile.cellIndex, 1)
        tile.setSelected(true)
      }
      this._syncSelectionUI()
      return
    }

    // Play mode
    if (this.engine.isTerminal) return
    const legal = this.engine.legalActions()
    if (!legal.includes(tile.cellIndex)) return

    const result = this.engine.applyAction(tile.cellIndex)
    this.lastMoveIndex = result.placed ? tile.cellIndex : null
    this.board.applyBoard(this.engine.trueBoard(), this.lastMoveIndex)
    this._updateHud()
  }

  private _onKeyDown = (e: KeyboardEvent): void => {
    if (this.mode === 'strategy') {
      if (e.key === 'Escape') this._exitStrategyMode()
      if (e.key === 'Enter' && this.selectedTiles.size > 0) {
        const result = this.actionPanel.getActionProbs()
        if (result) this._handleConfirm(result.actions, result.probs)
      }
      return
    }
    if (e.key === 'r' || e.key === 'R') this._reset()
    if (e.key === 's' || e.key === 'S') this._enterStrategyMode()
  }

  private async _reset(): Promise<void> {
    this.engine = await GameEngine.create(BOARD_ROWS, BOARD_COLS)
    this.lastMoveIndex = null
    this.board.applyBoard(this.engine.trueBoard(), null)
    this._updateHud()
  }

  // ── Strategy mode ─────────────────────────────────────────────────────────

  /** Remove all meshes/groups from the scene except lights. */
  private _clearBoardFromScene(): void {
    const toRemove: THREE.Object3D[] = []
    this.scene.traverse((obj) => {
      if (obj === this.scene) return
      // Keep lights and their targets
      if (obj instanceof THREE.Light || obj.parent instanceof THREE.Light) return
      // Only collect top-level children (not nested)
      if (obj.parent === this.scene) toRemove.push(obj)
    })
    for (const obj of toRemove) this.scene.remove(obj)
  }

  private _rebuildBoard(rows: number, cols: number): void {
    this._clearBoardFromScene()
    this.board = new BoardLayout3D(this.scene, rows, cols)
    this.outlineRender.invalidateMeshList()

    const [cx, , cz] = boardCenter(rows, cols)
    const dist = 14
    this.camera.position.set(cx + dist * 0.55, dist * 0.7, cz + dist * 0.55)
    this.camera.lookAt(cx, 0, cz)
    this.controls.target.set(cx, 0, cz)
    this.controls.update()
  }

  private async _enterStrategyMode(): Promise<void> {
    const config = await this.setupPanel.show()
    if (!config) return

    // Rebuild board for the requested dimensions
    this._rebuildBoard(config.rows, config.cols)

    const infoOps = await InfoStateOps.create(config.rows, config.cols)
    this.stratGen = new StrategyGenerator(infoOps, config)
    this.mode = 'strategy'

    this.hudTop.style.display = 'none'
    this.hudBottom.style.display = 'none'
    this.actionPanel.show()
    this.infoPanel.show()
    this._updateStrategyView()
  }

  private _exitStrategyMode(): void {
    this._clearSelection()
    // Clear prob overlay labels directly (in case _clearSelection skipped due to empty stratGen)
    for (const el of this.probLabels.values()) el.remove()
    this.probLabels.clear()
    this.mode = 'play'
    this.stratGen = null
    this.actionPanel.hide()
    this.infoPanel.hide()

    // Restore play-mode board and HUD
    this.hudTop.style.display = 'flex'
    this.hudBottom.style.display = 'flex'
    this._rebuildBoard(BOARD_ROWS, BOARD_COLS)
    this.board.applyBoard(this.engine.trueBoard(), this.lastMoveIndex)
    this._updateHud()
  }

  /** Confirm selected actions with given probabilities. */
  private _handleConfirm(actions: number[], probs: number[]): void {
    if (!this.stratGen) return
    try {
      const complete = this.stratGen.submitActions(actions, probs)
      this._clearSelection()
      this._updateStrategyView()
      if (complete) this._showExportDialog()
    } catch (err) {
      console.error('Strategy action error:', err)
    }
  }

  private _handleStrategyAction(input: string): void {
    if (!this.stratGen) return
    try {
      const complete = this.stratGen.iterateBoard(input)
      this._clearSelection()
      this._updateStrategyView()
      if (complete) {
        this._showExportDialog()
      }
    } catch (err) {
      console.error('Strategy action error:', err)
    }
  }

  private _strategyUndo(): void {
    if (!this.stratGen) return
    this._clearSelection()
    this.stratGen.rewind()
    this._updateStrategyView()
  }

  private _strategyRestart(): void {
    if (!this.stratGen) return
    this._clearSelection()
    this.stratGen.restart()
    this._updateStrategyView()
  }

  private _clearSelection(): void {
    for (const cellIdx of this.selectedTiles.keys()) {
      this.board.tileByIndex(cellIdx).setSelected(false)
    }
    this.selectedTiles.clear()
    this._syncSelectionUI()
  }

  /** Sync the action panel and probability overlays with current selection. */
  private _syncSelectionUI(): void {
    if (!this.stratGen) return
    this.actionPanel.updateSelection(this.selectedTiles, this.stratGen.cols)
    this._updateProbOverlays()
  }

  /** Create/remove probability label overlays for selected tiles. */
  private _updateProbOverlays(): void {
    // Remove all existing labels
    for (const el of this.probLabels.values()) el.remove()
    this.probLabels.clear()

    const n = this.selectedTiles.size
    if (n === 0) return

    const prob = (1 / n).toFixed(2)
    for (const cellIdx of this.selectedTiles.keys()) {
      const label = document.createElement('div')
      label.textContent = prob
      label.style.cssText = `
        position: absolute; transform: translate(-50%, -50%);
        color: #fff; font-family: 'Courier New', monospace; font-size: 14px;
        font-weight: bold; text-shadow: 0 1px 3px rgba(0,0,0,0.7);
        pointer-events: none;
      `
      this.probOverlay.appendChild(label)
      this.probLabels.set(cellIdx, label)
    }
  }

  /** Sync board overlay labels when prob inputs change in the toolbar. */
  private _syncProbOverlaysFromInputs(probs: Map<number, number>): void {
    for (const [cellIdx, prob] of probs) {
      const label = this.probLabels.get(cellIdx)
      if (label) {
        label.textContent = prob.toFixed(2)
      }
    }
  }

  /** Project tile 3D positions to screen coordinates for prob labels. */
  private _updateProbLabelPositions(): void {
    if (this.probLabels.size === 0) return
    const rect = this.renderer.domElement.getBoundingClientRect()
    const vec = new THREE.Vector3()

    for (const [cellIdx, label] of this.probLabels) {
      const tile = this.board.tileByIndex(cellIdx)
      vec.setFromMatrixPosition(tile.group.matrixWorld)
      vec.y += 0.3 // slightly above tile surface
      vec.project(this.camera)

      const x = (vec.x * 0.5 + 0.5) * rect.width
      const y = (-vec.y * 0.5 + 0.5) * rect.height
      label.style.left = `${x}px`
      label.style.top = `${y}px`
    }
  }

  private _updateStrategyView(): void {
    if (!this.stratGen) return
    const view = this.stratGen.boardView
    this.board.applyView(view, this.stratGen.lastCollisionIndex)

    const { assigned, remaining } = this.stratGen.progress
    this.actionPanel.updateSelection(this.selectedTiles, this.stratGen.cols)
    this.infoPanel.update({
      infoState: this.stratGen.currentInfoState,
      assigned,
      remaining,
      player: this.stratGen.player,
      isCollision: this.stratGen.lastCollisionIndex !== null,
    })
  }

  private _showExportDialog(): void {
    if (!this.stratGen) return
    const policy = this.stratGen.exportPolicy()
    const json = JSON.stringify(policy, null, 2)
    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `policy_${this.stratGen.rows}x${this.stratGen.cols}_p${this.stratGen.player}.json`
    a.click()
    // Defer revocation so the browser has time to start the download
    setTimeout(() => URL.revokeObjectURL(url), 5000)
  }

  private _onResize = (): void => {
    const w = this.container.clientWidth
    const h = this.container.clientHeight
    const aspect = w / h
    const frustum = 5.5
    this.camera.left = -frustum * aspect
    this.camera.right = frustum * aspect
    this.camera.top = frustum
    this.camera.bottom = -frustum
    this.camera.updateProjectionMatrix()
    this.renderer.setSize(w, h)
    this.outlineRender.resize(w, h)
  }

  // ── HUD ───────────────────────────────────────────────────────────────

  private _buildHud(): void {
    this.hudTop = document.createElement('div')
    this.hudTop.style.cssText = `
      position: absolute; top: 0; left: 0; width: 100%; pointer-events: none;
      font-family: 'Courier New', monospace; color: #4a5568;
      padding: 16px 24px; display: flex; flex-direction: column; gap: 4px;
    `
    this.container.style.position = 'relative'
    this.container.appendChild(this.hudTop)

    const title = document.createElement('div')
    title.textContent = 'DSaGe — Dark Hex Strategy Generator'
    title.style.cssText = 'font-size: 13px; color: #64748b;'
    this.hudTop.appendChild(title)

    this.statusEl = document.createElement('div')
    this.statusEl.style.cssText = 'font-size: 15px; margin-top: 4px;'
    this.hudTop.appendChild(this.statusEl)

    this.hudBottom = document.createElement('div')
    this.hudBottom.style.cssText = `
      position: absolute; bottom: 0; left: 0; width: 100%;
      padding: 10px 24px; font-size: 11px; color: #64748b;
      display: flex; justify-content: space-between; pointer-events: none;
    `
    this.container.appendChild(this.hudBottom)

    this.infoEl = document.createElement('div')
    this.hudBottom.appendChild(this.infoEl)

    const legend = document.createElement('div')
    legend.innerHTML =
      '<span style="color:#5c6bc0">● Black</span> top↔bottom &nbsp; ' +
      '<span style="color:#ef5350">● White</span> left↔right &nbsp; ' +
      '<span style="color:#94a3b8">[R] restart &nbsp; [S] strategy</span>'
    this.hudBottom.appendChild(legend)
  }

  private _updateHud(): void {
    if (this.engine.isTerminal) {
      const w = this.engine.winner!
      const color = w === 0 ? '#5c6bc0' : '#ef5350'
      this.statusEl.innerHTML = `<span style="color:${color}">${PLAYER_LABEL[w]} wins!</span>`
      this.infoEl.textContent = ''
    } else {
      const p = this.engine.currentPlayer
      const color = p === 0 ? '#5c6bc0' : '#ef5350'
      const n = this.engine.legalActions().length
      this.statusEl.innerHTML =
        `<span style="color:${color}">${PLAYER_LABEL[p]}'s turn</span>` +
        ` <span style="color:#94a3b8; font-size:13px">(${n} moves)</span>`
      this.infoEl.textContent = `info: ${this.engine.infoStateString(p)}`
    }
  }
}
