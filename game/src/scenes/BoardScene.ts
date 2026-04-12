import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'
import { BoardLayout3D } from '../board/BoardLayout'
import { HexTile3D } from '../board/HexTile'
import { boardCenter, PALETTE } from '../board/IsometricHex'
import { InfoStateOps } from '../engine/InfoStateOps'
import { createOutlineComposer } from '../postprocess/OutlinePostProcess'
import { StrategyGenerator } from '../strategy/StrategyGenerator'
import { SetupPanel } from '../ui/SetupPanel'
import { ActionPanel } from '../ui/ActionPanel'
import { InfoPanel } from '../ui/InfoPanel'
import { CompletionPanel } from '../ui/CompletionPanel'
import { MainMenuPanel } from '../ui/MainMenuPanel'
import { InvestigationView } from '../investigation/InvestigationView'
import { loadPolicyFromFile } from '../investigation/policyImport'
import { ensureAudioReady, playThock, playPlace, playDrop, playReveal, playVanish, playChime } from '../audio/SoundEngine'
import { TutorialEngine } from '../tutorial/TutorialEngine'
import type { TutorialHooks, TutorialSceneAdapter } from '../tutorial/TutorialEngine'

const BOARD_ROWS = 4
const BOARD_COLS = 3

/**
 * Main 3D scene: orthographic camera at isometric angle,
 * warm lighting to match the mauve / board-game aesthetic.
 */
export class BoardScene {
  private renderer: THREE.WebGLRenderer
  private scene: THREE.Scene
  private camera: THREE.OrthographicCamera
  private controls: OrbitControls

  private board!: BoardLayout3D

  private outlineRender!: ReturnType<typeof createOutlineComposer>

  private raycaster = new THREE.Raycaster()
  private pointer = new THREE.Vector2()
  private hoveredTile: HexTile3D | null = null

  private clock = new THREE.Clock()
  private _rafId = 0

  // Strategy mode
  private _menuOpen = false
  private _setupOpen = false
  private _completionOpen = false
  private _investigationOpen = false
  private _sessionToken = 0 // incremented on each menu return; guards stale async continuations
  private stratGen: StrategyGenerator | null = null
  private mainMenu!: MainMenuPanel
  private setupPanel!: SetupPanel
  private actionPanel!: ActionPanel
  private infoPanel!: InfoPanel
  private completionPanel!: CompletionPanel
  private selectedTiles: Map<number, number> = new Map() // cellIndex → 1 (tracked for selection)
  private probOverlay!: HTMLDivElement // container for probability labels
  private probLabels: Map<number, HTMLDivElement> = new Map()
  private _lastClickTime = 0
  private _lastClickTile = -1

  // Tutorial mode
  private _tutorialActive = false
  private _tutorialHooks: TutorialHooks | null = null
  private _investigationView: InvestigationView | null = null

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

    // ── Events ────────────────────────────────────────────────────────────
    this.container.style.position = 'relative'
    this.renderer.domElement.addEventListener('pointermove', this._onPointerMove)
    this.renderer.domElement.addEventListener('pointerdown', this._onPointerDown)
    window.addEventListener('resize', this._onResize)
    window.addEventListener('keydown', this._onKeyDown)
  }

  async init(): Promise<void> {
    this.board = new BoardLayout3D(this.scene, BOARD_ROWS, BOARD_COLS)
    this.mainMenu = new MainMenuPanel(this.container)
    this.setupPanel = new SetupPanel(this.container)
    this.actionPanel = new ActionPanel(this.container)
    this.infoPanel = new InfoPanel(this.container)
    this.completionPanel = new CompletionPanel(this.container)

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

    this._animate()

    // Show main menu (board rotates gently in background)
    this._showMainMenu()
  }

  // ── Render loop ─────────────────────────────────────────────────────────

  private _animate = (): void => {
    this._rafId = requestAnimationFrame(this._animate)
    const dt = this.clock.getDelta()
    this.controls.update()
    if (this.board) this.board.tickAnimations(dt)
    this._updateProbLabelPositions()
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
      if (tile && tile.state === 'empty') {
        tile.setHovered(true)
        playThock()
      }
      this.hoveredTile = tile
    }
  }

  private _onPointerDown = (e: PointerEvent): void => {
    ensureAudioReady()
    this._updatePointer(e)
    const tile = this._raycastTile()
    if (!tile) return
    if (!this.stratGen) return
    if (tile.state !== 'empty') return

    const now = Date.now()
    const isDoubleClick = (now - this._lastClickTime < 400) && tile.cellIndex === this._lastClickTile
    this._lastClickTime = now
    this._lastClickTile = tile.cellIndex

    if (isDoubleClick) {
      // Double-click: instant deterministic action
      this._clearSelection()
      playPlace()
      this._tutorialHooks?.onDoubleClickTile?.(tile.cellIndex)
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
      playPlace()
    }
    this._syncSelectionUI()
    this._tutorialHooks?.onTileSelected?.(tile.cellIndex, this.selectedTiles.size)
  }

  private _confirmingLeave = false

  private _onKeyDown = (e: KeyboardEvent): void => {
    if (this._menuOpen || this._setupOpen || this._completionOpen || this._confirmingLeave || this._investigationOpen) return
    if (this._tutorialActive) {
      // During tutorial, only allow Enter for confirming actions (Esc is blocked)
      if (e.key === 'Enter' && this.selectedTiles.size > 0) {
        const result = this.actionPanel.getActionProbs()
        if (result) this._handleConfirm(result.actions, result.probs)
      }
      return
    }
    if (e.key === 'Escape') {
      if (this.stratGen && this.stratGen.progress.assigned > 0) {
        this._confirmLeave()
      } else {
        this._returnToMenu()
      }
    }
    if (e.key === 'Enter' && this.selectedTiles.size > 0) {
      const result = this.actionPanel.getActionProbs()
      if (result) this._handleConfirm(result.actions, result.probs)
    }
  }

  // ── Leave confirmation ─────────────────────────────────────────────────────

  private _confirmLeave(): void {
    this._confirmingLeave = true

    const overlay = document.createElement('div')
    overlay.style.cssText = `
      position: fixed; top: 0; left: 0; width: 100%; height: 100%;
      background: rgba(180, 160, 140, 0.45); backdrop-filter: blur(4px);
      z-index: 100; display: flex; align-items: center; justify-content: center;
    `

    const panel = document.createElement('div')
    panel.style.cssText = `
      background: #faf5ef; border-radius: 14px; padding: 28px 32px;
      min-width: 320px; max-width: 380px; text-align: center;
      box-shadow: 0 8px 32px rgba(100, 60, 60, 0.18), 0 2px 8px rgba(100, 60, 60, 0.10);
      border: 2px solid #e8d5d5;
      font-family: 'Nunito', -apple-system, BlinkMacSystemFont, sans-serif; color: #4a3535;
    `
    panel.innerHTML = `
      <h2 style="margin: 0 0 8px; font-size: 20px; font-weight: 800; color: #995a5a;">Leave Strategy?</h2>
      <p style="margin: 0 0 24px; font-size: 14px; color: #8a7070;">
        Current progress will be lost. You can download the policy first from the completion screen.
      </p>
      <div style="display: flex; gap: 10px;">
        <button id="leave-cancel" style="
          flex: 1; padding: 12px 16px; font-size: 15px; font-weight: 700;
          font-family: 'Nunito', sans-serif; color: #4a3535; background: #fff;
          border: 2px solid #e8d5d5; border-radius: 10px; cursor: pointer;
        ">Keep Working</button>
        <button id="leave-confirm" style="
          flex: 1; padding: 12px 16px; font-size: 15px; font-weight: 800;
          font-family: 'Nunito', sans-serif; color: #fff; background: #c75000;
          border: none; border-radius: 10px; cursor: pointer;
          box-shadow: 0 3px 0 #9a3d00, 0 4px 12px rgba(100, 60, 60, 0.2);
        ">Leave</button>
      </div>
    `
    overlay.appendChild(panel)
    this.container.appendChild(overlay)

    const cleanup = () => {
      overlay.remove()
      this._confirmingLeave = false
    }

    panel.querySelector('#leave-cancel')!.addEventListener('click', () => cleanup())
    panel.querySelector('#leave-confirm')!.addEventListener('click', () => {
      cleanup()
      this._returnToMenu()
    })
  }

  // ── Main menu ──────────────────────────────────────────────────────────────

  private async _showMainMenu(): Promise<void> {
    this._menuOpen = true
    this.controls.autoRotate = true
    this.controls.autoRotateSpeed = 0.3

    while (true) {
      const choice = await this.mainMenu.show()
      this._menuOpen = false
      this.controls.autoRotate = false

      switch (choice) {
        case 'strategy-generator': {
          const token = this._sessionToken
          this._setupOpen = true
          const config = await this.setupPanel.show(true) // cancellable — Cancel returns to menu
          this._setupOpen = false

          if (!config) {
            // User cancelled setup — loop back to menu
            this._menuOpen = true
            this.controls.autoRotate = true
            this.controls.autoRotateSpeed = 0.3
            continue
          }
          if (token !== this._sessionToken) return // session was cancelled while awaiting

          this._rebuildBoard(config.rows, config.cols)
          const infoOps = await InfoStateOps.create(config.rows, config.cols)
          if (token !== this._sessionToken) return // session was cancelled while WASM loaded

          this.stratGen = new StrategyGenerator(infoOps, config)
          this.actionPanel.show()
          this.infoPanel.show()
          this._updateStrategyView()
          return
        }

        case 'tutorial': {
          await this._runTutorial()
          this._menuOpen = true
          this.controls.autoRotate = true
          this.controls.autoRotateSpeed = 0.3
          continue
        }

        case 'strategy-investigation': {
          const token = this._sessionToken
          const loaded = await loadPolicyFromFile()
          if (!loaded) {
            // Cancelled file picker — loop back to menu
            this._menuOpen = true
            this.controls.autoRotate = true
            this.controls.autoRotateSpeed = 0.3
            continue
          }
          if (token !== this._sessionToken) return // session cancelled while picking file
          try {
            await this._runInvestigation(loaded.policy, loaded.config)
          } catch (err) {
            console.error('Investigation failed:', err)
            // Bad policy file — fall through to menu
          }
          // Return to menu after investigation exits
          this._menuOpen = true
          this.controls.autoRotate = true
          this.controls.autoRotateSpeed = 0.3
          continue
        }
      }
    }
  }

  private async _returnToMenu(): Promise<void> {
    this._sessionToken++ // invalidate any in-flight async from prior session
    this.actionPanel.hide()
    this.infoPanel.hide()
    this._clearSelection()
    for (const el of this.probLabels.values()) el.remove()
    this.probLabels.clear()
    this.stratGen = null
    this._rebuildBoard(BOARD_ROWS, BOARD_COLS)
    await this._showMainMenu()
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

    // Wire up sounds on all tiles
    for (const tile of this.board.allTiles()) {
      tile.onDropLand = () => {
        if (tile.stoneAnim === 'rise') playReveal()
        else playDrop()
      }
      tile.onVanishStart = () => playVanish()
    }

    const [cx, , cz] = boardCenter(rows, cols)
    const dist = 14
    this.camera.position.set(cx + dist * 0.55, dist * 0.7, cz + dist * 0.55)
    this.camera.lookAt(cx, 0, cz)
    this.controls.target.set(cx, 0, cz)
    this.controls.update()
  }

  /** Confirm selected actions with given probabilities. */
  private _handleConfirm(actions: number[], probs: number[]): void {
    if (!this.stratGen) return
    try {
      const complete = this.stratGen.submitActions(actions, probs)
      this._clearSelection()
      // Collision cells should rise (already there), not drop
      const dropCells = new Set(actions)
      if (this.stratGen.lastCollisionIndex !== null) {
        dropCells.delete(this.stratGen.lastCollisionIndex)
      }
      this._updateStrategyView(dropCells)
      this._tutorialHooks?.onActionConfirmed?.()
      if (complete && !this._tutorialActive) this._showCompletionFlow()
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
        this._showCompletionFlow()
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
        color: #fff; font-family: 'Nunito', -apple-system, sans-serif; font-size: 16px;
        font-weight: 800; text-shadow: 0 1px 4px rgba(0,0,0,0.5);
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

  /** @param playerActions cells the player just acted on (drop anim); others rise */
  private _updateStrategyView(playerActions?: Set<number>): void {
    if (!this.stratGen) return
    const view = this.stratGen.boardView
    this.board.applyView(view, this.stratGen.lastCollisionIndex, 'rise', playerActions)

    const { assigned, remaining } = this.stratGen.progress
    this.actionPanel.updateSelection(this.selectedTiles, this.stratGen.cols)
    this.infoPanel.update({
      infoState: this.stratGen.currentInfoState,
      assigned,
      remaining,
      player: this.stratGen.player,
      isCollision: this.stratGen.lastCollisionIndex !== null,
      perfectRecall: this.stratGen.perfectRecall,
    })
  }

  private async _showCompletionFlow(): Promise<void> {
    if (!this.stratGen) return

    // Play celebration chime
    playChime()
    this.completionPanel.resetDownloaded()

    // Loop: re-show completion modal if user cancels the setup dialog or dismisses
    while (true) {
      this._completionOpen = true
      const { assigned } = this.stratGen!.progress
      const action = await this.completionPanel.show({
        player: this.stratGen!.player,
        rows: this.stratGen!.rows,
        cols: this.stratGen!.cols,
        infoStates: assigned,
      })
      this._completionOpen = false

      if (action === 'download') {
        this._downloadPolicy()
        // After download, re-show the modal so user can still start new
        continue
      } else if (action === 'new') {
        this._returnToMenu()
        return
      } else if (action === 'investigate') {
        this._completionOpen = false
        const policy = this.stratGen!.policy
        const config = {
          rows: this.stratGen!.rows,
          cols: this.stratGen!.cols,
          player: this.stratGen!.player,
          perfectRecall: this.stratGen!.perfectRecall,
        }
        await this._runInvestigation(policy, config)
        // Return to completion modal (strategy preserved) — user can download/new/dismiss
        continue
      } else {
        // Dismissed — let user keep inspecting the completed board
        this._completionOpen = false
        return
      }
    }
  }

  private async _runInvestigation(policy: import('../strategy/types').Policy, config: import('../strategy/types').StrategyConfig): Promise<void> {
    const token = this._sessionToken
    const infoOps = await InfoStateOps.create(config.rows, config.cols)
    if (token !== this._sessionToken) return

    this._investigationOpen = true
    const view = new InvestigationView(this.container)
    try {
      await view.run(policy, config, infoOps)
    } finally {
      view.dispose()
      this._investigationOpen = false
    }
  }

  private _downloadPolicy(): void {
    if (!this.stratGen) return
    const policy = this.stratGen.exportPolicy()
    const json = JSON.stringify(policy, null, 2)
    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `policy_${this.stratGen.rows}x${this.stratGen.cols}_p${this.stratGen.player}.json`
    a.click()
    setTimeout(() => URL.revokeObjectURL(url), 5000)
  }

  // ── Tutorial ────────────────────────────────────────────────────────────

  private async _runTutorial(): Promise<void> {
    const adapter: TutorialSceneAdapter = {
      getCanvasRect: () => this.renderer.domElement.getBoundingClientRect(),
      getActionPanelRect: () => {
        const el = document.querySelector('[data-tutorial="action-panel"]')
        return el?.getBoundingClientRect() ?? null
      },
      getInfoPanelRect: () => {
        const el = document.querySelector('[data-tutorial="info-panel"]')
        return el?.getBoundingClientRect() ?? null
      },
      getTileScreenRect: (cellIndex: number) => {
        if (!this.board) return null
        const tile = this.board.tileByIndex(cellIndex)
        if (!tile) return null
        const vec = new THREE.Vector3()
        vec.setFromMatrixPosition(tile.group.matrixWorld)
        vec.project(this.camera)
        const rect = this.renderer.domElement.getBoundingClientRect()
        const x = (vec.x * 0.5 + 0.5) * rect.width + rect.left
        const y = (-vec.y * 0.5 + 0.5) * rect.height + rect.top
        return new DOMRect(x - 45, y - 45, 90, 90)
      },
      forceStartStrategy: async (rows: number, cols: number, player: number) => {
        this._rebuildBoard(rows, cols)
        const infoOps = await InfoStateOps.create(rows, cols)
        this.stratGen = new StrategyGenerator(infoOps, { rows, cols, player, perfectRecall: false })
        this.actionPanel.show()
        this.infoPanel.show()
        this._updateStrategyView()
      },
      autoCompleteStrategy: () => {
        if (!this.stratGen) return
        while (!this.stratGen.isComplete) {
          this.stratGen.iterateBoard('r')
        }
        this._clearSelection()
        this._updateStrategyView()
      },
      hasCollision: () => {
        return this.stratGen?.lastCollisionIndex !== null && this.stratGen?.lastCollisionIndex !== undefined
      },
      isStrategyComplete: () => {
        return this.stratGen?.isComplete ?? false
      },
      getContainer: () => this.container,
      runInvestigation: async () => {
        if (!this.stratGen) return
        const policy = this.stratGen.policy
        const config = {
          rows: this.stratGen.rows,
          cols: this.stratGen.cols,
          player: this.stratGen.player,
          perfectRecall: this.stratGen.perfectRecall,
        }
        const infoOps = await InfoStateOps.create(config.rows, config.cols)
        this._investigationOpen = true
        this._investigationView = new InvestigationView(this.container)
        try {
          await this._investigationView.run(policy, config, infoOps)
        } finally {
          this._investigationView?.dispose()
          this._investigationView = null
          this._investigationOpen = false
        }
      },
      closeInvestigation: () => {
        this._investigationView?.dispose()
        this._investigationView = null
        this._investigationOpen = false
      },
      setTutorialHooks: (hooks: TutorialHooks | null) => {
        this._tutorialHooks = hooks
      },
      setTutorialActive: (active: boolean) => {
        this._tutorialActive = active
      },
      setOrbitEnabled: (enabled: boolean) => {
        this.controls.enabled = enabled
      },
      prefillSetup: (rows: number, cols: number, _player: number) => {
        const rowsInput = document.querySelector('#sg-rows') as HTMLInputElement | null
        const colsInput = document.querySelector('#sg-cols') as HTMLInputElement | null
        if (rowsInput) rowsInput.value = String(rows)
        if (colsInput) colsInput.value = String(cols)
      },
      showSetupPanel: () => {
        // Show the setup overlay without blocking (for tutorial display only)
        const overlay = this.setupPanel as unknown as { overlay: HTMLDivElement }
        if (overlay.overlay) overlay.overlay.style.display = 'flex'
      },
      hideSetupPanel: () => {
        const overlay = this.setupPanel as unknown as { overlay: HTMLDivElement }
        if (overlay.overlay) overlay.overlay.style.display = 'none'
      },
    }

    const engine = new TutorialEngine(adapter, this.container)
    await engine.run()

    // Clean up after tutorial: return to menu state
    this._tutorialActive = false
    this._tutorialHooks = null
    this.actionPanel.hide()
    this.infoPanel.hide()
    this._clearSelection()
    for (const el of this.probLabels.values()) el.remove()
    this.probLabels.clear()
    this.stratGen = null

    // Close investigation if still open
    if (this._investigationOpen) {
      this._investigationView?.dispose()
      this._investigationView = null
      this._investigationOpen = false
    }

    this._rebuildBoard(BOARD_ROWS, BOARD_COLS)
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

}
